use alloc::vec::Vec;
use core::fmt;
use chumsky::prelude::*;

pub type Span = SimpleSpan;
pub type Spanned<T> = (T, Span);

#[derive(Clone, Debug, PartialEq)]
pub enum Token<'src> {
    Null,
    Str(&'src str),
    Ident(&'src str),
    Def,
    Do,
    End,
    If,
    Else,
}

impl fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Null => write!(f, "null"),
            Token::Str(s) => write!(f, "{s}"),
            Token::Ident(s) => write!(f, "{s}"),
            Token::Def => write!(f, "def"),
            Token::Do => write!(f, "do"),
            Token::End => write!(f, "end"),
            Token::If => write!(f, "if"),
            Token::Else => write!(f, "else"),
        }
    }
}


pub fn lexer<'a>() -> impl Parser<'a, &'a str, Vec<Spanned<Token<'a>>>, extra::Err<Rich<'a, char, Span>>> {
    let string = just('"')
        .ignore_then(none_of('"').repeated().to_slice())
        .then_ignore(just('"'))
        .map(Token::Str);

    let identifier = text::ascii::ident().map(|ident: &str| match ident {
        "def" => Token::Def,
        "do" => Token::Do,
        "end" => Token::End,
        "if" => Token::If,
        "else" => Token::Else,
        "null" => Token::Null,
        _ => Token::Ident(ident),
    });

    let token = string.or(identifier);

    let comment = just('#')
        .then(any().and_is(just('\n').not()).repeated())
        .padded();

    token.map_with(|t, e| (t, e.span()))
        .padded_by(comment.repeated())
        .padded()
        .recover_with(skip_then_retry_until(any().ignored(), end()))
        .repeated()
        .collect()
}
