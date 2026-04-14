use chumsky::extra::ParserExtra;
use chumsky::input::ValueInput;
use chumsky::label::LabelError;
use chumsky::prelude::*;

use super::Label::{self, *};
use super::{Child, Token};
use crate::location::Location;
use crate::location::Offset;

use super::parser_utils::MapToken;

/// Token list of whitespace consumption
#[derive(Clone)]
#[allow(clippy::struct_excessive_bools, reason = "Multiple bools are ok here")]
pub(super) struct WhiteSpaceConsumption {
    /// consume regular spaces.
    consume_space: bool,
    /// consume new lines.
    consume_newline: bool,
    /// consume block comments.
    consume_block_comment: bool,
    /// consume line comments.
    consume_line_comment: bool,
    /// consume error tokens.
    consume_error: bool,
}

impl WhiteSpaceConsumption {
    fn new() -> Self {
        WhiteSpaceConsumption {
            consume_space: true,
            consume_newline: true,
            consume_block_comment: true,
            consume_line_comment: true,
            consume_error: true,
        }
    }

    fn with_no_new_line(mut self) -> Self {
        self.consume_newline = false;
        self
    }

    fn ws_parser<'src, I, E>(self) -> impl Parser<'src, I, Child, E> + Clone
    where
        I: ValueInput<'src, Span = Location<Offset>, Token = Token>,
        E: ParserExtra<'src, I> + 'src,
        E::Error: LabelError<'src, I, Label>,
    {
        let space = just(Token::Space).map_token().labelled(Space);

        let new_line = just(Token::NewLine).map_token().labelled(NewLine);

        let error = just(Token::Err).map_token().labelled(Error);

        let mut ws = any().filter(|_| false).map(|_| unreachable!()).boxed();
        if self.consume_space {
            ws = space.or(ws).boxed();
        }
        if self.consume_newline {
            ws = new_line.or(ws).boxed();
        }
        if self.consume_error {
            ws = error.or(ws).boxed();
        }

        ws
    }

    fn comment_parser<'src, I, E>(self) -> impl Parser<'src, I, Child, E> + Clone
    where
        I: ValueInput<'src, Span = Location<Offset>, Token = Token>,
        E: ParserExtra<'src, I> + 'src,
        E::Error: LabelError<'src, I, Label>,
    {
        let line_comment = select! {
            Token::LineComment(comment) => Token::LineComment(comment)
        }
        .map_token()
        .labelled(Comment);

        let block_comment = select! {
            Token::BlockComment(comment) => Token::BlockComment(comment)
        }
        .map_token()
        .labelled(Comment);

        let mut comment_parser = any().filter(|_| false).map(|_| unreachable!()).boxed();
        if self.consume_block_comment {
            comment_parser = block_comment.or(comment_parser).boxed();
        }
        if self.consume_line_comment {
            comment_parser = line_comment.or(comment_parser).boxed();
        }

        comment_parser
    }
}

/// Trait of different strategies of comment consumption
pub(super) trait CommentConsumption: Sized {
    fn parse<'src, I, E>(
        self,
        token_consumption: WhiteSpaceConsumption,
    ) -> impl Parser<'src, I, Vec<Child>, E> + Clone
    where
        I: ValueInput<'src, Span = Location<Offset>, Token = Token>,
        E: ParserExtra<'src, I> + 'src,
        E::Error: LabelError<'src, I, Label>;
}

/// Consume any number of any comments
pub(super) struct BasicCommentConsumption {}

impl CommentConsumption for BasicCommentConsumption {
    fn parse<'src, I, E>(
        self,
        token_consumption: WhiteSpaceConsumption,
    ) -> impl Parser<'src, I, Vec<Child>, E> + Clone
    where
        I: ValueInput<'src, Span = Location<Offset>, Token = Token>,
        E: ParserExtra<'src, I> + 'src,
        E::Error: LabelError<'src, I, Label>,
    {
        let comment = token_consumption.clone().comment_parser();
        let ws = token_consumption.ws_parser();

        choice((comment, ws)).repeated().collect::<Vec<_>>()
    }
}

/// Consume zero comments
pub(super) struct NoCommentConsumption {}

impl CommentConsumption for NoCommentConsumption {
    fn parse<'src, I, E>(
        self,
        token_consumption: WhiteSpaceConsumption,
    ) -> impl Parser<'src, I, Vec<Child>, E> + Clone
    where
        I: ValueInput<'src, Span = Location<Offset>, Token = Token>,
        E: ParserExtra<'src, I> + 'src,
        E::Error: LabelError<'src, I, Label>,
    {
        token_consumption.ws_parser().repeated().collect()
    }
}

/// Consume no more that one comment
pub(super) struct LimitedCommentConsumption {}

impl CommentConsumption for LimitedCommentConsumption {
    fn parse<'src, I, E>(
        self,
        token_consumption: WhiteSpaceConsumption,
    ) -> impl Parser<'src, I, Vec<Child>, E> + Clone
    where
        I: ValueInput<'src, Span = Location<Offset>, Token = Token>,
        E: ParserExtra<'src, I> + 'src,
        E::Error: LabelError<'src, I, Label>,
    {
        let comment = token_consumption.clone().comment_parser();
        let ws = token_consumption.ws_parser();
        let ws_many = ws.repeated().collect::<Vec<_>>();

        ws_many
            .clone()
            .then(comment.or_not())
            .then(ws_many.clone())
            .map(|((mut ws1, comm), ws2)| {
                ws1.extend(comm);
                ws1.extend(ws2);
                ws1
            })
    }
}

/// Consume only binding comments (i.e. definition comments)
pub(super) struct BindedCommentConsumption {}

impl CommentConsumption for BindedCommentConsumption {
    fn parse<'src, I, E>(
        self,
        token_consumption: WhiteSpaceConsumption,
    ) -> impl Parser<'src, I, Vec<Child>, E> + Clone
    where
        I: ValueInput<'src, Span = Location<Offset>, Token = Token>,
        E: ParserExtra<'src, I> + 'src,
        E::Error: LabelError<'src, I, Label>,
    {
        let comment = token_consumption.clone().comment_parser();
        let ws = token_consumption.with_no_new_line().ws_parser();
        let ws_many = ws.repeated().collect::<Vec<_>>();
        let new_line = WhiteSpaceConsumption {
            consume_space: false,
            consume_newline: true,
            consume_block_comment: false,
            consume_line_comment: false,
            consume_error: false,
        }
        .ws_parser();

        comment
            .then(ws_many.clone())
            .then(new_line.or_not())
            .then(ws_many)
            .map(|(((comm, ws1), nl), ws2)| {
                let mut ans = vec![comm];
                ans.extend(ws1);
                ans.extend(nl);
                ans.extend(ws2);
                ans
            })
    }
}

/// Configuration for whitespace parser.
#[allow(
    clippy::struct_excessive_bools,
    reason = "That not a state machine and all states are correct"
)]
pub(super) struct WhiteSpaceConfig<CommentConfig> {
    /// config of white space token consumption
    token_consumption: WhiteSpaceConsumption,
    /// require at least one success token.
    progress: bool,
    /// limit comment token ammount
    comment_config: CommentConfig,
}

pub type WhiteSpace = WhiteSpaceConfig<BasicCommentConsumption>;

impl<CommentConfig> WhiteSpaceConfig<CommentConfig> {
    /// new whitespace config with default params.
    pub(super) fn new() -> WhiteSpaceConfig<BasicCommentConsumption> {
        WhiteSpaceConfig {
            token_consumption: WhiteSpaceConsumption::new(),
            progress: false,
            comment_config: BasicCommentConsumption {},
        }
    }

    /// forces progress of parser (at least one token on exit).
    pub(super) fn with_progress(mut self) -> Self {
        self.progress = true;
        self
    }

    /// stop new line token consumption.
    pub(super) fn with_no_new_line(mut self) -> Self {
        self.token_consumption = self.token_consumption.with_no_new_line();
        self
    }
}

impl WhiteSpaceConfig<BasicCommentConsumption> {
    /// consume no comment.
    pub(super) fn with_no_comment(self) -> WhiteSpaceConfig<NoCommentConsumption> {
        WhiteSpaceConfig {
            token_consumption: self.token_consumption,
            progress: self.progress,
            comment_config: NoCommentConsumption {},
        }
    }

    /// consume no more that one comment.
    pub(super) fn with_limit_comment(self) -> WhiteSpaceConfig<LimitedCommentConsumption> {
        WhiteSpaceConfig {
            token_consumption: self.token_consumption,
            progress: self.progress,
            comment_config: LimitedCommentConsumption {},
        }
    }
    /// consume binded comment.
    pub(super) fn with_bind_comment(self) -> WhiteSpaceConfig<BindedCommentConsumption> {
        WhiteSpaceConfig {
            token_consumption: self.token_consumption,
            progress: self.progress,
            comment_config: BindedCommentConsumption {},
        }
    }
}

impl<CommentConfig: CommentConsumption + 'static> WhiteSpaceConfig<CommentConfig> {
    pub(super) fn parser<'src, I, E>(self) -> impl Parser<'src, I, Vec<Child>, E> + Clone
    where
        I: ValueInput<'src, Span = Location<Offset>, Token = Token>,
        E: ParserExtra<'src, I> + 'src,
        E::Error: LabelError<'src, I, Label>,
    {
        let parser = self.comment_config.parse(self.token_consumption);

        if self.progress {
            parser.filter(|v| !v.is_empty()).boxed()
        } else {
            parser.boxed()
        }
    }
}
