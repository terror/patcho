use super::*;

#[derive(Clone, Debug, Eq, PartialEq, TypedBuilder)]
pub struct FilePatch {
  #[builder(default)]
  pub hunks: Vec<Hunk>,
  #[builder(default)]
  pub metadata: Vec<String>,
  #[builder(setter(into))]
  pub new: String,
  #[builder(setter(into))]
  pub old: String,
}
