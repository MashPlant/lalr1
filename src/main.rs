#[macro_use]
extern crate smallvec;
extern crate toml;
extern crate serde;
extern crate serde_derive;
extern crate regex;
extern crate re2dfa;

mod printer;
mod raw_grammar;
mod abstract_grammar;
mod grammar;
mod lr1;
mod lalr1_by_lr1;
mod bitset;
mod codegen;
mod lr0;
mod lalr1_by_lr0;
mod lalr1_common;
mod simple_grammar;

use crate::abstract_grammar::{AbstractGrammar, AbstractGrammarExt};
use crate::raw_grammar::{Assoc, RawGrammar};
use std::fs::{read_to_string, File};
use crate::simple_grammar::SimpleGrammar;
use std::io::Write;


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

  let s = read_to_string("src/example/decaf.toml").unwrap();
  let mut g: RawGrammar = toml::from_str(&s).unwrap();

  let g = g.extend_grammar().unwrap();

  use std::env;

  match env::args().nth(1) {
    Some(ref one) if one.as_str() == "1" => {
      let a = lr1::work(&g);
      let a = lalr1_by_lr1::work(&a, &g);
      use crate::codegen::RustCodegen;
      println!("{}", g.gen(&RustCodegen, &a));
      eprintln!("conflict: {:?}", a.conflict);
    }
    _ => {
      let a = lr0::work(&g);
      let a = lalr1_by_lr0::work(&a, &g);
      use crate::codegen::RustCodegen;
      println!("{}", g.gen(&RustCodegen, &a));
      eprintln!("conflict: {:?}", a.conflict);
    }
  }

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

//  use crate::parser::TokenType;
//  let prog = read_to_string("calc.txt").unwrap();
//  let mut lex = parser::Lexer::new(&prog);
//  while let Some(tk) = lex.next() {
//    println!("{:?}", tk);
//    if tk.ty == TokenType::_Eof {
//      break;
//    }
//  }
//  let mut parser = parser::Parser::new(&prog);
//  let a = parser.parse();
//  println!("{:?}", a);
}