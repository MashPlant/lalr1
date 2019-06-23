extern crate re2dfa;
extern crate lr;
extern crate toml;
extern crate aho_corasick;

mod codegen;

use lr::RawGrammar;
use std::fs::read_to_string;
//use std::io::Write;
use crate::codegen::Codegen;

fn main() {
//  let s = read_to_string("test.g").unwrap();
//  let g = SimpleGrammar::from_text(&s);
//
//  let lr1 = lr1::work(&g);
//  let mut f = File::create("lr1.dot").unwrap();
//  f.write(g.print_lr1(&lr1).as_bytes()).unwrap();
//
//  let lr0 = lr0::work(&g);
//  let mut f = File::create("lr0.dot").unwrap();
//  f.write(g.print_lr0(&lr0).as_bytes()).unwrap();
//
//  let lalr1 = lalr1_by_lr0::lalr1_only(&lr0, &g);
//  let mut f = File::create("lalr1.dot").unwrap();
//  f.write(g.print_lr1(&lalr1).as_bytes()).unwrap();

  let s = read_to_string("examples/calc.toml").unwrap();
  let mut g: RawGrammar = toml::from_str(&s).unwrap();

  let g = g.extend_grammar().unwrap();
  let (dfa, ec) = re2dfa::re2dfa(g.raw.lexical.iter().map(|(re, _)| re)).unwrap();
  eprintln!("dfa has {} states", dfa.nodes.len());

  let a = lr::lr0::work(&g);
  let a = lr::lalr1_by_lr0::work(&a, &g);
  eprintln!("lalr1 fsm has {} states", a.action.len());

  use crate::codegen::RustCodegen;
  println!("{}", RustCodegen { log_token: false, log_reduce: true }.gen(&g, &a, &dfa, &ec));
  eprintln!("conflict: {:?}", a.conflict);
//  use std::env;

//  match env::args().nth(1) {
//    Some(ref one) if one.as_str() == "1" => {
//      let a = lr1::work(&g);
//      let a = lalr1_by_lr1::work(&a, &g);
//      use crate::codegen::RustCodegen;
//      println!("{}", g.gen(&RustCodegen, &a));
//      eprintln!("conflict: {:?}", a.conflict);
//    }
//    _ => {
//      let a = lr0::work(&g);
//      let a = lalr1_by_lr0::work(&a, &g);
//      use crate::codegen::RustCodegen;
//      println!("{}", g.gen(&RustCodegen, &a));
//      eprintln!("conflict: {:?}", a.conflict);
//    }
//  }

  //  let stub = GrammarStub {
//    prod: vec![
//      vec![
//        (vec![1], 0) // E' -> E
//      ],
//      vec![
//        (vec![1, 5, 1], 1), // E -> E + E
//        (vec![1, 6, 1], 2), // E -> E * E
//        (vec![2], 3) // E -> T
//      ],
//      vec![
//        (vec![7], 4) // T -> num
//      ]
//    ]
//  };
//  let a = lr1::work(&stub);
////  for (i, a) in a.iter().enumerate() {
////    println!("{}: {:?}", i, a);
////  }
//  let a = lalr1::work(&a, &stub);
////  println!("{:?}", a);
//  for (i, a) in a.action.iter().enumerate() {
////    let print = a.1.iter().any(|(_, act)| act.len() >= 2);
////    if print {
//    println!("{}: {:?}", i, a);
////    }
//  }
//  println!("{}", a.conflict.len());
}