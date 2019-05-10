use std::fmt;
use std::iter;

#[derive(Clone, Hash, Eq, PartialEq)]
pub struct BitSet {
  inner: Box<[u64]>
}

impl BitSet {
  #[inline]
  pub fn new(n: u32) -> BitSet {
    let n = (n >> 6) + (((n & 63) != 0) as u32);
    BitSet { inner: iter::repeat(0).take(n as usize).collect() }
  }

  // I just need this bool
  // but it seems that there is not a library that provides it
  #[inline]
  pub fn or(&mut self, other: &BitSet) -> bool {
    let mut changed = false;
    for (x, y) in self.inner.iter_mut().zip(other.inner.iter()) {
      let ox = *x;
      *x |= *y;
      changed |= *x != ox;
    }
    changed
  }

  // it is possible that the n is out of range that `new` specified
  // no check, for my convenience
  #[inline]
  pub fn test(&self, n: u32) -> bool {
    return ((self.inner[(n >> 6) as usize] >> (n & 63)) & 1) != 0;
  }

  #[inline]
  pub fn set(&mut self, n: u32) {
    self.inner[(n >> 6) as usize] |= (1 as u64) << (n & 63);
  }

  #[inline]
  pub fn clear(&mut self, n: u32) {
    self.inner[(n >> 6) as usize] &= !((1 as u64) << (n & 63));
  }

  #[inline]
  pub fn inner_len(&self) -> u32 {
    self.inner.len() as u32
  }

  #[inline]
  pub fn as_ptr(&self) -> *const u64 {
    self.inner.as_ptr()
  }

  #[inline]
  pub fn as_mut_ptr(&mut self) -> *mut u64 {
    self.inner.as_mut_ptr()
  }

  #[inline]
  pub unsafe fn or_raw(mut x: *mut u64, mut y: *const u64, len: u32) -> bool {
    let mut changed = false;
    for _ in 0..len {
      let ox = *x;
      *x |= *y;
      changed |= *x != ox;
      x = x.add(1);
      y = y.add(1);
    }
    changed
  }
}

impl fmt::Debug for BitSet {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    let mut l = f.debug_list();
    for i in 0..self.inner.len() as u32 * 64 {
      if self.test(i) {
        l.entry(&i);
      }
    }
    l.finish();
    Ok(())
  }
}