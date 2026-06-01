use super::*;

pub(crate) type ParseError<'src> = Rich<'src, char>;

fn file_header_parser<'src>(
  prefix: &'static str,
) -> impl Parser<'src, &'src str, String, extra::Err<ParseError<'src>>> {
  just(prefix)
    .ignore_then(any().and_is(text::newline().not()).repeated().to_slice())
    .then_ignore(text::newline().or(end()))
    .map(|line: &str| {
      line
        .split_once('\t')
        .map_or(line, |(path, _timestamp)| path)
        .to_string()
    })
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
      hunks,
      metadata,
      new,
      old,
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
    .then(any().and_is(text::newline().not()).repeated().to_slice())
    .then_ignore(text::newline().or(end()))
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

fn hunk_line_parser<'src>()
-> impl Parser<'src, &'src str, HunkLine, extra::Err<ParseError<'src>>> {
  choice((
    just("\\ No newline at end of file")
      .then_ignore(text::newline().or(end()))
      .to(HunkLine::NoNewlineAtEndOfFile),
    just(' ')
      .ignore_then(any().and_is(text::newline().not()).repeated().to_slice())
      .then_ignore(text::newline().or(end()))
      .map(|line: &str| HunkLine::Context(line.to_string())),
    just('+')
      .ignore_then(any().and_is(text::newline().not()).repeated().to_slice())
      .then_ignore(text::newline().or(end()))
      .map(|line: &str| HunkLine::Add(line.to_string())),
    just('-')
      .ignore_then(any().and_is(text::newline().not()).repeated().to_slice())
      .then_ignore(text::newline().or(end()))
      .map(|line: &str| HunkLine::Remove(line.to_string())),
  ))
}

fn hunk_parser<'src>()
-> impl Parser<'src, &'src str, Hunk, extra::Err<ParseError<'src>>> {
  hunk_header_parser()
    .then(hunk_line_parser().repeated().collect())
    .map(|((old, new, section), lines)| Hunk {
      lines,
      new,
      old,
      section,
    })
}

fn line_range_parser<'src>()
-> impl Parser<'src, &'src str, LineRange, extra::Err<ParseError<'src>>> {
  usize_parser()
    .then(just(',').ignore_then(usize_parser()).or_not())
    .map(|(start, count)| LineRange {
      count: count.unwrap_or(1),
      start,
    })
}

fn metadata_line_parser<'src>()
-> impl Parser<'src, &'src str, String, extra::Err<ParseError<'src>>> {
  any()
    .and_is(text::newline().not())
    .repeated()
    .to_slice()
    .and_is(just("--- ").not())
    .then_ignore(text::newline())
    .map(|line: &str| line.to_string())
}

pub(crate) fn parser<'src>()
-> impl Parser<'src, &'src str, UnifiedDiff, extra::Err<ParseError<'src>>> {
  file_patch_parser()
    .repeated()
    .at_least(1)
    .collect()
    .then_ignore(end())
    .map(|files| UnifiedDiff { files })
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
