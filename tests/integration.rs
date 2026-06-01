use {
  indoc::indoc,
  patcho::{
    Diagnostic, Error, FilePatch, Hunk, HunkLine, LineRange, UnifiedDiff, parse,
  },
  pretty_assertions::assert_eq,
};

#[derive(Debug)]
enum Expectation {
  Error(Error),
  Files(Vec<FilePatch>),
  None,
}

#[derive(Debug)]
struct Test {
  expected: Expectation,
  input: &'static str,
}

impl Test {
  fn error(self, error: Error) -> Self {
    let Self { expected, input } = self;

    let expected = match expected {
      Expectation::Error(_) => panic!("cannot expect more than one error"),
      Expectation::Files(_) => panic!("cannot expect both files and errors"),
      Expectation::None => Expectation::Error(error),
    };

    Self { expected, input }
  }

  fn file(self, file: FilePatch) -> Self {
    let Self { expected, input } = self;

    let expected = match expected {
      Expectation::Error(_) => panic!("cannot expect both files and errors"),
      Expectation::Files(files) => {
        Expectation::Files(files.into_iter().chain([file]).collect())
      }
      Expectation::None => Expectation::Files(vec![file]),
    };

    Self { expected, input }
  }

  fn new(input: &'static str) -> Self {
    Self {
      expected: Expectation::None,
      input,
    }
  }

  fn run(self) {
    let Self { expected, input } = self;

    match expected {
      Expectation::Error(expected) => {
        assert_eq!(parse(input).unwrap_err(), expected);
      }
      Expectation::Files(files) => {
        assert_eq!(parse(input).unwrap(), UnifiedDiff { files });
      }
      Expectation::None => panic!("missing expected result"),
    }
  }
}

#[test]
fn parses_default_ranges_and_no_newline_marker() {
  Test::new(indoc! {
    "
    --- foo
    +++ bar
    @@ -1 +1 @@
    -foo
    +bar
    \\ No newline at end of file
    "
  })
  .file(
    FilePatch::builder()
      .old("foo")
      .new("bar")
      .hunks(vec![
        Hunk::builder()
          .old(LineRange { start: 1, count: 1 })
          .new(LineRange { start: 1, count: 1 })
          .lines(vec![
            HunkLine::Remove("foo".to_string()),
            HunkLine::Add("bar".to_string()),
            HunkLine::NoNewlineAtEndOfFile,
          ])
          .build(),
      ])
      .build(),
  )
  .run();
}

#[test]
fn parses_empty_hunk_lines() {
  Test::new("--- foo\n+++ bar\n@@ -1,3 +1,3 @@\n \n-\n+\n")
    .file(
      FilePatch::builder()
        .old("foo")
        .new("bar")
        .hunks(vec![
          Hunk::builder()
            .old(LineRange { start: 1, count: 3 })
            .new(LineRange { start: 1, count: 3 })
            .lines(vec![
              HunkLine::Context(String::new()),
              HunkLine::Remove(String::new()),
              HunkLine::Add(String::new()),
            ])
            .build(),
        ])
        .build(),
    )
    .run();
}

#[test]
fn parses_git_diff_metadata_and_hunk_lines() {
  Test::new(indoc! {
    "
    diff --git a/foo b/bar
    index foo..bar 100644
    --- a/foo
    +++ b/bar
    @@ -1,2 +1,2 @@ baz
     foo
    -bar
    +baz
    "
  })
  .file(
    FilePatch::builder()
      .metadata(vec![
        "diff --git a/foo b/bar".to_string(),
        "index foo..bar 100644".to_string(),
      ])
      .old("a/foo")
      .new("b/bar")
      .hunks(vec![
        Hunk::builder()
          .old(LineRange { start: 1, count: 2 })
          .new(LineRange { start: 1, count: 2 })
          .section("baz")
          .lines(vec![
            HunkLine::Context("foo".to_string()),
            HunkLine::Remove("bar".to_string()),
            HunkLine::Add("baz".to_string()),
          ])
          .build(),
      ])
      .build(),
  )
  .run();
}

#[test]
fn parses_hunk_line_at_end_of_input_without_trailing_newline() {
  Test::new("--- foo\n+++ bar\n@@ -1 +1 @@\n foo")
    .file(
      FilePatch::builder()
        .old("foo")
        .new("bar")
        .hunks(vec![
          Hunk::builder()
            .old(LineRange { start: 1, count: 1 })
            .new(LineRange { start: 1, count: 1 })
            .lines(vec![HunkLine::Context("foo".to_string())])
            .build(),
        ])
        .build(),
    )
    .run();
}

#[test]
fn parses_multiple_file_patches_and_header_timestamps() {
  Test::new(indoc! {
    "
    --- foo\told
    +++ bar\tnew
    @@ -0,0 +1,2 @@
    +foo
    +bar
    diff --git a/baz b/qux
    --- baz\told
    +++ qux\tnew
    @@ -1,2 +1,0 @@ foo
    -foo
    -bar
    "
  })
  .file(
    FilePatch::builder()
      .old("foo")
      .new("bar")
      .hunks(vec![
        Hunk::builder()
          .old(LineRange { start: 0, count: 0 })
          .new(LineRange { start: 1, count: 2 })
          .lines(vec![
            HunkLine::Add("foo".to_string()),
            HunkLine::Add("bar".to_string()),
          ])
          .build(),
      ])
      .build(),
  )
  .file(
    FilePatch::builder()
      .metadata(vec!["diff --git a/baz b/qux".to_string()])
      .old("baz")
      .new("qux")
      .hunks(vec![
        Hunk::builder()
          .old(LineRange { start: 1, count: 2 })
          .new(LineRange { start: 1, count: 0 })
          .section("foo")
          .lines(vec![
            HunkLine::Remove("foo".to_string()),
            HunkLine::Remove("bar".to_string()),
          ])
          .build(),
      ])
      .build(),
  )
  .run();
}

#[test]
fn parses_multiple_hunks() {
  Test::new(indoc! {
    "
    --- foo
    +++ bar
    @@ -1 +1 @@
    -foo
    +bar
    @@ -3 +3 @@
    -baz
    +qux
    "
  })
  .file(
    FilePatch::builder()
      .old("foo")
      .new("bar")
      .hunks(vec![
        Hunk::builder()
          .old(LineRange { start: 1, count: 1 })
          .new(LineRange { start: 1, count: 1 })
          .lines(vec![
            HunkLine::Remove("foo".to_string()),
            HunkLine::Add("bar".to_string()),
          ])
          .build(),
        Hunk::builder()
          .old(LineRange { start: 3, count: 1 })
          .new(LineRange { start: 3, count: 1 })
          .lines(vec![
            HunkLine::Remove("baz".to_string()),
            HunkLine::Add("qux".to_string()),
          ])
          .build(),
      ])
      .build(),
  )
  .run();
}

#[test]
fn rejects_empty_input() {
  Test::new("")
    .error(Error {
      diagnostics: vec![Diagnostic {
        message: "found end of input expected any, newline, or '-'".to_string(),
        span: 0..0,
      }],
    })
    .run();
}

#[test]
fn rejects_hunk_line_without_prefix() {
  Test::new("--- foo\n+++ bar\n@@ -1 +1 @@\nfoo\n")
    .error(Error {
      diagnostics: vec![Diagnostic {
        message: "found end of input expected any, newline, or '-'".to_string(),
        span: 32..32,
      }],
    })
    .run();
}

#[test]
fn rejects_line_range_overflow() {
  Test::new(
    "--- foo\n+++ bar\n@@ -999999999999999999999999999999 +1 @@\n foo\n",
  )
  .error(Error {
    diagnostics: vec![Diagnostic {
      message: "number too large to fit in target type".to_string(),
      span: 20..50,
    }],
  })
  .run();
}

#[test]
fn rejects_missing_hunk() {
  Test::new("--- foo\n+++ bar\n")
    .error(Error {
      diagnostics: vec![Diagnostic {
        message: "found end of input expected '@'".to_string(),
        span: 16..16,
      }],
    })
    .run();
}

#[test]
fn rejects_missing_new_file_header() {
  Test::new("--- foo\n")
    .error(Error {
      diagnostics: vec![Diagnostic {
        message: "found end of input expected '+'".to_string(),
        span: 8..8,
      }],
    })
    .run();
}
