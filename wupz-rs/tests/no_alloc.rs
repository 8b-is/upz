//! The zero-alloc invariant, enforced.
//!
//! `docs/DESIGN.md` §5 claims packing allocates nothing after the layout is
//! sealed. This test binary installs a counting global allocator and asserts it:
//! the plan is built first (that allocates, and that is fine), the arena is
//! allocated once, and then `pack_into` must not touch the allocator at all.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

use wupz_core::arena::Arena;
use wupz_core::pack::{MAGIC, pack_into, plan_image_len};
use wupz_core::plan::{Block, LayoutPlan, Section, SectionId};

struct Counting;

static ALLOCS: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCS.fetch_add(1, Ordering::SeqCst);
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        ALLOCS.fetch_add(1, Ordering::SeqCst);
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static ALLOCATOR: Counting = Counting;

fn plan() -> LayoutPlan {
    LayoutPlan {
        sections: vec![
            Section { id: SectionId(0), name: ".text".into(), addr: 0x1000, len: 0x200, hash: 0xaa },
            Section { id: SectionId(1), name: ".rodata".into(), addr: 0x2000, len: 0x80, hash: 0xbb },
        ],
        blocks: vec![
            Block { index: 0, covers: vec![SectionId(0)] },
            Block { index: 1, covers: vec![SectionId(1)] },
        ],
        window_size: 4096,
    }
}

#[test]
fn pack_after_arena_seal_allocates_nothing() {
    // Setup — allocation here is expected and irrelevant to the invariant.
    let plan = plan();
    let mut arena = Arena::with_capacity(plan_image_len(&plan));

    let before = ALLOCS.load(Ordering::SeqCst);
    let image = pack_into(&plan, &mut arena).expect("arena sized from plan_image_len");
    let after = ALLOCS.load(Ordering::SeqCst);

    assert_eq!(
        before, after,
        "pack_into must not allocate after the arena is sized (allocated {} times)",
        after - before
    );
    assert_eq!(&image[..4], &MAGIC);

    // Reading the sealed arena is likewise allocation-free.
    let sealed = arena.seal();
    let before = ALLOCS.load(Ordering::SeqCst);
    let len = sealed.as_slice().len();
    let after = ALLOCS.load(Ordering::SeqCst);
    assert_eq!(before, after, "reading a sealed arena must not allocate");
    assert_eq!(len, plan_image_len(&plan));
}
