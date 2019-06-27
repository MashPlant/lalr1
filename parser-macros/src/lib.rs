#![recursion_limit = "512"]
extern crate syn;
extern crate proc_macro;
extern crate quote;
extern crate regex;
extern crate lalr1_core;
extern crate re2dfa;
extern crate parser_gen;

use lalr1_core as lalr1;
use std::collections::HashMap;
use regex::Regex;
use quote::{ToTokens, quote};
use lalr1::{Assoc, AbstractGrammar, AbstractGrammarExt};
use std::fmt::Write;

fn parse_assoc(s: &str) -> Assoc {
  match s {
    "left" => Assoc::Left,
    "right" => Assoc::Right,
    "no_assoc" => Assoc::NoAssoc,
    _ => panic!("Invalid syntax in declaring term: assoc can only be `left`, `right`, or `no_assoc`, found `{}`.", s),
  }
}

enum ArgInfo {
  Self_,
  Arg { name: Option<String>, ty: String },
}

fn parse_arg(arg: &syn::FnArg) -> ArgInfo {
  match arg {
    syn::FnArg::SelfRef(_) => ArgInfo::Self_,
    syn::FnArg::SelfValue(_) => ArgInfo::Self_,
    syn::FnArg::Captured(arg) => ArgInfo::Arg {
      name: Some(arg.pat.clone().into_token_stream().to_string()),
      ty: arg.ty.clone().into_token_stream().to_string(),
    },
    // what is this?
    syn::FnArg::Inferred(_) => unimplemented!(),
    syn::FnArg::Ignored(ty) => ArgInfo::Arg { name: None, ty: ty.into_token_stream().to_string() }
  }
}

// is there a better way?
// I mean, if there is not such a api, since it is so frequently used, why do these libs not wrap them?
// and if there is, why is the document SO HARD to find?
fn parse_string(lit: &proc_macro2::Literal) -> String {
  let s = lit.to_string();
  s[s.find('\"').unwrap() + 1..s.rfind('\"').unwrap()].to_owned()
}

#[proc_macro_attribute]
pub fn lalr1(attr: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let parser_impl = match syn::parse::<syn::ItemImpl>(input) {
    Ok(parser_impl) => parser_impl,
    Err(_) => panic!("Attribute `lalr1` can only be applied to an impl block."),
  };
  let parser_type = parser_impl.self_ty.as_ref();

  const EPS: &'static str = "_Eps";
  const EOF: &'static str = "_Eof";

  let valid_name = Regex::new("^[a-zA-Z][a-zA-Z_0-9]*$").unwrap();
  let mut terms = vec![(EPS.to_owned(), None, None), (EOF.to_owned(), None, None)];
  let mut term2id = HashMap::new();
  term2id.insert(EPS.to_owned(), 0);
  term2id.insert(EOF.to_owned(), 1);
  let mut nt: Vec<(String, String)> = Vec::new();
  let mut nt2id = HashMap::new();

  let start = if let Some(proc_macro::TokenTree::Ident(ident)) = attr.clone().into_iter().next() {
    let start = ident.to_string();
    if !valid_name.is_match(&start) {
      panic!("Fail to parse start non-terminal, expect `#[lalr1(StartName)].");
    }
    start
  } else { panic!("Fail to parse start non-terminal, expect `#[lalr1(StartName)]."); };

  for (pri, attr) in parser_impl.attrs.iter().enumerate() {
    if attr.path.is_ident("term") {
      if let Some(proc_macro2::TokenTree::Group(group)) = attr.tts.clone().into_iter().next() {
        let mut term_it = group.stream().into_iter();
        while let Some(proc_macro2::TokenTree::Ident(ident)) = term_it.next() {
          let term = ident.to_string();
          match term.as_str() {
            EOF => panic!("User defined lex rule cannot return `{}`.", EOF),
            term if term != EPS => if term2id.contains_key(term) {
              panic!("Duplicate term: `{}`.", term);
            } else if !valid_name.is_match(term) {
              panic!("Term is not a valid variable name: `{}`.", term);
            } else {
              if let Some(proc_macro2::TokenTree::Group(group)) = term_it.next() {
                let mut re_it = group.stream().into_iter();
                let (re, pri_assoc) = match re_it.next() {
                  Some(proc_macro2::TokenTree::Literal(re)) => {
                    let re = parse_string(&re);
                    let pri_assoc = match re_it.next() {
                      Some(proc_macro2::TokenTree::Ident(assoc)) => Some((pri as u32, parse_assoc(&assoc.to_string()))),
                      _ => None,
                    };
                    (Some(re), pri_assoc)
                  }
                  Some(proc_macro2::TokenTree::Ident(assoc)) => {
                    (None, Some((pri as u32, parse_assoc(&assoc.to_string()))))
                  }
                  _ => panic!("Invalid syntax in declaring term: miss re/assoc. Should be `(TermName '(' (re|assoc|(re assoc)) ')')*`.")
                };
                term2id.insert(term.to_owned(), terms.len() as u32);
                terms.push((term.to_owned(), re, pri_assoc));
              } else { panic!("Invalid syntax in declaring term. Should be `(TermName '(' (re|assoc|(re assoc)) ')')*`."); }
            }
            // EPS
            _ => if let Some(proc_macro2::TokenTree::Group(group)) = term_it.next() {
              let mut re_it = group.stream().into_iter();
              let re = match re_it.next() {
                Some(proc_macro2::TokenTree::Literal(re)) => parse_string(&re),
                _ => panic!("Invalid syntax in declaring term: miss re(for `{}`, only accept re). Should be `(TermName '(' (re|assoc|(re assoc)) ')')*`..", EPS),
              };
              // index of EPS in terms is 0
              if terms[0].1.is_some() {// only allow define one(or zero) time
                panic!("Duplicate term: `{}`.", EPS);
              } else {
                terms[0].1 = Some(re);
              }
            } else { panic!("Invalid syntax in declaring term. Should be `(TermName '(' (re|assoc|(re assoc)) ')')*`."); }
          };
        }
      }
    }
  }
  // pass1, find all lhs
  for item in &parser_impl.items {
    if let syn::ImplItem::Method(method) = item {
      let lhs_ty = match &method.sig.decl.output {
        // still may be unit here, but not checked...
        syn::ReturnType::Type(_, ty) => ty.into_token_stream().to_string(),
        _ => panic!("Semantic rule `{}` must return a value.", method.sig.ident),
      };
      match method.attrs.get(0) {
        Some(attr) if attr.path.is_ident("rule") => {
          let rule = attr.tts.to_string();
          let mut rule_split = rule[1..rule.len() - 1].split_whitespace();
          let lhs = match rule_split.next() {
            Some(lhs) => lhs,
            None => panic!("The rule `{}` method `{}` defined doesn't have a valid lhs.", rule, method.sig.ident),
          };
          if !valid_name.is_match(lhs) {
            panic!("Non-terminal is not a valid variable name: `{}`.", lhs);
          } else if term2id.contains_key(lhs) {
            panic!("Non-terminal has a duplicate name with terminal: `{}`.", lhs);
          } else {
            nt2id.entry(lhs.to_owned()).and_modify(|old_loc| {
              // actually not modify, but check type
              let old_ty = &nt[*old_loc as usize].1;
              if old_ty != &lhs_ty {
                panic!("Non-terminal `{}` have conflict type: `{}` and `{}`.", lhs, old_ty, lhs_ty);
              }
            }).or_insert_with(|| {
              let id = nt.len() as u32;
              nt.push((lhs.to_owned(), lhs_ty));
              id
            });
          }
        }
        _ => panic!("Fail to parse production from method `{}`.", method.sig.ident),
      }
    } else { panic!("Impl block should only contain methods."); } // well I really want to support macro here, but I can't find api...
  }
  let mut prod = vec![Vec::new(); nt.len()];
  let mut prod_extra = Vec::new();
  let mut prod_id = 0u32;

  // pass2, find all rhs
  for item in &parser_impl.items {
    if let syn::ImplItem::Method(method) = item {
      let body = method.block.clone().into_token_stream().to_string();
      let attr = method.attrs.get(0).unwrap();
      let rule = attr.tts.to_string();
      let rule = &rule[1..rule.len() - 1];
      let mut rule_split = rule.split_whitespace();
      let lhs = nt2id[rule_split.next().unwrap()];
      match rule_split.next() {
        Some("->") => {}
        _ => panic!("The rule `{}` method `{}` defined doesn't have a `->`.", rule, method.sig.ident),
      };
      let lhs_prod = &mut prod[lhs as usize];
      let mut prod_rhs = Vec::new();
      let mut name_rhs = Vec::new();
      let mut pri_assoc = None;
      let rhs_ty = method.sig.decl.inputs.iter().map(parse_arg).collect::<Vec<_>>();
      let rhs_ty = match rhs_ty.get(0) {
        Some(ArgInfo::Self_) => &rhs_ty[1..],
        _ => &rhs_ty[..]
      };
      let rhs_tk = rule_split.collect::<Vec<_>>();
      if rhs_ty.len() != rhs_tk.len() {
        panic!("Production `{}`'s rhs and method `{}`'s arguments have different length.", rule, method.sig.ident);
      }
      for (&rhs_tk, rhs_ty) in rhs_tk.iter().zip(rhs_ty.iter()) {
        match rhs_ty {
          ArgInfo::Self_ => panic!("Method `{}` takes self argument in illegal position.", method.sig.ident),
          ArgInfo::Arg { name, ty } => {
            name_rhs.push(name.clone());
            match (nt2id.get(rhs_tk), term2id.get(rhs_tk)) {
              (Some(&nt_id), _) => {
                let nt_ty = &nt[nt_id as usize].1;
                if nt_ty != ty {
                  panic!("Production `{}`'s rhs and method `{}`'s arguments have conflict signature: `{}` requires `{}`, while method takes `{}`.",
                         rule, method.sig.ident, rhs_tk, nt_ty, ty);
                }
                prod_rhs.push(nt_id);
              }
              (_, Some(&t)) => {
                if !ty.starts_with("Token") { // maybe user will use some lifetime specifier
                  panic!("Production `{}`'s rhs and method `{}`'s arguments have conflict signature: `{}` requires Token, while method takes `{}`.",
                         rule, method.sig.ident, rhs_tk, ty);
                }
                prod_rhs.push(t + nt.len() as u32 + 1); // +1 for push a _start to nt later
                pri_assoc = terms[t as usize].2;
              }
              (None, None) => panic!("Production rhs contains undefined item: `{}`", rhs_tk),
            }
          }
        }
      }
      if let Some(prec) = match method.attrs.get(1) {
        Some(attr) if attr.path.is_ident("prec") => match attr.tts.clone().into_iter().next() {
          Some(proc_macro2::TokenTree::Group(group)) => match group.stream().into_iter().next() {
            Some(proc_macro2::TokenTree::Ident(ident)) => Some(ident.to_string()),
            _ => None,
          }
          _ => None,
        }
        _ => None
      } {
        match term2id.get(prec.as_str()) {
          None => panic!("Prec uses undefined terminal: `{}`", prec),
          Some(&t) => {
            pri_assoc = terms[t as usize].2;
          }
        }
      }
      let id = lhs_prod.len() as u32;
      lhs_prod.push((prod_rhs, prod_id));
      prod_extra.push(((name_rhs, body), (lhs, id), pri_assoc));
      prod_id += 1;
    }
  }

  let (res_type, res_id) = match nt2id.get(&start) {
    Some(&start) => {
      let new_start = {
        let start = &nt[start as usize];
        (format!("_{}", start.0), start.1.clone())
      };
      nt2id.insert(new_start.0.clone(), nt.len() as u32);
      prod.push(vec![(vec![start], prod_id)]);
      prod_extra.push(((vec![Some("_1".to_owned())], "_1".to_owned()), (nt.len() as u32, 0), None));
      nt.push(new_start.clone());
      (new_start.1.parse::<proc_macro2::TokenStream>().unwrap(), format!("_{}", start).parse::<proc_macro2::TokenStream>().unwrap())
    }
    _ => panic!("The start non-terminal `{}` is not produced by any production.", start),
  };

  pub struct Grammar {
    //         (name,   re,                    (pri, assoc) )
    terms: Vec<(String, Option<String>, Option<(u32, Assoc)>)>,
    //      (name,   type  )
    nt: Vec<(String, String)>,
    // all nt: one nt:   (prod, id)
    prod: Vec<Vec<(Vec<u32>, u32)>>,
    //               arg,                  body     lhs, index in prod[lhs]
    prod_extra: Vec<((Vec<Option<String>>, String), (u32, u32), Option<(u32, Assoc)>)>,
  }

  // this is almost the same as lalr1's Grammar struct...
  // maybe I will reduce some duplicate code in the future
  impl<'a> AbstractGrammar<'a> for Grammar {
    type ProdRef = Vec<u32>;
    type ProdIter = &'a Vec<(Vec<u32>, u32)>;

    fn start(&'a self) -> &'a (Self::ProdRef, u32) {
      &self.prod.last().unwrap()[0]
    }

    // first terminal
    fn eps(&self) -> u32 {
      self.prod.len() as u32
    }

    // second terminal
    fn eof(&self) -> u32 {
      self.prod.len() as u32 + 1
    }

    fn token_num(&self) -> u32 {
      self.terms.len() as u32 + self.prod.len() as u32
    }

    fn nt_num(&self) -> u32 {
      self.prod.len() as u32
    }

    fn get_prod(&'a self, lhs: u32) -> Self::ProdIter {
      &self.prod[lhs as usize]
    }
  }

  impl<'a> AbstractGrammarExt<'a> for Grammar {
    fn prod_pri_assoc(&self, id: u32) -> Option<(u32, Assoc)> {
      self.prod_extra[id as usize].2
    }

    fn term_pri_assoc(&self, ch: u32) -> Option<(u32, Assoc)> {
      self.terms[ch as usize - self.nt.len()].2
    }
  }

  let g = Grammar { terms, nt, prod, prod_extra };
  let table = lalr1::lr0::work(&g);
  let table = lalr1::lalr1_by_lr0::work(&table, &g);
  let mut dfas = Vec::new();
  for (idx, (_, re, _)) in g.terms.iter().enumerate() {
    match re {
      Some(re) => {
        match re2dfa::parse(&re) {
          Ok(re) => dfas.push(re2dfa::Dfa::from_nfa(&re2dfa::Nfa::from_re(&re), idx as u32).minimize()),
          Err(err) => panic!("Regex `{}` is invalid, reason: `{}`.", re, err),
        }
      }
      None => {}
    }
  }
  let dfa = re2dfa::Dfa::merge(&dfas);
  let ec = re2dfa::ec_of_dfas(&[&dfa]);

  let mut types = Vec::new();
  let mut types2id = HashMap::new();
  for (_, ty) in &g.nt {
    types2id.entry(ty.as_str()).or_insert_with(|| {
      let id = types.len() as u32;
      types.push(ty.as_str());
      id
    });
  }

  let mut token_type = String::new();
  for (nt, _) in &g.nt {
    let _ = write!(token_type, "{}, ", nt);
  }
  for (t, _, _) in &g.terms {
    let _ = write!(token_type, "{}, ", t);
  }
  let token_type = token_type.parse::<proc_macro2::TokenStream>().unwrap();

  let mut stack_item = "_Token(Token<'a>), ".to_owned();
  for (i, ty) in types.iter().enumerate() {
    let _ = write!(stack_item, "_{}({}), ", i, ty);
  }
  let stack_item = stack_item.parse::<proc_macro2::TokenStream>().unwrap();
  let dfa_size = dfa.nodes.len();
  let mut acc = String::new();
  for &(acc_, _) in &dfa.nodes {
    match acc_ {
      Some(acc_) => {
        let _ = write!(acc, "{}, ", g.terms[acc_ as usize].0);
      }
      None => acc += "_Eof, ",
    }
  }
  let acc = acc.parse::<proc_macro2::TokenStream>().unwrap();
  let mut ec_info = String::new();
  for ch in 0..128 {
    let _ = write!(ec_info, "{}, ", ec[ch]);
  }
  let ec_info = ec_info.parse::<proc_macro2::TokenStream>().unwrap();
  let u_dfa_size = parser_gen::min_u_of(dfa.nodes.len() as u32).parse::<proc_macro2::TokenStream>().unwrap();
  let ec_size = *ec.iter().max().unwrap() as usize + 1;
  let mut dfa_edge = String::new();
  let mut outs = vec![0; ec_size];
  for (_, edges) in dfa.nodes.iter() {
    for x in &mut outs { *x = 0; }
    for (&k, &out) in edges {
      outs[ec[k as usize] as usize] = out;
    }
    let _ = write!(dfa_edge, "{:?}, ", outs);
  }
  let dfa_edge = dfa_edge.parse::<proc_macro2::TokenStream>().unwrap();
  let u_lr_size = parser_gen::min_u_of(table.action.len() as u32).parse::<proc_macro2::TokenStream>().unwrap();
  let u_prod_len = parser_gen::min_u_of(g.prod_extra.iter().map(|&(_, (lhs, rhs), _)|
    g.prod[lhs as usize][rhs as usize].0.len()).max().unwrap() as u32).parse::<proc_macro2::TokenStream>().unwrap();
  ;
  let prod_size = g.prod_extra.len();
  let mut prod = String::new();
  for &(_, (lhs, rhs), _) in &g.prod_extra {
    let _ = write!(prod, "({}, {}), ", lhs, g.prod[lhs as usize][rhs as usize].0.len());
  }
  let prod = prod.parse::<proc_macro2::TokenStream>().unwrap();
  let token_size = g.terms.len() + g.nt.len();
  let lr_size = table.action.len();
  let mut lr_edge = String::new();
  for (_, edges) in &table.action {
    let _ = write!(lr_edge, "[");
    for i in 0..g.terms.len() + g.nt.len() {
      match edges.get(&(i as u32)) {
        Some(act) => { let _ = write!(lr_edge, "Act::{:?}, ", act[0]); }
        None => { let _ = write!(lr_edge, "Act::Err, "); }
      }
    }
    let _ = write!(lr_edge, "], ");
  }
  let lr_edge = lr_edge.parse::<proc_macro2::TokenStream>().unwrap();
  let mut parser_act = String::new();
  for (i, ((names, act), (lhs, rhs), _)) in g.prod_extra.iter().enumerate() {
    let _ = writeln!(parser_act, "{} => {{", i);
    let rhs = &g.prod[*lhs as usize][*rhs as usize];
    for (j, &x) in rhs.0.iter().enumerate().rev() {
      let name = names[j].as_ref().map(|s| s.as_ref()).unwrap_or("_");
      if x < g.nt.len() as u32 {
        let id = types2id[g.nt[x as usize].1.as_str()];
        let _ = writeln!(parser_act, "let {} = match value_stk.pop() {{ Some(StackItem::_{}(x)) => x, _ => impossible!() }};", name, id);
      } else {
        let _ = writeln!(parser_act, "let {} = match value_stk.pop() {{ Some(StackItem::_Token(x)) => x, _ => impossible!() }};", name);
      }
    }
    let _ = writeln!(parser_act, "let _0 = {{ {} }};", act);
    let id = types2id[g.nt[*lhs as usize].1.as_str()];
    let _ = writeln!(parser_act, "value_stk.push(StackItem::_{}(_0));", id);
    let _ = writeln!(parser_act, "}}");
  }
  let parser_act = parser_act.parse::<proc_macro2::TokenStream>().unwrap();

  let tokens = quote! {
    #[cfg(not(feature = "unsafe_parser"))]
    macro_rules! index {
      ($arr: expr, $idx: expr) => { $arr[$idx as usize] };
    }

    #[cfg(feature = "unsafe_parser")]
    macro_rules! index {
      ($arr: expr, $idx: expr) => { unsafe { *$arr.get_unchecked($idx as usize) } };
    }

    // just another name for unreachable
    #[cfg(not(feature = "unsafe_parser"))]
    macro_rules! impossible {
      () => { unreachable!() };
    }

    #[cfg(feature = "unsafe_parser")]
    macro_rules! impossible {
      () => { unsafe { std::hint::unreachable_unchecked() } };
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum TokenType { #token_type }

    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub enum Act { Shift(#u_lr_size), Reduce(#u_lr_size), Goto(#u_lr_size), Acc, Err }

    pub enum StackItem<'a> { #stack_item }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct Token<'a> {
      pub ty: TokenType,
      pub piece: &'a [u8],
      pub line: u32,
      pub col: u32,
    }

    pub struct Lexer<'a> {
      pub string: &'a [u8],
      pub cur_line: u32,
      pub cur_col: u32,
    }

    impl<'a> Lexer<'a> {
      pub fn new(string: &[u8]) -> Lexer {
        Lexer {
          string,
          cur_line: 1,
          cur_col: 1,
        }
      }

      pub fn next(&mut self) -> Option<Token<'a>> {
        use TokenType::*;
        static ACC: [TokenType; #dfa_size] = [#acc];
        static EC: [u8; 128] = [#ec_info];
        static EDGE: [[#u_dfa_size; #ec_size]; #dfa_size] = [#dfa_edge];
        loop {
          if self.string.is_empty() {
            return Some(Token { ty: _Eof, piece: "".as_bytes(), line: self.cur_line, col: self.cur_col });
          }
          let (mut line, mut col) = (self.cur_line, self.cur_col);
          let mut last_acc = _Eof; // this is arbitrary, just a value that cannot be returned by user defined function
          let mut state = 0;
          let mut i = 0;
          while i < self.string.len() {
            let ch = index!(self.string, i);
            let ec = index!(EC, ch & 0x7F);
            let nxt = index!(index!(EDGE, state), ec);
            let acc = index!(ACC, nxt);
            last_acc = if acc != _Eof { acc } else { last_acc };
            state = nxt;
            if nxt == 0 { // dead, should not eat this char
              if last_acc == _Eof { // completely dead
                return None;
              } else {
                let piece = &self.string[..i];
                self.string = &self.string[i..];
                if last_acc != _Eps {
                  return Some(Token { ty: last_acc, piece, line, col });
                } else {
                  line = self.cur_line;
                  col = self.cur_col;
                  last_acc = _Eof;
                  state = 0;
                  i = 0;
                }
              }
            } else { // continue, eat this char
              if ch == b'\n' {
                self.cur_line += 1;
                self.cur_col = 1;
              } else {
                self.cur_col += 1;
              }
              i += 1;
            }
          }
          // end of file
          if last_acc == _Eof { // completely dead
            return None;
          } else {
            // exec user defined function here
            let piece = &self.string[..i];
            self.string = &self.string[i..];
            if last_acc != _Eps {
              return Some(Token { ty: last_acc, piece, line, col });
            } else {
              return Some(Token { ty: _Eof, piece: "".as_bytes(), line: self.cur_line, col: self.cur_col });
            }
          }
        }
      }
    }

    impl<'a> Iterator for Lexer<'a> {
      type Item = Token<'a>;

      fn next(&mut self) -> Option<Self::Item> {
        Lexer::next(self)
      }
    }

    impl #parser_type {
      #[allow(unused)]
      #[allow(unused_mut)]
      pub fn parse<'a, L: IntoIterator<Item=Token<'a>>>(&mut self, lexer: L) -> Result<#res_type, Option<Token<'a>>> {
        static PROD: [(#u_lr_size, #u_prod_len); #prod_size] = [#prod];
        static EDGE: [[Act; #token_size]; #lr_size] = [#lr_edge];
        let mut value_stk: Vec<StackItem<'a>> = vec![];
        let mut state_stk: Vec<#u_lr_size> = vec![0];
        let mut lexer = lexer.into_iter();
        let mut token = match lexer.next() { Some(t) => t, None => return Err(None) };
        loop {
          let state = index!(state_stk, state_stk.len() - 1);
          let act = index!(index!(EDGE, state), token.ty);
          match act {
            Act::Shift(s) => {
              value_stk.push(StackItem::_Token(token));
              state_stk.push(s);
              token = match lexer.next() { Some(t) => t, None => return Err(None) };
            }
            Act::Reduce(r) => {
              let prod = index!(PROD, r);
              for _ in 0..prod.1 { match state_stk.pop() { None => impossible!(), Some(_) => {} }; }
              match r {
                #parser_act
                _ => impossible!(),
              }
              let cur = index!(state_stk, state_stk.len() - 1);
              let nxt = match index!(index!(EDGE, cur), prod.0) { Act::Goto(n) => n, _ => impossible!() };
              state_stk.push(nxt);
            }
            Act::Acc => {
              match state_stk.pop() { None => impossible!(), Some(_) => {} };
              let res = match value_stk.pop() { Some(StackItem::#res_id(r)) => r, _ => impossible!() };
              return Ok(res);
            }
            Act::Err => return Err(Some(token)),
            _ => impossible!(),
          }
        }
      }
    }
  };
  proc_macro::TokenStream::from(tokens)
}