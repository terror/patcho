use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
  pub message: String,
  pub span: Range<usize>,
}

impl Display for Diagnostic {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "{} at {}..{}",
      self.message, self.span.start, self.span.end
    )
  }
}

impl From<ParseError<'_>> for Diagnostic {
  fn from(error: ParseError<'_>) -> Self {
    Self {
      message: error.to_string(),
      span: error.span().into_range(),
    }
  }
}
