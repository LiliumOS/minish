use core::{
    alloc::{Allocator, Layout},
    ffi::{c_long, c_void},
    ptr::NonNull,
};

use alloc::alloc::AllocError;
use lilium_sys::{
    result::Error,
    sync::mutex::RawMutex,
    sys::{
        kstr::KCSlice,
        process::{self as sys, CreateMapping, RemoveMapping, ResizeMapping},
    },
};
use talc::{OomHandler, Span, Talc, Talck};

use crate::{eprintln, println};

#[derive(Copy, Clone)]
pub struct CreateMappingAlloc;

unsafe impl Allocator for CreateMappingAlloc {
    fn allocate(
        &self,
        layout: core::alloc::Layout,
    ) -> Result<core::ptr::NonNull<[u8]>, alloc::alloc::AllocError> {
        self.allocate_zeroed(layout)
    }

    fn allocate_zeroed(
        &self,
        layout: core::alloc::Layout,
    ) -> Result<core::ptr::NonNull<[u8]>, alloc::alloc::AllocError> {
        if layout.size() == 0 {
            return Ok(NonNull::slice_from_raw_parts(layout.dangling(), 0));
        }

        if layout.align() > 4096 {
            return Err(AllocError);
        }
        let size = layout.size().next_multiple_of(4096) / 4096;

        let mut addr = core::ptr::null_mut();
        Error::from_code(unsafe {
            CreateMapping(
                &mut addr,
                size as c_long,
                sys::MAP_ATTR_READ | sys::MAP_ATTR_WRITE | sys::MAP_ATTR_PROC_PRIVATE,
                sys::MAP_KIND_NORMAL,
                &KCSlice::empty(),
            )
        })
        .map_err(|_| AllocError)?;

        Ok(NonNull::slice_from_raw_parts(
            NonNull::new(addr.cast::<u8>()).expect("Expected a non-null pointer"),
            size * 4096,
        ))
    }

    unsafe fn deallocate(&self, ptr: core::ptr::NonNull<u8>, layout: core::alloc::Layout) {
        let size = layout.size().next_multiple_of(4096) / 4096;

        unsafe {
            RemoveMapping(ptr.as_ptr().cast(), size as c_long);
        }
    }
}

#[repr(C, align(4096))]
struct Page([u8; 4096]);

impl OomHandler for CreateMappingAlloc {
    fn handle_oom(talc: &mut talc::Talc<Self>, layout: core::alloc::Layout) -> Result<(), ()> {
        let mut alloc_layout = Layout::new::<[Page; usize::BITS as usize]>();
        if alloc_layout.size() < layout.size() {
            alloc_layout = layout
                .align_to(4096)
                .unwrap()
                .extend(Layout::new::<usize>())
                .unwrap()
                .0
                .pad_to_align();
        }
        eprintln!("Expanding map by alloc_layout={alloc_layout:?} (requested layout={layout:?})");
        let block = talc
            .oom_handler
            .allocate_zeroed(alloc_layout)
            .map_err(|_| ())?;

        eprintln!("Allocated: {block:p}");

        let span = unsafe { talc.claim(Span::from_slice(block.as_ptr()))? };

        eprintln!("Claimed {span:?}");
        Ok(())
    }
}

#[global_allocator]
static GLOBAL: Talck<RawMutex, CreateMappingAlloc> = Talck::new(Talc::new(CreateMappingAlloc));
