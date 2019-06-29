use grammar_config::AbstractGrammar;
use std::collections::{HashMap, HashSet};
use std::collections::vec_deque::VecDeque;
use std::hash::{Hash, Hasher};
use std::cmp::Ordering;

#[derive(Clone, Copy, Debug)]
pub struct LRItem<'a> {
  pub prod: &'a [u32],
  pub prod_id: u32,
  // prod[dot] = the token after dot
  pub dot: u32,
  // look_ahead is the map value in LRState
}

impl LRItem<'_> {
  pub fn unique_id(&self) -> u64 {
    ((self.prod_id as u64) << 32) | (self.dot as u64)
  }
}

impl Hash for LRItem<'_> {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.unique_id().hash(state);
  }
}

impl PartialEq for LRItem<'_> {
  fn eq(&self, other: &LRItem) -> bool {
    self.unique_id() == other.unique_id()
  }
}

impl Eq for LRItem<'_> {}

impl PartialOrd for LRItem<'_> {
  fn partial_cmp(&self, other: &LRItem) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for LRItem<'_> {
  fn cmp(&self, other: &Self) -> Ordering {
    self.unique_id().cmp(&other.unique_id())
  }
}

struct LRCtx {
  token_num: u32,
  nt_num: u32,
}

impl LRCtx {
  fn go<'a>(&self, items: &Vec<LRItem<'a>>, mov: u32, g: &'a impl AbstractGrammar<'a>) -> Vec<LRItem<'a>> {
    let mut new_items = HashSet::new();
    for item in items {
      if item.dot as usize >= item.prod.len() { // dot is after the last ch
        continue;
      }
      if item.prod[item.dot as usize] == mov {
        new_items.insert(LRItem { prod: item.prod, prod_id: item.prod_id, dot: item.dot + 1 });
      }
    }
    self.closure(new_items, g)
  }

  fn closure<'a>(&self, mut items: HashSet<LRItem<'a>>, g: &'a impl AbstractGrammar<'a>) -> Vec<LRItem<'a>> {
    let mut q = items.clone().into_iter().collect::<VecDeque<_>>();
    while let Some(item) = q.pop_front() {
      if item.dot as usize >= item.prod.len() { // dot is after the last ch
        continue;
      }
      let b = item.prod[item.dot as usize];
      if b < self.nt_num {
        for new_prod in g.get_prod(b) {
          let new_item = LRItem { prod: new_prod.0.as_ref(), prod_id: new_prod.1, dot: 0 };
          if items.insert(new_item) {
            q.push_back(new_item);
          }
        }
      }
    }
    let mut items = items.into_iter().collect::<Vec<_>>();
    // why sort_unstable_by_key(|x| &x.0) won't work here?
    items.sort_unstable_by(|l, r| l.cmp(&r));
    items
  }
}

pub fn work<'a>(g: &'a impl AbstractGrammar<'a>) -> Vec<(Vec<LRItem<'a>>, HashMap<u32, u32>)> {
  let ctx = LRCtx { token_num: g.token_num(), nt_num: g.nt_num() };
  let mut ss = HashMap::new();
  let init = ctx.closure({
                           let start = g.start().1;
                           let mut init = HashSet::new();
                           init.insert(LRItem { prod: start.0.as_ref(), prod_id: start.1, dot: 0 });
                           init
                         }, g);
  ss.insert(init.clone(), 0);
  let mut q = VecDeque::new();
  let mut result = Vec::new();
  q.push_back(init);
  while let Some(cur) = q.pop_front() {
    let mut link = HashMap::new();
    for mov in 0..ctx.token_num {
      let ns = ctx.go(&cur, mov, g);
      if !ns.is_empty() {
        let id = match ss.get(&ns) {
          None => {
            let id = ss.len() as u32;
            ss.insert(ns.clone(), id);
            q.push_back(ns);
            id
          }
          Some(id) => *id,
        };
        link.insert(mov, id);
      }
    }
    result.push((cur, link));
  }
  result
}