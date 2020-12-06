use crate::{Lr0Item, Lr0Fsm, Lr0Node};
use common::{grammar::Grammar, *};
use std::collections::VecDeque;

fn go<'a>(items: &Vec<Lr0Item<'a>>, mov: usize, g: &'a Grammar<'a>) -> Vec<Lr0Item<'a>> {
  let mut new_items = HashSet::default();
  for item in items {
    if item.dot as usize >= item.prod.len() { // dot is after the last ch
      continue;
    }
    if item.prod[item.dot as usize] == mov as u32 {
      new_items.insert(Lr0Item { prod: item.prod, prod_id: item.prod_id, dot: item.dot + 1 });
    }
  }
  closure(new_items, g)
}

fn closure<'a>(mut items: HashSet<Lr0Item<'a>>, g: &'a Grammar<'a>) -> Vec<Lr0Item<'a>> {
  let mut q = items.clone().into_iter().collect::<VecDeque<_>>();
  while let Some(item) = q.pop_front() {
    if item.dot as usize >= item.prod.len() { // dot is after the last ch
      continue;
    }
    let ch = item.prod[item.dot as usize];
    if let Some(ch) = g.as_nt(ch) {
      for new_prod in g.get_prod(ch) {
        let new_item = Lr0Item { prod: &new_prod.rhs, prod_id: new_prod.id, dot: 0 };
        if items.insert(new_item) {
          q.push_back(new_item);
        }
      }
    }
  }
  let mut items = items.into_iter().collect::<Vec<_>>();
  items.sort_unstable();
  items
}

pub fn work<'a>(g: &'a Grammar) -> Lr0Fsm<'a> {
  let token_num = g.token_num();
  let mut ss = HashMap::default();
  let init = closure({
                       let start = g.start().1;
                       let mut init = HashSet::default();
                       init.insert(Lr0Item { prod: &start.rhs, prod_id: start.id, dot: 0 });
                       init
                     }, g);
  ss.insert(init.clone(), 0);
  let mut q = VecDeque::new();
  let mut result = Vec::new();
  q.push_back(init);
  while let Some(cur) = q.pop_front() {
    let mut link = HashMap::default();
    for mov in 0..token_num {
      let ns = go(&cur, mov, g);
      if !ns.is_empty() {
        let new_id = ss.len() as u32;
        let id = *ss.entry(ns.clone()).or_insert_with(|| (q.push_back(ns), new_id).1);
        link.insert(mov as u32, id);
      }
    }
    result.push(Lr0Node { closure: cur, link });
  }
  result
}