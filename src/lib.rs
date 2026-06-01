use {chumsky::prelude::*, typed_builder::TypedBuilder};

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

pub fn parse(input: &str) -> Result<UnifiedDiff, Vec<ParseError<'_>>> {
  parser().parse(input).into_result()
}

pub fn parser<'src>()
-> impl Parser<'src, &'src str, UnifiedDiff, extra::Err<ParseError<'src>>> {
  file_patch_parser()
    .repeated()
    .at_least(1)
    .collect()
    .then_ignore(end())
    .map(|files| UnifiedDiff { files })
}

fn file_patch_parser<'src>()
-> impl Parser<'src, &'src str, FilePatch, extra::Err<ParseError<'src>>> {
  metadata_line_parser()
    .repeated()
    .collect()
    .then(file_header_parser("--- "))
    .then(file_header_parser("+++ "))
    .then(hunk_parser().repeated().at_least(1).collect())
    .map(|(((metadata, old), new), hunks)| FilePatch {
      metadata,
      old,
      new,
      hunks,
    })
}

fn metadata_line_parser<'src>()
-> impl Parser<'src, &'src str, String, extra::Err<ParseError<'src>>> {
  line_text()
    .and_is(just("--- ").not())
    .then_ignore(text::newline())
    .map(|line: &str| line.to_string())
}

fn file_header_parser<'src>(
  prefix: &'static str,
) -> impl Parser<'src, &'src str, String, extra::Err<ParseError<'src>>> {
  just(prefix)
    .ignore_then(line_text())
    .then_ignore(line_end())
    .map(|line: &str| {
      line
        .split_once('\t')
        .map(|(path, _timestamp)| path)
        .unwrap_or(line)
        .to_string()
    })
}

fn hunk_parser<'src>()
-> impl Parser<'src, &'src str, Hunk, extra::Err<ParseError<'src>>> {
  hunk_header_parser()
    .then(hunk_line_parser().repeated().collect())
    .map(|((old, new, section), lines)| Hunk {
      old,
      new,
      section,
      lines,
    })
}

fn hunk_header_parser<'src>() -> impl Parser<
  'src,
  &'src str,
  (LineRange, LineRange, Option<String>),
  extra::Err<ParseError<'src>>,
> {
  just("@@ -")
    .ignore_then(line_range_parser())
    .then_ignore(just(" +"))
    .then(line_range_parser())
    .then_ignore(just(" @@"))
    .then(line_text())
    .then_ignore(line_end())
    .map(|((old, new), section)| {
      let section = section.strip_prefix(' ').unwrap_or(section);
      let section = if section.is_empty() {
        None
      } else {
        Some(section.to_string())
      };

      (old, new, section)
    })
}

fn line_range_parser<'src>()
-> impl Parser<'src, &'src str, LineRange, extra::Err<ParseError<'src>>> {
  usize_parser()
    .then(just(',').ignore_then(usize_parser()).or_not())
    .map(|(start, count)| LineRange {
      start,
      count: count.unwrap_or(1),
    })
}

fn usize_parser<'src>()
-> impl Parser<'src, &'src str, usize, extra::Err<ParseError<'src>>> {
  text::int(10)
    .to_slice()
    .validate(
      |digits: &str, extra, emitter| match digits.parse::<usize>() {
        Ok(value) => value,
        Err(error) => {
          emitter.emit(Rich::custom(extra.span(), error.to_string()));
          0
        }
      },
    )
}

fn hunk_line_parser<'src>()
-> impl Parser<'src, &'src str, HunkLine, extra::Err<ParseError<'src>>> {
  choice((
    just("\\ No newline at end of file")
      .then_ignore(line_end())
      .to(HunkLine::NoNewlineAtEndOfFile),
    just(' ')
      .ignore_then(line_text())
      .then_ignore(line_end())
      .map(|line: &str| HunkLine::Context(line.to_string())),
    just('+')
      .ignore_then(line_text())
      .then_ignore(line_end())
      .map(|line: &str| HunkLine::Add(line.to_string())),
    just('-')
      .ignore_then(line_text())
      .then_ignore(line_end())
      .map(|line: &str| HunkLine::Remove(line.to_string())),
  ))
}

fn line_text<'src>()
-> impl Parser<'src, &'src str, &'src str, extra::Err<ParseError<'src>>> {
  any().and_is(text::newline().not()).repeated().to_slice()
}

fn line_end<'src>()
-> impl Parser<'src, &'src str, (), extra::Err<ParseError<'src>>> {
  text::newline().or(end())
}
