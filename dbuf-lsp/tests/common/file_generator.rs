//! Module contains functions to easily write easy-to-read
//! tests.
use std::iter;

use tower_lsp::lsp_types::Position;
use tower_lsp::lsp_types::Range;

enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L, R, Item> Iterator for Either<L, R>
where
    L: Iterator<Item = Item>,
    R: Iterator<Item = Item>,
{
    type Item = Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Either::Left(l) => l.next(),
            Either::Right(r) => r.next(),
        }
    }
}

/// Configuration for file creation.
pub struct FileConfig {
    /// current file name.
    file_name: &'static str,
    /// cursor representing character.
    cursor: char,
    /// end of line characters indicator for configuration lines
    config_line_indicator: &'static str,
    /// location representing character.
    location_pointer: char,
    /// character representing start of line.
    start_line_char: char,
}

impl Default for FileConfig {
    fn default() -> Self {
        Self {
            file_name: "test.dbuf",
            cursor: '|',
            config_line_indicator: "<---",
            location_pointer: '^',
            start_line_char: '|',
        }
    }
}

struct Pos(Position);

impl Pos {
    fn new_line(&mut self) {
        self.0.line += 1;
        self.0.character = 0;
    }

    fn new_char(&mut self) {
        self.0.character += 1;
    }

    fn reset_char(&mut self) {
        self.0.character = 0;
    }
}

#[derive(Default)]
struct Cursor(Vec<Position>);

impl Cursor {
    fn set(&mut self, p: Position) {
        self.0.push(p);
    }

    fn take(self) -> Vec<Position> {
        self.0
    }
}

#[derive(Default)]
struct Locations {
    current: Option<Range>,
    locations: Vec<Range>,
}

impl Locations {
    fn first_range(mut p: Position) -> Range {
        let start = p;
        p.character += 1;
        let end = p;
        Range::new(start, end)
    }

    fn advance(mut r: Range) -> Range {
        r.end.character += 1;
        r
    }

    fn set(&mut self, p: Position) {
        match self.current {
            Some(x) => {
                if x.end == p {
                    self.current = Self::advance(x).into();
                } else {
                    self.locations.push(x);
                    self.current = Self::first_range(p).into();
                }
            }
            None => self.current = Self::first_range(p).into(),
        }
    }

    fn take(mut self) -> Vec<Range> {
        if let Some(x) = self.current {
            self.locations.push(x);
        }
        self.locations
    }
}

impl FileConfig {
    /// Reads text and builds file metadata based on configuration.
    pub fn construct(self, text: &'static str) -> FileMetadata {
        let file_name = self.file_name;

        let mut content = String::new();
        let mut position = Position::new(0, 0);
        let mut cursor = Cursor::default();
        let mut locations = Locations::default();

        let text = if text.chars().nth(0) == Some('\n') {
            &text[1..]
        } else {
            text
        };

        let mut prev_line: Option<&str> = None;
        for line in text.lines() {
            let line = line
                .trim_start_matches(' ')
                .strip_prefix(self.start_line_char)
                .unwrap_or(line);
            let (line, special) =
                if let Some(stripped) = line.strip_suffix(self.config_line_indicator) {
                    (stripped, true)
                } else {
                    (line, false)
                };

            if special {
                assert!(position.line != 0, "Special line shouldn't be first");
                position.line -= 1;
            }

            let prev_line_iter = match prev_line {
                Some(line) => Either::Left(line.chars().map(Some).chain(iter::repeat(None))),
                None => Either::Right(iter::repeat(None)),
            };

            for (ch, prev_ch) in line.chars().zip(prev_line_iter) {
                if ch == self.cursor {
                    if !special {
                        cursor.set(position);
                    }
                    continue;
                }
                if special && prev_ch == Some(self.cursor) {
                    continue;
                }

                if special && ch == self.location_pointer {
                    locations.set(position);
                }

                if !special {
                    content.push(ch);
                }

                position.character += 1;
            }

            if !special {
                prev_line = line.into();
                content.push('\n');
            }

            position.character = 0;
            position.line += 1;
        }

        content = content.trim_end().to_string();
        content.push('\n');

        FileMetadata {
            file_name,
            content,
            cursors: cursor.take(),
            locations: locations.take(),
        }
    }
}

/// File with some metadata.
#[derive(Debug)]
pub struct FileMetadata {
    /// Name of file.
    file_name: &'static str,
    /// Content of file.
    content: String,
    /// Cursor positions.
    cursors: Vec<Position>,
    /// Extra locations.
    locations: Vec<Range>,
}

impl FileMetadata {
    pub fn file_name(&self) -> &'static str {
        self.file_name
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn cursors(&self) -> &[Position] {
        &self.cursors
    }

    pub fn locations(&self) -> &[Range] {
        &self.locations
    }
}
