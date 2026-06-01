use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Error {
  pub diagnostics: Vec<Diagnostic>,
}

impl Error {
  pub fn iter(&self) -> Iter<'_, Diagnostic> {
    self.diagnostics.iter()
  }
}

impl Display for Error {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let mut diagnostics = self.diagnostics.iter();

    match diagnostics.next() {
      Some(diagnostic) => {
        write!(f, "{diagnostic}")?;

        for diagnostic in diagnostics {
          write!(f, "\n{diagnostic}")?;
        }

        Ok(())
      }
      None => f.write_str("failed to parse unified diff"),
    }
  }
}

impl std::error::Error for Error {}

impl IntoIterator for Error {
  type IntoIter = std::vec::IntoIter<Diagnostic>;
  type Item = Diagnostic;

  fn into_iter(self) -> Self::IntoIter {
    self.diagnostics.into_iter()
  }
}

impl<'a> IntoIterator for &'a Error {
  type IntoIter = Iter<'a, Diagnostic>;
  type Item = &'a Diagnostic;

  fn into_iter(self) -> Self::IntoIter {
    self.diagnostics.iter()
  }
}

impl From<Vec<ParseError<'_>>> for Error {
  fn from(errors: Vec<ParseError<'_>>) -> Self {
    Self {
      diagnostics: errors.into_iter().map(Diagnostic::from).collect(),
    }
  }
}
