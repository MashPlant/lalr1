#![allow(dead_code)]
#![allow(unused_mut)]

use regex::Regex;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TokenType {
  _Skip,
  Or,
  And,
  BOr,
  BXor,
  BAnd,
  Eq,
  Ne,
  Le,
  Ge,
  Lt,
  Gt,
  Repeat,
  Shl,
  Shr,
  Add,
  Sub,
  Mul,
  Div,
  Mod,
  UMinus,
  Not,
  Inc,
  Dec,
  LBracket,
  Dot,
  Default,
  RParenthes,
  Empty,
  Else,
  Identifier,
  GuardSplit,
  Colon,
  LBrace,
  RBrace,
  RBracket,
  LParenthes,
  Comma,
  Semicolon,
  Void,
  Int,
  Bool,
  String,
  New,
  Null,
  True,
  False,
  Class,
  Extends,
  This,
  While,
  Foreach,
  For,
  If,
  Return,
  Break,
  Print,
  ReadInteger,
  ReadLine,
  Static,
  InstanceOf,
  SCopy,
  Sealed,
  Var,
  In,
}

#[derive(Debug, Clone, Copy)]
pub enum LexerState {
  _Initial = 0,
  S = 1,
}

macro_rules! map (
  { $($key:expr => $value:expr),+ } => {{
    let mut m = ::std::collections::HashMap::new();
    $( m.insert($key, $value); )+
    m
  }};
);

#[derive(Debug, Clone, Copy)]
pub struct Token<'a> {
  ty: TokenType,
  piece: &'a str,
  line: u32,
  col: u32,
}


pub struct Lexer<'a> {
  string: &'a str,
  states: Vec<LexerState>,
  cur_line: u32,
  cur_col: u32,
  piece: &'a str,
  string_builder: String,
}

impl Lexer<'_> {
  pub fn new(string: &str) -> Lexer {
    Lexer {
      string,
      states: vec![LexerState::_Initial],
      cur_line: 1,
      cur_col: 0,
      piece: "",
      string_builder: String::new(),
    }
  }

  pub fn next(&mut self) -> Option<Token> {
    loop {
      if self.string.is_empty() {
        return None;
      }
      let mut max: Option<(&str, fn(&mut Lexer) -> TokenType)> = None;
      for (re, act) in &LEX_RULES[*self.states.last()? as usize] {
        match re.find(self.string) {
          None => {}
          Some(n) => {
            let n = n.as_str();
            if match max {
              None => true,
              Some((o, _)) => o.len() < n.len(),
            } {
              max = Some((n, *act));
            }
          }
        }
      }
      let (piece, act) = max?;
      self.piece = piece;
      self.string = &self.string[piece.len()..];
      let (line, col) = (self.cur_line, self.cur_col);
      for (i, l) in piece.split('\n').enumerate() {
        self.cur_line += 1;
        if i == 0 {
          self.cur_col += l.len() as u32;
        } else {
          self.cur_col = l.len() as u32;
        }
      }
      let ty = act(self);
      if ty != TokenType::_Skip {
        break Some(Token { ty, piece, line, col });
      }
    }
  }
}

lazy_static! {
  static ref LEX_RULES: [Vec<(Regex, fn(&mut Lexer) -> TokenType)>; 1] = [
    vec![
      (Regex::new(r#"^void"#).unwrap(), lex_act0),
      (Regex::new(r#"^int"#).unwrap(), lex_act1),
      (Regex::new(r#"^bool"#).unwrap(), lex_act2),
      (Regex::new(r#"^string"#).unwrap(), lex_act3),
      (Regex::new(r#"^new"#).unwrap(), lex_act4),
      (Regex::new(r#"^null"#).unwrap(), lex_act5),
      (Regex::new(r#"^true"#).unwrap(), lex_act6),
      (Regex::new(r#"^false"#).unwrap(), lex_act7),
      (Regex::new(r#"^class"#).unwrap(), lex_act8),
      (Regex::new(r#"^extends"#).unwrap(), lex_act9),
      (Regex::new(r#"^this"#).unwrap(), lex_act10),
      (Regex::new(r#"^while"#).unwrap(), lex_act11),
      (Regex::new(r#"^foreach"#).unwrap(), lex_act12),
      (Regex::new(r#"^for"#).unwrap(), lex_act13),
      (Regex::new(r#"^if"#).unwrap(), lex_act14),
      (Regex::new(r#"^else"#).unwrap(), lex_act15),
      (Regex::new(r#"^return"#).unwrap(), lex_act16),
      (Regex::new(r#"^break"#).unwrap(), lex_act17),
      (Regex::new(r#"^Print"#).unwrap(), lex_act18),
      (Regex::new(r#"^ReadInteger"#).unwrap(), lex_act19),
      (Regex::new(r#"^ReadLine"#).unwrap(), lex_act20),
      (Regex::new(r#"^static"#).unwrap(), lex_act21),
      (Regex::new(r#"^instanceof"#).unwrap(), lex_act22),
      (Regex::new(r#"^scopy"#).unwrap(), lex_act23),
      (Regex::new(r#"^sealed"#).unwrap(), lex_act24),
      (Regex::new(r#"^var"#).unwrap(), lex_act25),
      (Regex::new(r#"^default"#).unwrap(), lex_act26),
      (Regex::new(r#"^in"#).unwrap(), lex_act27),
      (Regex::new(r#"^\|\|\|"#).unwrap(), lex_act28),
      (Regex::new(r#"^<="#).unwrap(), lex_act29),
      (Regex::new(r#"^>="#).unwrap(), lex_act30),
      (Regex::new(r#"^=="#).unwrap(), lex_act31),
      (Regex::new(r#"^!="#).unwrap(), lex_act32),
      (Regex::new(r#"^\&\&"#).unwrap(), lex_act33),
      (Regex::new(r#"^\|\|"#).unwrap(), lex_act34),
      (Regex::new(r#"^%%"#).unwrap(), lex_act35),
      (Regex::new(r#"^\+\+"#).unwrap(), lex_act36),
      (Regex::new(r#"^\-\-"#).unwrap(), lex_act37),
      (Regex::new(r#"^<<"#).unwrap(), lex_act38),
      (Regex::new(r#"^>>"#).unwrap(), lex_act39),
      (Regex::new(r#"^\+"#).unwrap(), lex_act40),
      (Regex::new(r#"^\-"#).unwrap(), lex_act41),
      (Regex::new(r#"^\*"#).unwrap(), lex_act42),
      (Regex::new(r#"^/"#).unwrap(), lex_act43),
      (Regex::new(r#"^%"#).unwrap(), lex_act44),
      (Regex::new(r#"^\&"#).unwrap(), lex_act45),
      (Regex::new(r#"^\|"#).unwrap(), lex_act46),
      (Regex::new(r#"^\^"#).unwrap(), lex_act47),
      (Regex::new(r#"^="#).unwrap(), lex_act48),
      (Regex::new(r#"^<"#).unwrap(), lex_act49),
      (Regex::new(r#"^>"#).unwrap(), lex_act50),
      (Regex::new(r#"^\."#).unwrap(), lex_act51),
      (Regex::new(r#"^,"#).unwrap(), lex_act52),
      (Regex::new(r#"^;"#).unwrap(), lex_act53),
      (Regex::new(r#"^!"#).unwrap(), lex_act54),
      (Regex::new(r#"^\("#).unwrap(), lex_act55),
      (Regex::new(r#"^\)"#).unwrap(), lex_act56),
      (Regex::new(r#"^\["#).unwrap(), lex_act57),
      (Regex::new(r#"^\]"#).unwrap(), lex_act58),
      (Regex::new(r#"^\{"#).unwrap(), lex_act59),
      (Regex::new(r#"^\}"#).unwrap(), lex_act60),
      (Regex::new(r#"^:"#).unwrap(), lex_act61),
      (Regex::new(r#"^\s+"#).unwrap(), lex_act62),
      (Regex::new(r#"^\d+"#).unwrap(), lex_act63),
      (Regex::new(r#"^[A-Za-z][_0-9A-Za-z]*"#).unwrap(), lex_act64),
    ],
  ];
}

fn lex_act0(l: &mut Lexer) -> TokenType {
  TokenType::Void
}

fn lex_act1(l: &mut Lexer) -> TokenType {
  TokenType::Int
}

fn lex_act2(l: &mut Lexer) -> TokenType {
  TokenType::Bool
}

fn lex_act3(l: &mut Lexer) -> TokenType {
  TokenType::String
}

fn lex_act4(l: &mut Lexer) -> TokenType {
  TokenType::New
}

fn lex_act5(l: &mut Lexer) -> TokenType {
  TokenType::Null
}

fn lex_act6(l: &mut Lexer) -> TokenType {
  TokenType::True
}

fn lex_act7(l: &mut Lexer) -> TokenType {
  TokenType::False
}

fn lex_act8(l: &mut Lexer) -> TokenType {
  TokenType::Class
}

fn lex_act9(l: &mut Lexer) -> TokenType {
  TokenType::Extends
}

fn lex_act10(l: &mut Lexer) -> TokenType {
  TokenType::This
}

fn lex_act11(l: &mut Lexer) -> TokenType {
  TokenType::While
}

fn lex_act12(l: &mut Lexer) -> TokenType {
  TokenType::Foreach
}

fn lex_act13(l: &mut Lexer) -> TokenType {
  TokenType::For
}

fn lex_act14(l: &mut Lexer) -> TokenType {
  TokenType::If
}

fn lex_act15(l: &mut Lexer) -> TokenType {
  TokenType::Else
}

fn lex_act16(l: &mut Lexer) -> TokenType {
  TokenType::Return
}

fn lex_act17(l: &mut Lexer) -> TokenType {
  TokenType::Break
}

fn lex_act18(l: &mut Lexer) -> TokenType {
  TokenType::Print
}

fn lex_act19(l: &mut Lexer) -> TokenType {
  TokenType::ReadInteger
}

fn lex_act20(l: &mut Lexer) -> TokenType {
  TokenType::ReadLine
}

fn lex_act21(l: &mut Lexer) -> TokenType {
  TokenType::Static
}

fn lex_act22(l: &mut Lexer) -> TokenType {
  TokenType::InstanceOf
}

fn lex_act23(l: &mut Lexer) -> TokenType {
  TokenType::SCopy
}

fn lex_act24(l: &mut Lexer) -> TokenType {
  TokenType::Sealed
}

fn lex_act25(l: &mut Lexer) -> TokenType {
  TokenType::Var
}

fn lex_act26(l: &mut Lexer) -> TokenType {
  TokenType::Default
}

fn lex_act27(l: &mut Lexer) -> TokenType {
  TokenType::In
}

fn lex_act28(l: &mut Lexer) -> TokenType {
  TokenType::GuardSplit
}

fn lex_act29(l: &mut Lexer) -> TokenType {
  TokenType::Le
}

fn lex_act30(l: &mut Lexer) -> TokenType {
  TokenType::Ge
}

fn lex_act31(l: &mut Lexer) -> TokenType {
  TokenType::Eq
}

fn lex_act32(l: &mut Lexer) -> TokenType {
  TokenType::Ne
}

fn lex_act33(l: &mut Lexer) -> TokenType {
  TokenType::And
}

fn lex_act34(l: &mut Lexer) -> TokenType {
  TokenType::Or
}

fn lex_act35(l: &mut Lexer) -> TokenType {
  TokenType::Repeat
}

fn lex_act36(l: &mut Lexer) -> TokenType {
  TokenType::Inc
}

fn lex_act37(l: &mut Lexer) -> TokenType {
  TokenType::Dec
}

fn lex_act38(l: &mut Lexer) -> TokenType {
  TokenType::Shl
}

fn lex_act39(l: &mut Lexer) -> TokenType {
  TokenType::Shr
}

fn lex_act40(l: &mut Lexer) -> TokenType {
  TokenType::Add
}

fn lex_act41(l: &mut Lexer) -> TokenType {
  TokenType::Sub
}

fn lex_act42(l: &mut Lexer) -> TokenType {
  TokenType::Mul
}

fn lex_act43(l: &mut Lexer) -> TokenType {
  TokenType::Div
}

fn lex_act44(l: &mut Lexer) -> TokenType {
  TokenType::Mod
}

fn lex_act45(l: &mut Lexer) -> TokenType {
  TokenType::BAnd
}

fn lex_act46(l: &mut Lexer) -> TokenType {
  TokenType::BOr
}

fn lex_act47(l: &mut Lexer) -> TokenType {
  TokenType::BXor
}

fn lex_act48(l: &mut Lexer) -> TokenType {
  TokenType::Eq
}

fn lex_act49(l: &mut Lexer) -> TokenType {
  TokenType::Lt
}

fn lex_act50(l: &mut Lexer) -> TokenType {
  TokenType::Gt
}

fn lex_act51(l: &mut Lexer) -> TokenType {
  TokenType::Dot
}

fn lex_act52(l: &mut Lexer) -> TokenType {
  TokenType::Comma
}

fn lex_act53(l: &mut Lexer) -> TokenType {
  TokenType::Semicolon
}

fn lex_act54(l: &mut Lexer) -> TokenType {
  TokenType::Not
}

fn lex_act55(l: &mut Lexer) -> TokenType {
  TokenType::LParenthes
}

fn lex_act56(l: &mut Lexer) -> TokenType {
  TokenType::RParenthes
}

fn lex_act57(l: &mut Lexer) -> TokenType {
  TokenType::LBracket
}

fn lex_act58(l: &mut Lexer) -> TokenType {
  TokenType::RBracket
}

fn lex_act59(l: &mut Lexer) -> TokenType {
  TokenType::LBrace
}

fn lex_act60(l: &mut Lexer) -> TokenType {
  TokenType::RBrace
}

fn lex_act61(l: &mut Lexer) -> TokenType {
  TokenType::Colon
}

fn lex_act62(l: &mut Lexer) -> TokenType {
  TokenType::_Skip
}

fn lex_act63(l: &mut Lexer) -> TokenType {
  TokenType::Int
}

fn lex_act64(l: &mut Lexer) -> TokenType {
  TokenType::Identifier
}


