//! Tests for common (workspace generation)

mod common;

use common::FileConfig;
use tower_lsp::lsp_types::{Position, Range};

use pretty_assertions::assert_eq;

#[test]
fn as_it() {
    {
        const TEXT: &str = r"
          |aba
        ";

        let meta = FileConfig::default().construct(TEXT);

        assert_eq!(meta.content(), "aba\n");
        assert_eq!(meta.cursors(), vec![]);
        assert_eq!(meta.locations(), vec![]);
    }

    {
        const TEXT: &str = r"
          |aba
          |caba
        ";

        let meta = FileConfig::default().construct(TEXT);

        assert_eq!(meta.content(), "aba\ncaba\n");
        assert_eq!(meta.cursors(), vec![]);
        assert_eq!(meta.locations(), vec![]);
    }
}

#[test]
fn with_cursors() {
    {
        const TEXT: &str = r"
          |a|ba
        ";

        let meta = FileConfig::default().construct(TEXT);

        assert_eq!(meta.content(), "aba\n");
        assert_eq!(meta.cursors(), vec![Position::new(0, 1)]);
        assert_eq!(meta.locations(), vec![]);
    }

    {
        const TEXT: &str = r"
          |aba|
        ";

        let meta = FileConfig::default().construct(TEXT);

        assert_eq!(meta.content(), "aba\n");
        assert_eq!(meta.cursors(), vec![Position::new(0, 3)]);
        assert_eq!(meta.locations(), vec![]);
    }

    {
        const TEXT: &str = r"
          ||aba
        ";

        let meta = FileConfig::default().construct(TEXT);

        assert_eq!(meta.content(), "aba\n");
        assert_eq!(meta.cursors(), vec![Position::new(0, 0)]);
        assert_eq!(meta.locations(), vec![]);
    }

    {
        const TEXT: &str = r"
          |aba
          |c|aba
        ";

        let meta = FileConfig::default().construct(TEXT);

        assert_eq!(meta.content(), "aba\ncaba\n");
        assert_eq!(meta.cursors(), vec![Position::new(1, 1)]);
        assert_eq!(meta.locations(), vec![]);
    }
}

#[test]
fn with_multi_cursors() {
    {
        const TEXT: &str = r"
          ||aba|caba|
        ";

        let meta = FileConfig::default().construct(TEXT);

        assert_eq!(meta.content(), "abacaba\n");
        assert_eq!(
            meta.cursors(),
            vec![
                Position::new(0, 0),
                Position::new(0, 3),
                Position::new(0, 7)
            ]
        );
        assert_eq!(meta.locations(), vec![]);
    }

    {
        const TEXT: &str = r"
          ||abc
          |a|bc
          |ab|c
          |abc|
        ";

        let meta = FileConfig::default().construct(TEXT);

        assert_eq!(meta.content(), "abc\nabc\nabc\nabc\n");
        assert_eq!(
            meta.cursors(),
            vec![
                Position::new(0, 0),
                Position::new(1, 1),
                Position::new(2, 2),
                Position::new(3, 3),
            ]
        );
        assert_eq!(meta.locations(), vec![]);
    }

    {
        const TEXT: &str = r"
          |||abc
          |a||bc||
        ";

        let meta = FileConfig::default().construct(TEXT);

        assert_eq!(meta.content(), "abc\nabc\n");
        assert_eq!(
            meta.cursors(),
            vec![
                Position::new(0, 0),
                Position::new(0, 0),
                Position::new(1, 1),
                Position::new(1, 1),
                Position::new(1, 3),
                Position::new(1, 3),
            ]
        );
        assert_eq!(meta.locations(), vec![]);
    }
}

#[test]
fn with_location() {
    {
        const TEXT: &str = r"
          |aba
          |^^^  <---
        ";

        let meta = FileConfig::default().construct(TEXT);

        assert_eq!(meta.content(), "aba\n");
        assert_eq!(meta.cursors(), vec![]);
        assert_eq!(
            meta.locations(),
            vec![Range::new(Position::new(0, 0), Position::new(0, 3))]
        );
    }

    {
        const TEXT: &str = r"
          |abacaba
          |^^^ ^^^  <---
        ";

        let meta = FileConfig::default().construct(TEXT);

        assert_eq!(meta.content(), "abacaba\n");
        assert_eq!(meta.cursors(), vec![]);
        assert_eq!(
            meta.locations(),
            vec![
                Range::new(Position::new(0, 0), Position::new(0, 3)),
                Range::new(Position::new(0, 4), Position::new(0, 7))
            ]
        );
    }

    {
        const TEXT: &str = r"
          |aba
          |^^^   <---
          |caba
          | ^^^  <---
        ";

        let meta = FileConfig::default().construct(TEXT);

        assert_eq!(meta.content(), "aba\ncaba\n");
        assert_eq!(meta.cursors(), vec![]);
        assert_eq!(
            meta.locations(),
            vec![
                Range::new(Position::new(0, 0), Position::new(0, 3)),
                Range::new(Position::new(1, 1), Position::new(1, 4))
            ]
        );
    }

    {
        const TEXT: &str = r"
          |aba
          |^^^  <---
          | ^^  <---
        ";

        let meta = FileConfig::default().construct(TEXT);

        assert_eq!(meta.content(), "aba\n");
        assert_eq!(meta.cursors(), vec![]);
        assert_eq!(
            meta.locations(),
            vec![
                Range::new(Position::new(0, 0), Position::new(0, 3)),
                Range::new(Position::new(0, 1), Position::new(0, 3))
            ]
        );
    }
}

#[test]
fn with_cursors_location() {
    {
        const TEXT: &str = r"
          |aba|
          |^^^   <---
        ";

        let meta = FileConfig::default().construct(TEXT);

        assert_eq!(meta.content(), "aba\n");
        assert_eq!(meta.cursors(), vec![Position::new(0, 3)]);
        assert_eq!(
            meta.locations(),
            vec![Range::new(Position::new(0, 0), Position::new(0, 3))]
        );
    }

    {
        const TEXT: &str = r"
          |a|ba
          |^^^^  <---
        ";

        let meta = FileConfig::default().construct(TEXT);

        assert_eq!(meta.content(), "aba\n");
        assert_eq!(meta.cursors(), vec![Position::new(0, 1)]);
        assert_eq!(
            meta.locations(),
            vec![Range::new(Position::new(0, 0), Position::new(0, 3))]
        );
    }

    {
        const TEXT: &str = r"
          ||aba
          | ^^^  <---
        ";

        let meta = FileConfig::default().construct(TEXT);

        assert_eq!(meta.content(), "aba\n");
        assert_eq!(meta.cursors(), vec![Position::new(0, 0)]);
        assert_eq!(
            meta.locations(),
            vec![Range::new(Position::new(0, 0), Position::new(0, 3))]
        );
    }
}

#[test]
fn with_multi_cursors_location() {
    {
        const TEXT: &str = r"
          |||aba||
          |  ^^^    <---
        ";

        let meta = FileConfig::default().construct(TEXT);

        assert_eq!(meta.content(), "aba\n");
        assert_eq!(
            meta.cursors(),
            vec![
                Position::new(0, 0),
                Position::new(0, 0),
                Position::new(0, 3),
                Position::new(0, 3)
            ]
        );
        assert_eq!(
            meta.locations(),
            vec![Range::new(Position::new(0, 0), Position::new(0, 3))]
        );
    }

    {
        const TEXT: &str = r"
          |a|b|a
          |^^^^^  <---
        ";

        let meta = FileConfig::default().construct(TEXT);

        assert_eq!(meta.content(), "aba\n");
        assert_eq!(
            meta.cursors(),
            vec![Position::new(0, 1), Position::new(0, 2)]
        );
        assert_eq!(
            meta.locations(),
            vec![Range::new(Position::new(0, 0), Position::new(0, 3))]
        );
    }

    {
        const TEXT: &str = r"
          |a|b|a
          |^^^^^  <---
          | ^^^^  <---
          |  ^^^  <---
          |   ^^  <---
          |    ^  <---
        ";

        let meta = FileConfig::default().construct(TEXT);

        assert_eq!(meta.content(), "aba\n");
        assert_eq!(
            meta.cursors(),
            vec![Position::new(0, 1), Position::new(0, 2)]
        );
        assert_eq!(
            meta.locations(),
            vec![
                Range::new(Position::new(0, 0), Position::new(0, 3)),
                Range::new(Position::new(0, 1), Position::new(0, 3)),
                Range::new(Position::new(0, 1), Position::new(0, 3)),
                Range::new(Position::new(0, 2), Position::new(0, 3)),
                Range::new(Position::new(0, 2), Position::new(0, 3))
            ]
        );
    }
}
