use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnifiedDiff {
  pub files: Vec<FilePatch>,
}
