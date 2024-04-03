//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec;
use core::ops::Range;
use core::ptr;
use core::slice;

use sel4_capdl_initializer_core::{Initializer, InitializerBuffers, PerObjectBuffer};
use sel4_capdl_initializer_types::{
    IndirectDeflatedBytesContent, IndirectEmbeddedFrame, IndirectObjectName, SpecWithIndirection,
    SpecWithSources,
};
use sel4_dlmalloc::StaticHeapBounds;
use sel4_logging::{LevelFilter, Logger, LoggerBuilder};
use sel4_root_task::root_task;

sel4::config::sel4_cfg_if! {
    if #[sel4_cfg(PRINTING)] {
        pub use sel4::debug_print;
    } else {
        /// No-op for this configuration.
        #[macro_export]
        macro_rules! debug_print {
            ($($arg:tt)*) => {};
        }
    }
}

const LOG_LEVEL: LevelFilter = {
    // LevelFilter::Trace
    // LevelFilter::Debug
    LevelFilter::Info
};

static LOGGER: Logger = LoggerBuilder::const_default()
    .level_filter(LOG_LEVEL)
    .filter(|meta| meta.target() == "sel4_capdl_initializer_core")
    .write(|_s| {
        debug_print!("{}", _s);
    })
    .build();

#[root_task(stack_size = 0x10000)]
fn main(bootinfo: &sel4::BootInfoPtr) -> ! {
    LOGGER.set().unwrap();
    let spec_with_sources = get_spec_with_sources();
    let mut buffers = InitializerBuffers::new(vec![
        PerObjectBuffer::const_default();
        spec_with_sources.spec.objects.len()
    ]);
    Initializer::initialize(
        bootinfo,
        user_image_bounds(),
        &spec_with_sources,
        &mut buffers,
    )
}

#[no_mangle]
#[link_section = ".data"]
static mut sel4_capdl_initializer_serialized_spec_start: *mut u8 = ptr::null_mut();

#[no_mangle]
#[link_section = ".data"]
static mut sel4_capdl_initializer_serialized_spec_size: usize = 0;

#[no_mangle]
#[link_section = ".data"]
static mut sel4_capdl_initializer_heap_start: *mut u8 = ptr::null_mut();

#[no_mangle]
#[link_section = ".data"]
static mut sel4_capdl_initializer_heap_size: usize = 0;

#[no_mangle]
#[link_section = ".data"]
static mut sel4_capdl_initializer_image_start: *mut u8 = ptr::null_mut();

#[no_mangle]
#[link_section = ".data"]
static mut sel4_capdl_initializer_image_end: *mut u8 = ptr::null_mut();

fn get_spec_with_sources<'a>() -> SpecWithSources<
    'a,
    Option<IndirectObjectName>,
    IndirectDeflatedBytesContent,
    IndirectEmbeddedFrame,
> {
    let blob = unsafe {
        slice::from_raw_parts(
            sel4_capdl_initializer_serialized_spec_start,
            sel4_capdl_initializer_serialized_spec_size,
        )
    };
    let (spec, source) = postcard::take_from_bytes::<SpecWithIndirection>(blob).unwrap();
    SpecWithSources {
        spec,
        object_name_source: source,
        content_source: source,
        embedded_frame_source: source,
    }
}

fn user_image_bounds() -> Range<usize> {
    unsafe {
        (sel4_capdl_initializer_image_start as usize)..(sel4_capdl_initializer_image_end as usize)
    }
}

fn static_heap_bounds() -> StaticHeapBounds {
    unsafe {
        StaticHeapBounds::new(
            sel4_capdl_initializer_heap_start,
            sel4_capdl_initializer_heap_size,
        )
    }
}

mod heap {
    use sel4_dlmalloc::{StaticDlmallocGlobalAlloc, StaticHeapBounds};
    use sel4_sync::{GenericRawMutex, PanickingMutexSyncOps};

    use super::static_heap_bounds;

    #[global_allocator]
    static GLOBAL_ALLOCATOR: StaticDlmallocGlobalAlloc<
        GenericRawMutex<PanickingMutexSyncOps>,
        fn() -> StaticHeapBounds,
    > = StaticDlmallocGlobalAlloc::new(
        GenericRawMutex::new(PanickingMutexSyncOps::new()),
        static_heap_bounds,
    );
}
