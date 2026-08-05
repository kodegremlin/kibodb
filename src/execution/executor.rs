use crate::{
    catalog::manager::CatalogManager, error::Error, relation::tuple::Tuple,
    storage::buffer_pool::BufferPool,
};

/// The runtime context that will be passed down the Volcano execution Tree.
/// For a running query, it will provide access to storage, metadata, and a
/// reusable memory block.
pub struct ExecutionContext<'a> {
    /// A reference to the buffer pool for fetching page.
    pub buffer_pool: &'a BufferPool,

    /// A reference to the catalog for O(1) metadata lookups.
    pub catalog: &'a CatalogManager,

    /// A reusable buffer to avoid heap allocation when fetching raw bytes
    /// from pages, defaults as 2KiB.
    pub buffer_block: Vec<u8>,
}

impl<'a> ExecutionContext<'a> {
    /// Initializes a new `ExecutionContext`.
    pub fn new(pool: &'a BufferPool, catalog: &'a CatalogManager) -> Self {
        Self {
            buffer_pool: pool,
            catalog,
            buffer_block: Vec::with_capacity(2048),
        }
    }
}

/// The Volcano execution model interface.
pub trait Executor {
    /// Pulls the next [Tuple] from the tree.
    fn next(&mut self, context: ExecutionContext) -> Result<Option<Tuple>, Error>;
}
