use std::{collections::HashMap, io::Cursor};

use crate::{
    error::Error,
    relation::{
        catalog::{sys_pages_schema, sys_schema_schema},
        schema::{Column, Schema},
        tuple::Tuple,
        types::DataType,
    },
    storage::{
        buffer_pool::BufferPool,
        page::{BTreeNode, PageId},
    },
};

pub const SYS_PAGES_ROOT_ID: PageId = PageId(1);
pub const SYS_SCHEMA_ROOT_ID: PageId = PageId(2);

/// The in-memory metadata cache for the entire database.
///
/// The `CatalogManager` is responsible for bootstrapping the database on startup,
/// reading the physical `sys_pages` and `sys_schema` BpTrees, and caching their
/// contents to provide constant time lookups for the Binder and Planner layer, when
/// queries are run.
#[derive(Debug)]
pub struct CatalogManager {
    /// Maps a table name to its physical B-Tree root [PageId].
    table_roots: HashMap<String, PageId>,

    /// Maps a table name to its logical [Schema] definition.
    table_schema: HashMap<String, Schema>,
}

impl CatalogManager {
    /// Initializes an empty CatalogManager.
    pub fn new() -> Self {
        Self {
            table_roots: HashMap::new(),
            table_schema: HashMap::new(),
        }
    }

    /// Bootstraps the catalog from disk. If the database is empty, it allocates and
    /// initializes the system catalog pages.
    pub fn bootstrap(&mut self, pool: &mut BufferPool) -> Result<(), Error> {
        if pool.is_empty() {
            self.initialize_new_database(pool)?;
        }
        let sys_pages_schema = sys_pages_schema();

        // Load `sys_pages` as {table_name -> root_page_id} in the catalog map
        Self::scan_system_table(pool, SYS_PAGES_ROOT_ID, &sys_pages_schema, |tuple| {
            let table_name = tuple.values[0]
                .varchar_to_str()
                .ok_or_else(|| Error::CorruptPage("sys_pages table_name is not Varchar".into()))?;

            let root_page_id = tuple.values[1]
                .bigint_to_i64()
                .ok_or_else(|| Error::CorruptPage("sys_pages root_page_id is not BigInt".into()))?;

            self.table_roots
                .insert(table_name.to_string(), PageId(root_page_id as u64));
            Ok(())
        })?;
        let mut raw_columns = HashMap::new();
        let sys_schema_schema = sys_schema_schema();

        // Load `sys_schema` as {table_name -> vec[columns]} in temp raw_columns
        Self::scan_system_table(pool, SYS_SCHEMA_ROOT_ID, &sys_schema_schema, |tuple| {
            let table_name = tuple.values[0]
                .varchar_to_str()
                .ok_or_else(|| Error::CorruptPage("sys_schema table_name is not Varchar".into()))?;

            let field_name = tuple.values[1]
                .varchar_to_str()
                .ok_or_else(|| Error::CorruptPage("sys_schema field_name is not Varchar".into()))?;

            let field_type = tuple.values[2]
                .int_to_i32()
                .ok_or_else(|| Error::CorruptPage("sys_schema field_type is not Int".into()))?;

            let field_length = tuple.values[3]
                .int_to_i32()
                .ok_or_else(|| Error::CorruptPage("sys_schema field_length is not Int".into()))?;

            let data_type = DataType::from_u8(field_type as u8)?;
            let length = (field_length > 0).then_some(field_length as u32);

            let column = Column::new(field_name, data_type, length);
            raw_columns
                .entry(table_name.to_string())
                .or_insert_with(Vec::new)
                .push(column);
            Ok(())
        })?;
        self.table_schema.extend(
            raw_columns
                .into_iter()
                .map(|(name, col)| (name, Schema::new(col))),
        );
        Ok(())
    }

    /// Retrieves the physical root PageId for a given table name.
    pub fn get_table_root(&self, table_name: &str) -> Result<PageId, Error> {
        self.table_roots
            .get(table_name)
            .copied()
            .ok_or_else(|| Error::TableNotFound(table_name.into()))
    }

    /// Retrieves the logical Schema for a given table name.
    pub fn get_table_schema(&self, table_name: &str) -> Result<&Schema, Error> {
        self.table_schema
            .get(table_name)
            .ok_or_else(|| Error::TableNotFound(table_name.into()))
    }

    /// Locates and Reads system pages into memory and processes them against the
    /// provided schema before passing them to the given closure.
    fn scan_system_table<F>(
        pool: &mut BufferPool,
        root_id: PageId,
        schema: &Schema,
        mut process_tuple: F,
    ) -> Result<(), Error>
    where
        F: FnMut(Tuple) -> Result<(), Error>,
    {
        let mut curr_page_id = root_id;
        loop {
            let frame = pool.fetch_page(curr_page_id)?;
            let node_guard = frame.read();

            match &*node_guard {
                BTreeNode::Internal(node) => {
                    if !node.slot_array.is_empty() {
                        let rec_idx = node.slot_array[0] as usize;
                        curr_page_id = node.entries[rec_idx].child_page_id;
                    } else {
                        curr_page_id = node.rightmost_child_id;
                    }
                }
                _ => break,
            }
        }
        loop {
            let frame = pool.fetch_page(curr_page_id)?;
            let node_guard = frame.read();

            let BTreeNode::Leaf(node) = &*node_guard else {
                return Err(Error::CorruptPage(
                    "expected a leaf node during horizontal scan".into(),
                ));
            };
            for &rec_idx in &node.slot_array {
                let record = &node.records[rec_idx as usize];
                if !record.is_deleted {
                    let mut cursor = Cursor::new(&record.data);
                    let tuple = Tuple::decode(schema, &mut cursor)?;
                    process_tuple(tuple)?;
                }
            }
            if !node.has_next {
                break;
            }
            curr_page_id = node.next_page_id;
        }
        Ok(())
    }

    /// Allocates the baseline system pages for a completely fresh database.
    fn initialize_new_database(&mut self, pool: &mut BufferPool) -> Result<(), Error> {
        let (p1_id, p1_frame) = pool.new_page(true)?;
        if p1_id != SYS_PAGES_ROOT_ID {
            return Err(Error::CorruptPage(format!(
                "failed to allocate sys_pages at PageId 1, got={:?}",
                p1_id
            )));
        }
        p1_frame.write().mark_dirty(0);

        let (p2_id, p2_frame) = pool.new_page(true)?;
        if p2_id != SYS_SCHEMA_ROOT_ID {
            return Err(Error::CorruptPage(format!(
                "failed to allocate sys_schema at PageId 2, got={:?}",
                p2_id
            )));
        }
        p2_frame.write().mark_dirty(0);

        pool.flush_all_pages()?;
        Ok(())
    }
}

// TODO: write tests unless it already works correctly :)
