pub struct IndentPrinter {
  indent: String,
  content: String,
}

impl IndentPrinter {
  const INDENT: u32 = 2;
  const INDENT_STR: &'static str = "  ";

  pub fn new() -> Self {
    Self { indent: String::new(), content: String::new() }
  }

  pub fn inc(&mut self) -> &mut Self {
    self.indent += IndentPrinter::INDENT_STR;
    self
  }

  pub fn dec(&mut self) -> &mut Self {
    for _ in 0..IndentPrinter::INDENT {
      self.indent.pop().unwrap();
    }
    self
  }

  pub fn ln(&mut self, s: impl AsRef<str>) -> &mut Self {
    self.content += self.indent.as_ref();
    self.content += s.as_ref();
    self.content.push('\n');
    self
  }

  pub fn lns(&mut self, s: impl AsRef<str>) -> &mut Self {
    for s in s.as_ref().split('\n') {
      self.ln(s);
    }
    self
  }

  pub fn finish(self) -> String {
    self.content
  }
}