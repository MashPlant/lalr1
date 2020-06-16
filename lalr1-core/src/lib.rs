use std::{hash::{Hash, Hasher}, cmp::Ordering, ops::Deref};
use common::{SmallVec, BitSet, HashMap};

pub mod lr1;
pub mod lr0;
pub mod lalr1_by_lr0;
pub mod mk_table;

#[derive(Clone, Copy)]
pub struct Lr0Item<'a> {
  pub prod: &'a [u32],
  pub prod_id: u32,
  // prod[dot] = the token after dot
  pub dot: u32,
}

#[derive(Clone, Hash, Eq, PartialEq)]
pub struct Lr1Item<'a> {
  pub lr0: Lr0Item<'a>,
  pub lookahead: BitSet,
}

pub type Lr0Closure<'a> = Vec<Lr0Item<'a>>;
pub type Lr1Closure<'a> = Vec<Lr1Item<'a>>;

pub type Link = HashMap<u32, u32>;

pub struct Lr0Node<'a> {
  pub closure: Lr0Closure<'a>,
  pub link: Link,
}

// originally the `link` field type is a generic parameter L: Borrow<Link>
// because the Lr1Node generated from `lalr1_by_lr0` is a borrowed ref of the `link` in Lr0Node
// but later I decided to give up this design, for the **simplicity** of my code,
// although it can indeed eliminate the unnecessary clone
pub struct Lr1Node<'a> {
  pub closure: Lr1Closure<'a>,
  pub link: Link,
}

pub type Lr0Fsm<'a> = Vec<Lr0Node<'a>>;
pub type Lr1Fsm<'a> = Vec<Lr1Node<'a>>;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Act {
  Acc,
  Shift(u32),
  Reduce(u32),
}

pub type Acts = SmallVec<[Act; 2]>;

#[derive(Clone)]
pub struct TableEntry<'a> {
  // actually we are now only using the Lr0Closure part of it, but these Lr1Closure can't be directly converted Lr0Closure
  // so now just leave it as Lr1Closure; if we need Lr0Closure in the future, we can write a trait to extract the common behaviour of them
  pub closure: &'a Lr1Closure<'a>,
  pub act: HashMap<u32, Acts>,
  pub goto: HashMap<u32, u32>,
}

pub type Table<'a> = Vec<TableEntry<'a>>;

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

impl Conflict {
  pub fn is_many(&self) -> bool {
    match self.kind { ConflictKind::Many(_) => true, _ => false }
  }
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

impl<'a> Deref for Lr1Item<'a> {
  type Target = Lr0Item<'a>;
  fn deref(&self) -> &Self::Target { &self.lr0 }
}