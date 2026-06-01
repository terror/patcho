use {chumsky::prelude::*, parser::parser, typed_builder::TypedBuilder};

mod parser;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnifiedDiff {
  pub files: Vec<FilePatch>,
}

#[derive(Clone, Debug, Eq, PartialEq, TypedBuilder)]
pub struct FilePatch {
  #[builder(default)]
  pub metadata: Vec<String>,
  #[builder(setter(into))]
  pub old: String,
  #[builder(setter(into))]
  pub new: String,
  #[builder(default)]
  pub hunks: Vec<Hunk>,
}

#[derive(Clone, Debug, Eq, PartialEq, TypedBuilder)]
pub struct Hunk {
  pub old: LineRange,
  pub new: LineRange,
  #[builder(default, setter(strip_option, into))]
  pub section: Option<String>,
  #[builder(default)]
  pub lines: Vec<HunkLine>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LineRange {
  pub start: usize,
  pub count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HunkLine {
  Context(String),
  Add(String),
  Remove(String),
  NoNewlineAtEndOfFile,
}

pub type ParseError<'src> = Rich<'src, char>;

/// Parses a unified diff into a [`UnifiedDiff`].
///
/// The input must contain one or more file patches. Each file patch may start
/// with metadata lines, followed by an old file header beginning with `--- `,
/// a new file header beginning with `+++ `, and one or more hunks. Header
/// timestamps separated from file paths by a tab are ignored.
///
/// Hunk ranges without an explicit count default to a count of `1`. Hunk
/// section text is returned as [`Hunk::section`], with the separating space
/// removed and an empty section represented as [`None`]. Hunk body lines are
/// returned as [`HunkLine::Context`], [`HunkLine::Add`],
/// [`HunkLine::Remove`], or [`HunkLine::NoNewlineAtEndOfFile`].
///
/// The parser consumes the entire input. If the input is empty, incomplete, or
/// contains text that does not match the supported unified diff grammar, this
/// returns the parser errors emitted while reading the input.
///
/// # Examples
///
/// ```
/// use patcho::{HunkLine, parse};
///
/// let diff = parse(
///   "\
/// --- foo
/// +++ bar
/// @@ -1 +1 @@ baz
/// -foo
/// +bar
/// ",
/// )
/// .unwrap();
///
/// let file = &diff.files[0];
///
/// assert_eq!(file.old, "foo");
/// assert_eq!(file.new, "bar");
/// assert_eq!(file.hunks[0].section.as_deref(), Some("baz"));
/// assert_eq!(file.hunks[0].lines[0], HunkLine::Remove("foo".to_string()));
/// assert_eq!(file.hunks[0].lines[1], HunkLine::Add("bar".to_string()));
/// ```
pub fn parse(input: &str) -> Result<UnifiedDiff, Vec<ParseError<'_>>> {
  parser().parse(input).into_result()
}
