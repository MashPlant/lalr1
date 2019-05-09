#![allow(unused)]
#![feature(fn_traits)]
extern crate toml;
extern crate serde;
extern crate serde_derive;
extern crate regex;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate smallvec;

mod printer;
mod parser;
mod raw_grammar;
mod abstract_grammar;
mod grammar;
mod lr1;
mod lalr1;
mod bitset;
mod codegen;

use crate::abstract_grammar::{AbstractGrammar, AbstractGrammarExt};
use crate::raw_grammar::{Assoc, RawGrammar};
use std::fs::read_to_string;
use std::slice::Iter;
use std::iter::Map;
//use crate::parser::TokenType;
use crate::codegen::RustCodegen;

struct GrammarStub {
  prod: Vec<Vec<(Vec<u32>, u32)>>
}

impl<'a> AbstractGrammar<'a> for GrammarStub {
  type ProdRef = Vec<u32>;
  type ProdIter = &'a Vec<(Vec<u32>, u32)>;

  fn start(&'a self) -> &'a (Self::ProdRef, u32) {
    &self.prod[0][0]
  }

  fn eps(&self) -> u32 {
    3
  }

  fn eof(&self) -> u32 {
    4
  }

  fn token_num(&self) -> u32 {
    8
  }

  fn nt_num(&self) -> u32 {
    3
  }

  fn get_prod(&'a self, lhs: u32) -> Self::ProdIter {
    &self.prod[lhs as usize]
  }
}

impl<'a> AbstractGrammarExt<'a> for GrammarStub {
  fn prod_pri_assoc(&self, id: u32) -> Option<(u32, Assoc)> {
    match id {
      0 => None,
      1 => Some((0, Assoc::Left)),
      2 => Some((1, Assoc::Left)),
      3 => None,
      4 => None,
      _ => panic!("out of range")
    }
  }

  fn term_pri_assoc(&self, ch: u32) -> Option<(u32, Assoc)> {
    match ch {
      3 => None,
      4 => None,
      5 => Some((0, Assoc::Left)),
      6 => Some((1, Assoc::Left)),
      7 => None,
      _ => panic!("out of range")
    }
  }
}

fn main() {
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

//  let prog = read_to_string("test.decaf").unwrap();
//  let mut lex = parser::Lexer::new(&prog);
//  while let Some(tk) = lex.next() {
//    println!("{:?}", tk);
//    if tk.ty == TokenType::_Eof {
//      break;
//    }
//  }
  let s = read_to_string("decaf.toml").unwrap();
  let mut g: RawGrammar = toml::from_str(&s).unwrap();
  let g = g.to_grammar().unwrap();

  let a = lr1::work(&g);
//  for (i, a) in a.iter().enumerate() {
//    println!("{}: {:?}", i, a);
//  }
  let a = lalr1::work(&a, &g);
//  for (i, a) in a.action.iter().enumerate() {
//    println!("{}: {:?}", i, a);
//  }
//  println!("{:?}", a.conflict);
  println!("{}", g.gen(&RustCodegen,&a));
}