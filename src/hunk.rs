use super::*;

#[derive(Clone, Debug, Eq, PartialEq, TypedBuilder)]
pub struct Hunk {
  #[builder(default)]
  pub lines: Vec<HunkLine>,
  pub new: LineRange,
  pub old: LineRange,
  #[builder(default, setter(strip_option, into))]
  pub section: Option<String>,
}
