use grammar_config::*;
use crate::lr0::LRItem;
use crate::lr1::LRResult;
use hashbrown::HashMap;
use std::fmt::Write;

pub struct SimpleGrammar<'a> {
  prod: Vec<Vec<(Vec<u32>, u32)>>,
  nt: Vec<&'a str>,
  t: Vec<&'a str>,
}

impl<'a> SimpleGrammar<'a> {
  // very simple, no err check, will panic when handling illegal input...
  pub fn from_text(text: &'a str) -> SimpleGrammar {
    let mut nt = Vec::new();
    let mut t = Vec::new();
    let mut nt2id = HashMap::new();
    let mut t2id = HashMap::new();
    let mut prod = Vec::new();
    let mut all = 0;
    // TODO: test it
    t.push("ε");
    t.push("#");
    t.push("");
    t2id.insert("ε", 0);
    t2id.insert("#", 1);
    t2id.insert("", 2);

    for line in text.lines() {
      let mut sp = line.split("->");
      // my IDE refuse to give me type hint...
      let lhs: &str = sp.next().unwrap().trim();
      assert!(lhs.chars().nth(0).unwrap().is_uppercase());
      nt2id.entry(lhs).or_insert_with(|| {
        let id = nt.len() as u32;
        nt.push(lhs);
        id
      });
    }
    nt.push(""); // S'
    for (idx, line) in text.lines().enumerate() {
      let mut sp = line.split("->");
      // my IDE refuse to give me type hint...
      let (lhs, rhs): (&str, &str) = (sp.next().unwrap().trim(), sp.next().unwrap());
      let lhs = nt2id[lhs];
      let rhs = rhs.split_whitespace().map(|rhs| {
        // first char is uppercase -> nt, else t
        if rhs.chars().nth(0).unwrap().is_uppercase() { // nt
          nt2id[rhs]
        } else { // t
          *t2id.entry(rhs).or_insert_with(|| {
            let id = t.len() as u32;
            t.push(rhs);
            id
          }) + nt.len() as u32
        }
      }).collect::<Vec<_>>();
      if prod.len() <= lhs as usize {
        prod.resize_with(lhs as usize + 1, || Vec::new());
      }
      prod[lhs as usize].push((rhs, idx as u32));
      all += 1;
    }
    prod.push(vec![(vec![0], all)]); // all == lines().count()
    SimpleGrammar { prod, nt, t }
  }


  fn token_at(&self, k: u32) -> &str {
    let k = k as usize;
    if k < self.nt.len() { self.nt[k] } else { self.t[k - self.nt.len()] }
  }

  pub fn print_lr1(&self, lr1: &Vec<LRResult<'a>>) -> String {
    let mut s = "digraph g {".to_owned();
    for (idx, (state, link)) in lr1.iter().enumerate() {
      for (&k, &v) in link {
        let _ = writeln!(s, r#"{} -> {} [label="{}"];"#, idx, v, self.token_at(k));
      }
      let mut state_text = String::new();
      for (item, look_ahead) in &state.items {
        self.state_text_common(&item, &mut state_text);
        state_text.push(',');
        for i in self.nt.len()..self.nt.len() + self.t.len() {
          if look_ahead.test(i) {
            state_text += self.token_at(i as u32);
            state_text.push('/');
          }
        }
        state_text.pop();
        state_text += r#"\n"#;
      }
      state_text.pop();
      state_text.pop(); // extra \n
      let _ = writeln!(s, r#"{}[shape=box, label="{}"]"#, idx, state_text);
    }
    s.push('}');
    s
  }

  pub fn print_lr0(&self, lr0: &Vec<(Vec<LRItem<'a>>, HashMap<u32, u32>)>) -> String {
    let mut s = "digraph g {".to_owned();
    for (idx, (state, link)) in lr0.iter().enumerate() {
      for (&k, &v) in link {
        let _ = writeln!(s, r#"{} -> {} [label="{}"];"#, idx, v, self.token_at(k));
      }
      let mut state_text = String::new();
      for item in state {
        self.state_text_common(&item, &mut state_text);
        state_text += r#"\n"#;
      }
      state_text.pop();
      state_text.pop(); // extra \n
      let _ = writeln!(s, r#"{}[shape=box, label="{}"]"#, idx, state_text);
    }
    s.push('}');
    s
  }

  fn state_text_common(&self, item: &LRItem, state_text: &mut String) {
    let mut lhs = 0;
    // a naive search... no need to optimize after all
    'out: for (idx, prods) in self.prod.iter().enumerate() {
      for prod in prods {
        if prod.1 == item.prod_id {
          lhs = idx;
          break 'out;
        }
      }
    }
    if lhs == self.nt.len() - 1 { // added S'
      let _ = write!(state_text, "{}'→", self.nt[0]);
    } else {
      let _ = write!(state_text, "{}→", self.nt[lhs]);
    }
    for i in 0..item.dot {
      *state_text += self.token_at(item.prod[i as usize]);
    }
    state_text.push('.');
    for i in item.dot..item.prod.len() as u32 {
      *state_text += self.token_at(item.prod[i as usize]);
    }
  }
}

impl<'a> AbstractGrammar<'a> for SimpleGrammar<'a> {
  type ProdRef = Vec<u32>;
  type ProdIter = &'a Vec<(Vec<u32>, u32)>;

  fn start(&'a self) -> (u32, &'a (Self::ProdRef, u32)) {
    let last = self.prod.len() - 1;
    (last as u32, &self.prod[last][0])
  }

  fn eps(&self) -> u32 {
    self.prod.len() as u32
  }

  fn eof(&self) -> u32 {
    self.prod.len() as u32 + 1
  }

  fn err(&self) -> u32 {
    self.prod.len() as u32 + 2
  }

  fn token_num(&self) -> u32 {
    self.t.len() as u32 + self.prod.len() as u32
  }

  fn nt_num(&self) -> u32 {
    self.prod.len() as u32
  }

  fn get_prod(&'a self, lhs: u32) -> Self::ProdIter {
    &self.prod[lhs as usize]
  }
}

// doesn't really have, just a simple grammar
impl<'a> AbstractGrammarExt<'a> for SimpleGrammar<'a> {
  fn prod_pri(&self, _id: u32) -> Option<u32> {
    None
  }

  fn term_pri_assoc(&self, _ch: u32) -> Option<(u32, Assoc)> {
    None
  }
}