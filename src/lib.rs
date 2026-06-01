use {
  chumsky::{error::Rich, prelude::*},
  diagnostic::Diagnostic,
  error::Error,
  parser::{ParseError, parser},
  std::{
    fmt::{self, Display, Formatter},
    ops::Range,
    slice::Iter,
    vec::IntoIter,
  },
  typed_builder::TypedBuilder,
};

pub use {
  file_patch::FilePatch, hunk::Hunk, hunk_line::HunkLine,
  line_range::LineRange, unified_diff::UnifiedDiff,
};

mod diagnostic;
mod error;
mod file_patch;
mod hunk;
mod hunk_line;
mod line_range;
mod parser;
mod unified_diff;

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
/// The parser consumes the entire input.
///
/// # Errors
///
/// Returns [`Error`] if `input` is empty, incomplete, or does not match the
/// supported unified diff grammar. The error contains one or more diagnostics
/// with byte spans into the input.
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
pub fn parse(input: &str) -> Result<UnifiedDiff, Error> {
  parser().parse(input).into_result().map_err(Error::from)
}
