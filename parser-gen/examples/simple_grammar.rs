use grammar_config::*;
use std::fmt::Write;
use clap::{App, Arg};
use std::{io, fs, process};
use indexmap::IndexSet;

pub struct SimpleGrammar<'a> {
  nt: IndexSet<&'a str>,
  t: IndexSet<&'a str>,
  prod: Vec<Vec<(Vec<u32>, u32)>>,
  prod_num: u32,
  augment_lhs: String,
}

impl<'a> SimpleGrammar<'a> {
  pub fn from_text(text: &'a str) -> Result<SimpleGrammar, String> {
    let (mut nt, mut t) = (IndexSet::default(), IndexSet::default());
    t.insert("ε"); // eps
    t.insert("#"); // eof
    t.insert("ERROR"); // err

    for line in text.lines() {
      let mut sp = line.split("->");
      let lhs = sp.next().ok_or_else(||
        format!("production lhs not found in line `{}`, please use format lhs -> rhs1 rhs2 ... in each line", line))?.trim();
      // lhs comes from `split`, it is possible to have zero length, so need `unwrap_or`
      if !lhs.chars().nth(0).unwrap_or('a').is_uppercase() {
        return Err(format!("production lhs `{}` should start with a uppercase character", lhs));
      }
      nt.insert(lhs);
    }
    let augment_lhs = if let Some(lhs) = nt.get_index(0) { format!("{}'", lhs) } else {
      return Err("grammar must have at least one production rule".to_owned());
    };
    nt.insert(""); // S', won't be shown, just a placeholder

    let (mut prod, mut prod_num) = (Vec::new(), 0);
    for (idx, line) in text.lines().enumerate() {
      let mut sp = line.split("->");
      let (lhs, rhs) = (sp.next().unwrap().trim(), sp.next().unwrap_or(""));
      let lhs = nt.get_full(lhs).unwrap().0;
      let mut rhs_vec = Vec::new();
      for rhs in rhs.split_whitespace() {
        // rhs comes from split_whitespace, it contains at least 1 char, so call `unwrap` is ok
        let id = if rhs.chars().nth(0).unwrap().is_uppercase() { // nt
          nt.get_full(rhs).ok_or_else(|| format!("no such non-terminal token `{}`", rhs))?.0
        } else { // t
          t.insert_full(rhs).0 + nt.len()
        } as u32;
        rhs_vec.push(id);
      }
      if prod.len() <= lhs {
        prod.resize_with(lhs + 1, || Vec::new());
      }
      prod[lhs].push((rhs_vec, idx as u32));
      prod_num += 1;
    }
    prod.push(vec![(vec![0], prod_num)]); // all == lines().count()
    Ok(SimpleGrammar { nt, t, prod, prod_num: prod_num + 1, augment_lhs })
  }
}

impl<'a> AbstractGrammar<'a> for SimpleGrammar<'a> {
  type ProdRef = Vec<u32>;
  type ProdIter = &'a Vec<(Vec<u32>, u32)>;

  fn start(&'a self) -> (u32, &'a (Self::ProdRef, u32)) {
    let last = self.prod.len() - 1;
    (last as u32, &self.prod[last][0])
  }

  fn eps(&self) -> u32 { self.prod.len() as u32 }
  fn eof(&self) -> u32 { self.prod.len() as u32 + 1 }
  fn err(&self) -> u32 { self.prod.len() as u32 + 2 }

  fn token_num(&self) -> u32 { self.t.len() as u32 + self.prod.len() as u32 }
  fn nt_num(&self) -> u32 { self.prod.len() as u32 }
  fn prod_num(&self) -> u32 { self.prod_num }
  fn get_prod(&'a self, lhs: u32) -> Self::ProdIter { &self.prod[lhs as usize] }
}

// doesn't really have, just a simple grammar
impl<'a> AbstractGrammarExt<'a> for SimpleGrammar<'a> {
  fn prod_pri(&self, _id: u32) -> Option<u32> { None }
  fn term_pri_assoc(&self, _ch: u32) -> Option<(u32, Assoc)> { None }

  fn show_token(&self, id: u32) -> &str {
    let id = id as usize;
    match self.nt.get_index(id) {
      Some(&"") => &self.augment_lhs,
      Some(s) => s,
      None => self.t.get_index(id - self.nt.len()).unwrap()
    }
  }

  fn show_prod(&self, id: u32, dot: Option<u32>) -> String {
    let mut text = String::new();
    let (lhs, (prod, _)) = self.prod.iter().enumerate()
      .filter_map(|(idx, prods)| Some((idx, prods.iter().find(|prod| prod.1 == id)?)))
      .nth(0).unwrap();
    let _ = write!(text, "{}→", self.show_token(lhs as u32));
    for i in 0..prod.len() as u32 {
      if Some(i) == dot { text.push('.'); }
      text += self.show_token(prod[i as usize]);
    }
    if Some(prod.len() as u32) == dot { text.push('.'); }
    text
  }
}

fn main() -> io::Result<()> {
  use lalr1_core::*;
  use parser_gen::{show_lr, show_ll};

  let m = App::new("simple_grammar")
    .arg(Arg::with_name("input").required(true))
    .arg(Arg::with_name("output").long("output").short("o").takes_value(true))
    .arg(Arg::with_name("grammar").long("grammar").short("g").takes_value(true).possible_values(&["lr0", "lr1", "lalr1", "ll1"]).required(true))
    .get_matches();
  let input = fs::read_to_string(m.value_of("input").unwrap())?;
  let ref g = SimpleGrammar::from_text(&input).unwrap_or_else(|e| {
    eprintln!("Grammar is invalid, reason: {}.", e);
    process::exit(1);
  });
  let result = match m.value_of("grammar") {
    Some("lr0") => show_lr::lr0_dot(g, &lr0::work(g)),
    Some("lr1") => show_lr::lr1_dot(g, &lr1::work(g)),
    Some("lalr1") => show_lr::lr1_dot(g, &lalr1_by_lr0::work(&lr0::work(g), g)),
    Some("ll1") => show_ll::table(&ll1_core::LLCtx::new(g), g),
    _ => unreachable!(),
  };
  if let Some(output) = m.value_of("output") {
    fs::write(output, result)?;
  } else { print!("{}", result); }
  Ok(())
}