//! The plan image — the deterministic bytes the linker hands the packer.
//!
//! This is the P1 artifact: the layout plan serialized into a fixed, reproducible
//! byte layout. It is **not** the compressed output yet (that is P3); it is the
//! inter-pass contract, written into a pre-sized arena so the hot path allocates
//! nothing.
//!
//! Wire layout (little-endian):
//!
//! ```text
//! magic         4   "WUPZ"
//! version       1   = PLAN_VERSION
//! section_count 4
//! block_count   4
//! sections[]    id u32 · addr u64 · len u64 · hash u64 · name_len u16 · name[..]
//! blocks[]      index u32 · cover_count u32 · covers u32..
//! ```

use crate::arena::Arena;
use crate::plan::LayoutPlan;

pub const MAGIC: [u8; 4] = *b"WUPZ";
pub const PLAN_VERSION: u8 = 1;

const HEADER_LEN: usize = 4 + 1 + 4 + 4;
const SECTION_FIXED: usize = 4 + 8 + 8 + 8 + 2;
const BLOCK_FIXED: usize = 4 + 4;

/// Exact byte length of the plan image — no guessing, no growth.
pub fn plan_image_len(plan: &LayoutPlan) -> usize {
    let sections: usize = plan
        .sections
        .iter()
        .map(|s| SECTION_FIXED + s.name.len())
        .sum();
    let blocks: usize = plan
        .blocks
        .iter()
        .map(|b| BLOCK_FIXED + b.covers.len() * 4)
        .sum();
    HEADER_LEN + sections + blocks
}

fn put_u8(out: &mut [u8], at: &mut usize, v: u8) {
    out[*at] = v;
    *at += 1;
}

fn put_u16(out: &mut [u8], at: &mut usize, v: u16) {
    out[*at..*at + 2].copy_from_slice(&v.to_le_bytes());
    *at += 2;
}

fn put_u32(out: &mut [u8], at: &mut usize, v: u32) {
    out[*at..*at + 4].copy_from_slice(&v.to_le_bytes());
    *at += 4;
}

fn put_u64(out: &mut [u8], at: &mut usize, v: u64) {
    out[*at..*at + 8].copy_from_slice(&v.to_le_bytes());
    *at += 8;
}

fn put_bytes(out: &mut [u8], at: &mut usize, v: &[u8]) {
    out[*at..*at + v.len()].copy_from_slice(v);
    *at += v.len();
}

/// Write the plan image into `arena`. Returns `None` if the arena is too small
/// (the arena never grows — a silent grow would be an allocation).
pub fn pack_into<'a>(plan: &LayoutPlan, arena: &'a mut Arena) -> Option<&'a [u8]> {
    let len = plan_image_len(plan);
    let out = arena.alloc(len)?;
    let mut at = 0usize;

    put_bytes(out, &mut at, &MAGIC);
    put_u8(out, &mut at, PLAN_VERSION);
    put_u32(out, &mut at, plan.sections.len() as u32);
    put_u32(out, &mut at, plan.blocks.len() as u32);

    for s in &plan.sections {
        put_u32(out, &mut at, s.id.0);
        put_u64(out, &mut at, s.addr);
        put_u64(out, &mut at, s.len);
        put_u64(out, &mut at, s.hash);
        put_u16(out, &mut at, s.name.len() as u16);
        put_bytes(out, &mut at, s.name.as_bytes());
    }
    for b in &plan.blocks {
        put_u32(out, &mut at, b.index);
        put_u32(out, &mut at, b.covers.len() as u32);
        for c in &b.covers {
            put_u32(out, &mut at, c.0);
        }
    }

    debug_assert_eq!(at, len, "plan_image_len disagrees with the writer");
    Some(&out[..at])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plan::{Block, Section, SectionId};

    fn sample() -> LayoutPlan {
        LayoutPlan {
            sections: vec![
                Section { id: SectionId(0), name: ".text".into(), addr: 0x1000, len: 0x200, hash: 0xaa },
                Section { id: SectionId(1), name: ".data".into(), addr: 0x3000, len: 0x40, hash: 0xbb },
            ],
            blocks: vec![
                Block { index: 0, covers: vec![SectionId(0)] },
                Block { index: 1, covers: vec![SectionId(0), SectionId(1)] },
            ],
            window_size: 4096,
        }
    }

    #[test]
    fn image_has_the_magic_and_version() {
        let plan = sample();
        let mut arena = Arena::with_capacity(plan_image_len(&plan));
        let img = pack_into(&plan, &mut arena).unwrap();
        assert_eq!(&img[..4], b"WUPZ");
        assert_eq!(img[4], PLAN_VERSION);
    }

    #[test]
    fn image_is_deterministic() {
        let plan = sample();
        let mut a = Arena::with_capacity(1024);
        let mut b = Arena::with_capacity(1024);
        assert_eq!(pack_into(&plan, &mut a).unwrap(), pack_into(&plan, &mut b).unwrap());
    }

    #[test]
    fn length_is_exact() {
        let plan = sample();
        let expected = 4 + 1 + 4 + 4
            + (4 + 8 + 8 + 8 + 2 + 5)  // ".text"
            + (4 + 8 + 8 + 8 + 2 + 5)  // ".data"
            + (4 + 4 + 4)              // block 0
            + (4 + 4 + 4 + 4);         // block 1
        assert_eq!(plan_image_len(&plan), expected);
    }

    #[test]
    fn a_full_arena_refuses() {
        let plan = sample();
        let mut tiny = Arena::with_capacity(4);
        assert!(pack_into(&plan, &mut tiny).is_none());
    }
}
