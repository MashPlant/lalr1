use smallvec::SmallVec;
use std::{hash::{Hash, Hasher}, cmp::Ordering};
use bitset::BitSet;
use hashbrown::HashMap;

pub mod lr1;
pub mod lr0;
pub mod lalr1_by_lr0;
pub mod conflict;

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
  pub items: Lr0Closure<'a>,
  pub link: HashMap<u32, u32>,
}

pub type LrFsm<'a> = Vec<LrNode<'a>>;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Act {
  Acc,
  Shift(u32),
  Reduce(u32),
}

pub type Acts = SmallVec<[Act; 2]>;

#[derive(Clone)]
pub struct TableEntry<'a> {
  pub items: &'a Lr0Closure<'a>,
  pub act: HashMap<u32, Acts>,
  pub goto: HashMap<u32, u32>,
}

pub type RawTable<'a> = Vec<TableEntry<'a>>;

pub enum ConflictKind {
  RR { r1: u32, r2: u32 },
  SR { s: u32, r: u32 },
  Many(Acts),
}

pub struct Conflict {
  pub kind: ConflictKind,
  pub state: u32,
  pub ch: u32,
}

pub struct ActTable<'a> {
  pub table: RawTable<'a>,
  pub conflict: Vec<Conflict>,
}