pub mod grammar;

use hashbrown::hash_map::DefaultHashBuilder;

// define some data structures that will be used in other crates, so that they don't need to import them
pub type IndexMap<K, V> = indexmap::IndexMap<K, V, DefaultHashBuilder>;
pub type IndexSet<K> = indexmap::IndexSet<K, DefaultHashBuilder>;

pub use hashbrown::{HashMap, HashSet};
pub use smallvec::{smallvec, SmallVec};
pub use bitset::{BitSet, traits::ToUsize};

// parse a "lhs -> rhs1 rhs2 ..." string
pub fn parse_arrow_prod(s: &str) -> Option<(&str, Vec<&str>)> {
  let mut sp = s.split_whitespace();
  let lhs = sp.next()?;
  match sp.next() { Some("->") => {} _ => return None };
  Some((lhs, sp.collect()))
}