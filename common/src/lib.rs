pub mod grammar;

use std::hash::BuildHasherDefault;
use ahash::AHasher;

// define some data structures that will be used in other crates, so that they don't need to import them
pub type IndexMap<K, V> = indexmap::IndexMap<K, V, BuildHasherDefault<AHasher>>;
pub type IndexSet<K> = indexmap::IndexSet<K, BuildHasherDefault<AHasher>>;

pub use re2dfa::{self, HashMap, HashSet, print::fn2display};
pub use smallvec::{smallvec, SmallVec};
pub use bitset;

// parse a "lhs -> rhs1 rhs2 ..." string
pub fn parse_arrow_prod(s: &str) -> Option<(&str, Vec<&str>)> {
  let mut sp = s.split_whitespace();
  let lhs = sp.next()?;
  match sp.next() { Some("->") => {} _ => return None };
  Some((lhs, sp.collect()))
}