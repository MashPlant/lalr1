// about the distribution of non-terminal & terminal & eof & eps on u32:
// non-terminal: 0..nt_num(), terminal & eof & eps: nt_num()..token_num()
pub trait AbstractGrammar {
  fn eps(&self) -> u32;

  fn eof(&self) -> u32;

  fn token_num(&self) -> u32;

  fn nt_num(&self) -> u32;

  fn get_prod<T, U>(&self, lhs: u32) -> T
    where T: IntoIterator<Item=U>,
          U: IntoIterator<Item=u32>;
}