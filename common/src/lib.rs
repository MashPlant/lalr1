pub mod grammar;

// pub use re2dfa::{re2dfa, Dfa, Nfa};
pub use smallvec::{smallvec, SmallVec};
pub use tools::{*, fmt as fmt_};

pub use std::fmt::{Formatter, Debug, Display, Result as FmtResult};
pub use grammar::*;

// define some data structures that will be used in other crates, so that they don't need to import them
pub type IndexMap<K, V> = indexmap::IndexMap<K, V, AHashBuilder>;
pub type IndexSet<K> = indexmap::IndexSet<K, AHashBuilder>;

// parse a "lhs -> rhs1 rhs2 ..." string
pub fn parse_arrow_prod(s: &str) -> Option<(&str, Vec<&str>)> {
  let mut sp = s.split_whitespace();
  let lhs = sp.next()?;
  match sp.next() { Some("->") => {} _ => return None };
  Some((lhs, sp.collect()))
}