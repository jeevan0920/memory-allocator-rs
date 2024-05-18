use std::alloc::{alloc, dealloc, Layout};
use std::collections::VecDeque;
use std::ptr::NonNull;

pub struct MemoryBlock {
    ptr: NonNull<u8>,
    size: usize,
}

impl MemoryBlock {
    pub fn new(size: usize) -> Self {
        let layout = Layout::from_size_align(size, 8).unwrap();
        let ptr = unsafe { NonNull::new(alloc(layout)).expect("Failed to allocate memory") };
        Self { ptr, size }
    }
}

impl Drop for MemoryBlock {
    fn drop(&mut self) {
        let layout = Layout::from_size_align(self.size, 8).unwrap();
        unsafe { dealloc(self.ptr.as_ptr(), layout) }
    }
}

pub struct MemoryPool {
    free_list: VecDeque<MemoryBlock>,
    allocated_list: Vec<MemoryBlock>,
    pool_size: usize,
}

impl MemoryPool {
    pub fn new(pool_size: usize) -> Self {
        Self {
            free_list: VecDeque::new(),
            allocated_list: Vec::new(),
            pool_size,
        }
    }

    pub fn allocate(&mut self, size: usize) -> Option<NonNull<u8>> {
        for i in 0..self.free_list.len() {
            if self.free_list[i].size >= size {
                let block = self.free_list.remove(i).unwrap();
                let ptr = block.ptr.clone();
                self.allocated_list.push(block);
                return Some(ptr);
            }
        }

        if self.pool_size >= size {
            let block = MemoryBlock::new(size);
            let ptr = block.ptr.clone();
            self.allocated_list.push(block);
            self.pool_size -= size;
            return Some(ptr);
        }

        None
    }

    pub fn deallocate(&mut self, ptr: NonNull<u8>) {
        let pos = self.allocated_list.iter().position(|b| b.ptr == ptr);

        if let Some(index) = pos {
            let block = self.allocated_list.swap_remove(index);
            self.free_list.push_back(block);
        }
    }
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_pool() {
        let mut pool = MemoryPool::new(1024);

        // Allocate a block of 256 bytes
        let ptr1 = pool.allocate(256).unwrap();
        assert!(pool.allocate(800).is_none());

        // Deallocate the block
        pool.deallocate(ptr1);

        // Allocate again and check if it reuses the block
        let ptr2 = pool.allocate(256).unwrap();
        assert_eq!(ptr1, ptr2);
    }
}
