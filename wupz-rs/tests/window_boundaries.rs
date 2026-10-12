use wupz_core::incremental::windows_touched;
use wupz_core::plan::{LayoutPlan, Section, SectionId};

fn windows(addr: u64, len: u64, window_size: u64) -> Vec<u64> {
    let plan = LayoutPlan {
        sections: vec![Section {
            id: SectionId(7),
            name: ".text".into(),
            addr,
            len,
            hash: 0,
        }],
        blocks: vec![],
        window_size,
    };
    windows_touched(&plan, &[SectionId(7), SectionId(7), SectionId(99)])
}

#[test]
fn empty_sections_touch_no_windows() {
    for addr in [0, 4096, 1 << 44, u64::MAX] {
        assert!(windows(addr, 0, 4096).is_empty(), "address {addr}");
    }
}

#[test]
fn section_ending_on_boundary_does_not_touch_next_window() {
    assert_eq!(windows(4095, 1, 4096), vec![0]);
    assert_eq!(windows(4095, 2, 4096), vec![0, 1]);
    assert_eq!(windows(4096, 4096, 4096), vec![1]);
}

#[test]
fn span_crossing_u32_window_limit_keeps_both_windows() {
    assert_eq!(
        windows((1 << 44) - 1, 2, 4096),
        vec![u32::MAX as u64, u32::MAX as u64 + 1]
    );
}

#[test]
fn high_address_does_not_alias_window_zero() {
    assert_eq!(windows(1 << 44, 1, 4096), vec![1 << 32]);
    assert_eq!(windows(u64::MAX, 1, 1), vec![u64::MAX]);
}

#[test]
fn end_address_saturates_without_wrapping() {
    assert_eq!(windows(u64::MAX - 1, 4, 1), vec![u64::MAX - 1, u64::MAX]);
}

#[test]
fn zero_window_size_is_treated_as_one() {
    assert_eq!(windows(7, 2, 0), vec![7, 8]);
}
