//! The incremental rebuild rule — which sections changed, which blocks must be
//! re-packed, and which address windows a change perturbs.
//!
//! This is P4's logic, landed early because it is pure and testable without a
//! linker: given two plans, decide the minimum work.

use crate::plan::{LayoutPlan, SectionId};
use std::collections::BTreeSet;

/// Sections whose resolved bytes differ between `old` and `new`.
///
/// A section is changed when it is new, when its content hash moved, or when its
/// size moved. A section that *disappeared* also counts — dropping bytes is a
/// change too.
pub fn changed_sections(old: &LayoutPlan, new: &LayoutPlan) -> Vec<SectionId> {
    let mut changed = BTreeSet::new();
    for n in &new.sections {
        match old.section(n.id) {
            Some(o) if o.hash == n.hash && o.len == n.len => {}
            _ => {
                changed.insert(n.id);
            }
        }
    }
    for o in &old.sections {
        if new.section(o.id).is_none() {
            changed.insert(o.id);
        }
    }
    changed.into_iter().collect()
}

/// Blocks whose covering set intersects the changed set. Exactly these must be
/// re-packed; every other block is reused verbatim.
pub fn blocks_to_repack(plan: &LayoutPlan, changed: &[SectionId]) -> Vec<u32> {
    let set: BTreeSet<SectionId> = changed.iter().copied().collect();
    plan.blocks
        .iter()
        .filter(|b| b.covers.iter().any(|s| set.contains(s)))
        .map(|b| b.index)
        .collect()
}

/// The address-independent windows a change perturbs. A section that spans a
/// window boundary touches every window it overlaps — that is the whole point of
/// fixed windows: the blast radius stays local.
pub fn windows_touched(plan: &LayoutPlan, changed: &[SectionId]) -> Vec<u32> {
    let mut windows = BTreeSet::new();
    for id in changed {
        if let Some(s) = plan.section(*id) {
            let first = plan.window_of(s.addr);
            let last_addr = s.addr.saturating_add(s.len.saturating_sub(1));
            let last = plan.window_of(last_addr);
            for w in first..=last {
                windows.insert(w);
            }
        }
    }
    windows.into_iter().collect()
}

/// Fraction of blocks reused (not re-packed) — `1.0` means nothing moved.
pub fn reuse_ratio(plan: &LayoutPlan, changed: &[SectionId]) -> f64 {
    if plan.blocks.is_empty() {
        return 1.0;
    }
    let repack = blocks_to_repack(plan, changed).len();
    1.0 - (repack as f64 / plan.blocks.len() as f64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plan::{Block, Section};

    fn plan(hashes: &[(u32, u64, u64)]) -> LayoutPlan {
        // (id, hash, addr)
        let sections = hashes
            .iter()
            .map(|(id, hash, addr)| Section {
                id: SectionId(*id),
                name: format!(".s{id}"),
                addr: *addr,
                len: 16,
                hash: *hash,
            })
            .collect();
        let blocks = hashes
            .iter()
            .enumerate()
            .map(|(i, (id, _, _))| Block {
                index: i as u32,
                covers: vec![SectionId(*id)],
            })
            .collect();
        LayoutPlan {
            sections,
            blocks,
            window_size: 4096,
        }
    }

    #[test]
    fn only_the_changed_section_is_flagged() {
        let old = plan(&[(0, 0xaa, 0x1000), (1, 0xbb, 0x2000)]);
        let new = plan(&[(0, 0xaa, 0x1000), (1, 0xcc, 0x2000)]);
        assert_eq!(changed_sections(&old, &new), vec![SectionId(1)]);
    }

    #[test]
    fn only_blocks_covering_changed_sections_repack() {
        let old = plan(&[(0, 0xaa, 0x1000), (1, 0xbb, 0x2000), (2, 0xcc, 0x3000)]);
        let new = plan(&[(0, 0xaa, 0x1000), (1, 0xcc, 0x2000), (2, 0xcc, 0x3000)]);
        let changed = changed_sections(&old, &new);
        assert_eq!(blocks_to_repack(&new, &changed), vec![1]);
        assert!((reuse_ratio(&new, &changed) - (2.0 / 3.0)).abs() < 1e-9);
    }

    #[test]
    fn nothing_changed_reuses_everything() {
        let old = plan(&[(0, 0xaa, 0x1000), (1, 0xbb, 0x2000)]);
        let new = old.clone();
        let changed = changed_sections(&old, &new);
        assert!(changed.is_empty());
        assert_eq!(reuse_ratio(&new, &changed), 1.0);
    }

    #[test]
    fn a_new_section_counts_as_changed() {
        let old = plan(&[(0, 0xaa, 0x1000)]);
        let new = plan(&[(0, 0xaa, 0x1000), (1, 0xdd, 0x2000)]);
        assert_eq!(changed_sections(&old, &new), vec![SectionId(1)]);
    }

    #[test]
    fn a_removed_section_counts_as_changed() {
        let old = plan(&[(0, 0xaa, 0x1000), (1, 0xbb, 0x2000)]);
        let new = plan(&[(0, 0xaa, 0x1000)]);
        assert_eq!(changed_sections(&old, &new), vec![SectionId(1)]);
    }

    #[test]
    fn windows_are_address_independent() {
        let p = plan(&[(0, 0xaa, 0x1000), (1, 0xbb, 0x9000)]);
        // one changed section → exactly its own window
        assert_eq!(windows_touched(&p, &[SectionId(0)]), vec![1]);
        assert_eq!(windows_touched(&p, &[SectionId(1)]), vec![9]);
    }
}
