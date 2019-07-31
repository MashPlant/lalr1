use smallvec::SmallVec;
use std::{hash::{Hash, Hasher}, cmp::Ordering};
use bitset::BitSet;
use hashbrown::HashMap;

pub mod lr1;
pub mod lr0;
pub mod lalr1_by_lr0;
pub mod conflict;

pub use crate::conflict::*;

// define some common structs and types here

#[derive(Clone, Copy)]
pub struct Lr0Item<'a> {
  pub prod: &'a [u32],
  pub prod_id: u32,
  // prod[dot] = the token after dot
  pub dot: u32,
}

impl Lr0Item<'_> {
  pub fn unique_id(&self) -> u64 { ((self.prod_id as u64) << 32) | (self.dot as u64) }
}

impl Hash for Lr0Item<'_> {
  fn hash<H: Hasher>(&self, state: &mut H) { self.unique_id().hash(state); }
}

impl PartialEq for Lr0Item<'_> {
  fn eq(&self, other: &Lr0Item) -> bool { self.unique_id() == other.unique_id() }
}

impl Eq for Lr0Item<'_> {}

impl PartialOrd for Lr0Item<'_> {
  fn partial_cmp(&self, other: &Lr0Item) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl Ord for Lr0Item<'_> {
  fn cmp(&self, other: &Self) -> Ordering { self.unique_id().cmp(&other.unique_id()) }
}

pub struct Lr1Item<'a> {
  pub item: Lr0Item<'a>,
  pub lookahead: BitSet,
}

pub type Lr0Closure<'a> = Vec<Lr0Item<'a>>;
pub type Lr1Closure<'a> = Vec<Lr1Item<'a>>;

pub struct LrNode<'a> {
  pub items: Vec<Lr0Item<'a>>,
  pub link: HashMap<u32, u32>,
}

pub type LrFsm<'a> = Vec<LrNode<'a>>;

pub type Acts = SmallVec<[ParserAct; 2]>;