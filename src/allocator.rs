use core::{
    alloc::{GlobalAlloc, Layout},
    ptr,
};

pub(crate) struct SystemAllocator;

// SAFETY: allocations come from libc and are released through the matching
// libc free. posix_memalign honors Rust alignments larger than malloc's.
unsafe impl GlobalAlloc for SystemAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut result = ptr::null_mut();
        let alignment = layout.align().max(core::mem::size_of::<*mut u8>());
        // SAFETY: alignment is a power of two and a multiple of pointer size.
        if unsafe { libc::posix_memalign(&mut result, alignment, layout.size()) } != 0 {
            ptr::null_mut()
        } else {
            result.cast()
        }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, _: Layout) {
        // SAFETY: GlobalAlloc supplies a live allocation from this allocator.
        unsafe { libc::free(ptr.cast()) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocations_and_growth_preserve_alignment_and_contents() {
        for alignment in [1, 8, 64, 4096] {
            let layout = Layout::from_size_align(17, alignment).unwrap();
            // SAFETY: layouts are valid and every allocation is checked before
            // access; deallocation uses the updated layout following realloc.
            unsafe {
                let ptr = SystemAllocator.alloc(layout);
                assert!(!ptr.is_null());
                assert_eq!(ptr as usize % alignment, 0);
                ptr.write_bytes(0x5a, layout.size());
                let grown = SystemAllocator.realloc(ptr, layout, 129);
                assert!(!grown.is_null());
                assert_eq!(grown as usize % alignment, 0);
                assert_eq!(core::slice::from_raw_parts(grown, 17), &[0x5a; 17]);
                SystemAllocator.dealloc(grown, Layout::from_size_align(129, alignment).unwrap());
            }
        }
    }
}
