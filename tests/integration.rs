use {
  indoc::indoc,
  patcho::{FilePatch, Hunk, HunkLine, LineRange, UnifiedDiff, parse},
  pretty_assertions::assert_eq,
};

#[derive(Debug)]
struct Test {
  files: Vec<FilePatch>,
  input: &'static str,
}

impl Test {
  fn file(self, file: FilePatch) -> Self {
    Self {
      files: self.files.into_iter().chain([file]).collect(),
      ..self
    }
  }

  fn new(input: &'static str) -> Self {
    Self {
      files: Vec::new(),
      input,
    }
  }

  fn run(self) {
    assert_eq!(
      parse(self.input)
        .unwrap_or_else(|errors| panic!("parse errors: {errors:#?}")),
      UnifiedDiff { files: self.files }
    );
  }
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
