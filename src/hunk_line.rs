#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HunkLine {
  Add(String),
  Context(String),
  NoNewlineAtEndOfFile,
  Remove(String),
}
