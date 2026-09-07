//! Process-memory telemetry for the F3 debug overlay (vanilla 1.16.5 right
//! column: "Mem: 24% 501/2048MB" / "Allocated: 41% 856MB").
//!
//! The JVM reads its own heap MBeans for those lines; a Rust binary has no
//! such runtime, so the engine installs a counting wrapper around the
//! system allocator. To keep the atomics off the hot small-object path
//! (world-gen/meshing allocate thousands of small Vecs per chunk), only
//! allocations of `TRACK_MIN` bytes and up are counted — the large buffers
//! (chunk meshes, region arenas, atlas copies) dominate live heap bytes,
//! so the number tracks real usage closely while staying cheap.
//!
//! `main.rs` / `lib.rs` (wasm) install `Counting` as the global allocator;
//! `game.rs` samples `allocated_bytes()` at the F3 cadence.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicU64, Ordering};

/// allocations this size and larger are tracked
pub const TRACK_MIN: usize = 4096;

static ALLOCATED: AtomicU64 = AtomicU64::new(0);

/// live large-allocation bytes (a good proxy for the engine's heap usage)
pub fn allocated_bytes() -> u64 {
    ALLOCATED.load(Ordering::Relaxed)
}

/// Counting wrapper over the system allocator.
pub struct Counting;

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let p = System.alloc(layout);
        if !p.is_null() && layout.size() >= TRACK_MIN {
            ALLOCATED.fetch_add(layout.size() as u64, Ordering::Relaxed);
        }
        p
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if layout.size() >= TRACK_MIN {
            ALLOCATED.fetch_sub(layout.size() as u64, Ordering::Relaxed);
        }
        System.dealloc(ptr, layout);
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let p = System.realloc(ptr, layout, new_size);
        if layout.size() >= TRACK_MIN {
            ALLOCATED.fetch_sub(layout.size() as u64, Ordering::Relaxed);
        }
        if !p.is_null() && new_size >= TRACK_MIN {
            ALLOCATED.fetch_add(new_size as u64, Ordering::Relaxed);
        }
        p
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let p = System.alloc_zeroed(layout);
        if !p.is_null() && layout.size() >= TRACK_MIN {
            ALLOCATED.fetch_add(layout.size() as u64, Ordering::Relaxed);
        }
        p
    }
}
