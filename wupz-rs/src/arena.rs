//! The arena — one allocation up front, none on the hot path.
//!
//! `Arena::with_capacity` allocates once. After that, `alloc` hands out slices
//! and never touches the global allocator. The post-layout invariant
//! (`docs/DESIGN.md` §5) is: once the plan is sealed, packing must not allocate.
//! The integration test `tests/no_alloc.rs` enforces it with a counting
//! allocator.

/// A bump arena with a fixed buffer.
pub struct Arena {
    buf: Vec<u8>,
    used: usize,
}

impl Arena {
    /// Allocate the whole buffer once.
    pub fn with_capacity(bytes: usize) -> Self {
        Self { buf: vec![0u8; bytes], used: 0 }
    }

    pub fn capacity(&self) -> usize {
        self.buf.len()
    }

    pub fn used(&self) -> usize {
        self.used
    }

    pub fn remaining(&self) -> usize {
        self.buf.len() - self.used
    }

    /// Bump-allocate `len` zeroed bytes. Returns `None` when the arena is full —
    /// it never grows (growing would allocate).
    pub fn alloc(&mut self, len: usize) -> Option<&mut [u8]> {
        if len > self.remaining() {
            return None;
        }
        let start = self.used;
        self.used += len;
        Some(&mut self.buf[start..start + len])
    }

    /// Seal the arena: all allocation is done, only reads remain.
    pub fn seal(self) -> Seal {
        Seal { buf: self.buf, used: self.used }
    }
}

/// A sealed arena. Read-only by construction.
pub struct Seal {
    buf: Vec<u8>,
    used: usize,
}

impl Seal {
    pub fn as_slice(&self) -> &[u8] {
        &self.buf[..self.used]
    }

    pub fn len(&self) -> usize {
        self.used
    }

    pub fn is_empty(&self) -> bool {
        self.used == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alloc_returns_zeroed_slices() {
        let mut a = Arena::with_capacity(64);
        let s = a.alloc(8).unwrap();
        assert_eq!(s, &[0u8; 8]);
        s[0] = 7;
        assert_eq!(a.used(), 8);
    }

    #[test]
    fn alloc_refuses_to_grow() {
        let mut a = Arena::with_capacity(8);
        assert!(a.alloc(8).is_some());
        assert!(a.alloc(1).is_none());
    }

    #[test]
    fn slices_do_not_overlap() {
        let mut a = Arena::with_capacity(16);
        let first = a.alloc(8).unwrap().as_ptr() as usize;
        let second = a.alloc(8).unwrap().as_ptr() as usize;
        assert_eq!(second, first + 8);
    }

    #[test]
    fn seal_exposes_exactly_what_was_used() {
        let mut a = Arena::with_capacity(32);
        let _ = a.alloc(5);
        assert_eq!(a.seal().as_slice().len(), 5);
    }
}
