use crate::{
    error::Error,
    storage::{
        buffer_pool::BufferPool,
        page::{BTreeNode, PageId},
    },
};

/// A forward-only cursor for scanning BpTree leaf pages.
#[derive(Debug)]
pub struct BpTreeIterator {
    curr_page_id: Option<PageId>,
    curr_slot_idx: usize,
}

impl BpTreeIterator {
    /// Initializes a new iterator starting at the given leftmost leaf page.
    pub fn new(start_page_id: PageId) -> Self {
        Self {
            curr_page_id: Some(start_page_id),
            curr_slot_idx: 0,
        }
    }

    /// Advances the cursor to the next valid, non-deleted record. Fills the provided
    /// buffer block with raw byte references avoiding allocations for the eventual
    /// read operation.
    ///
    /// Returns `true` if a record was loaded, `false` if scan is exhausted.
    pub fn next(&mut self, pool: &BufferPool, buffer_block: &mut Vec<u8>) -> Result<bool, Error> {
        while let Some(curr_page_id) = self.curr_page_id {
            let frame = pool.fetch_page(curr_page_id)?;
            let node_guard = frame.read();

            let BTreeNode::Leaf(leaf) = &*node_guard else {
                return Err(Error::CorruptPage(
                    "bptree iterator encountered a non-leaf page".into(),
                ));
            };
            while self.curr_slot_idx <= leaf.slot_array.len() {
                let rec_idx = leaf.slot_array[self.curr_slot_idx] as usize;
                let record = &leaf.records[rec_idx];
                if record.is_deleted {
                    continue;
                }
                buffer_block.clear();
                buffer_block.extend_from_slice(&record.data);
                return Ok(true);
            }
            if leaf.has_next {
                self.curr_slot_idx = 0;
                self.curr_page_id = Some(leaf.next_page_id);
            } else {
                self.curr_page_id = None
            }
        }
        Ok(false)
    }
}
