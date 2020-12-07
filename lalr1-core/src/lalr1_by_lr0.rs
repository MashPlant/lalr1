// "Compilers: Principles, Techniques and Tools" Algorithm 4.63
use crate::{lr1::Lr1Ctx, Lr1Item, Lr0Fsm, Lr0Node, Lr1Node, Lr1Fsm};
use common::{grammar::{Grammar, EOF_IDX, ERR_IDX}, *};
use unchecked_unwrap::UncheckedUnwrap;

pub fn work<'a>(lr0: Lr0Fsm<'a>, g: &'a Grammar<'a>) -> Lr1Fsm<'a> {
  let token_num = g.token_num();
  let elem_len = bitset::bslen(token_num);
  let mut ctx = Lr1Ctx::new(g);
  let mut lookahead = lr0.iter()
    .map(|Lr0Node { closure, .. }| vec![0; elem_len * closure.len()].into_boxed_slice()).collect::<Vec<_>>();
  let mut prop = Vec::new();
  let start_prod = g.start().1.id;

  for (i, item) in lr0[0].closure.iter().enumerate() {
    if item.prod_id == start_prod {
      bitset::bs(&mut lookahead[0][i * elem_len..(i + 1) * elem_len]).set(EOF_IDX);
      break;
    }
  }

  // use ERR_IDX as the special token
  for (i, Lr0Node { closure: state, link }) in lr0.iter().enumerate() {
    for (item_id, &item) in state.iter().enumerate() {
      // only consider lr0 core item
      if item.prod_id == start_prod || item.dot != 0 {
        unsafe {
          let cl = ctx.closure({
                                 let lookahead = bitset::bsmake(g.token_num());
                                 bitset::ubs(lookahead.as_ref()).set(ERR_IDX);
                                 let mut init = HashMap::default();
                                 init.insert(item, lookahead);
                                 init
                               }, g);
          let from = lookahead.get_unchecked(i).as_ptr().add(item_id * elem_len);
          for Lr1Item { lr0: cl_item, lookahead: cl_lookahead } in &cl {
            if let Some(ch) = cl_item.prod.get(cl_item.dot as usize) {
              let goto_state = *link.get(ch).unchecked_unwrap() as usize;
              let goto_item_id = cl_item.unique_id() + 1; // dot + 1
              let goto_item_idx = lr0.get_unchecked(goto_state as usize).closure.iter()
                .position(|item| item.unique_id() == goto_item_id).unchecked_unwrap();
              let goto_lookahead = lookahead.get_unchecked(goto_state as usize).as_ptr().add(goto_item_idx * elem_len);
              bitset::ubs1(goto_lookahead).or(cl_lookahead.as_ptr(), elem_len);
              if bitset::ubs(cl_lookahead.as_ref()).get(ERR_IDX) {
                prop.push((from, goto_lookahead));
              }
            }
          }
        }
      }
    }
  }

  loop {
    let mut changed = false;
    for &(from, to) in &prop {
      unsafe { changed |= bitset::ubs1(to).or(from, elem_len); }
    }
    if !changed { break; }
  }

  let mut result = Vec::with_capacity(lr0.len());
  for (i, Lr0Node { closure, link }) in lr0.into_iter().enumerate() {
    let mut lr1_closure = HashMap::with_capacity_and_hasher(closure.len(), Default::default());
    unsafe {
      let l = lookahead.get_unchecked(i).as_ptr();
      for (i, closure) in closure.into_iter().enumerate() {
        let l = l.add(i * elem_len);
        bitset::ubs1(l).del(ERR_IDX);
        lr1_closure.insert(closure, Box::from(std::slice::from_raw_parts(l, elem_len)));
      }
    }
    result.push(Lr1Node { closure: ctx.closure(lr1_closure, g), link });
  }
  result
}