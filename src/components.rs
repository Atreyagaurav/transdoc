use crate::{
    errors::{MatchErr, MatchRes, ParseErrorType},
    tokenizer::{Token, TokenList, TokenType},
};
use nom::{
    branch::alt,
    combinator::value,
    multi::{many0, many1},
    sequence::{preceded, terminated}, Parser,
};

pub fn string_val(inp: TokenList<'_>) -> MatchRes<'_, String> {
    many1(alt((character, space)))
        .map(|ch| {
            let mut res = String::with_capacity(ch.len());
            for c in ch {
                res.push_str(c.content);
            }
            res
        })
        .parse(inp)
}

macro_rules! one_token {
    ($name:ident, $ty:pat) => {
        pub fn $name(inp: TokenList<'_>) -> MatchRes<'_, &Token<'_>> {
            match inp.internal() {
                [first, rest @ ..] => match first.ty {
                    $ty => Ok((TokenList::new(rest), first)),
                    _ => Err(nom::Err::Error(
                        MatchErr::new(inp).ty(&ParseErrorType::TokenMismatch(first.ty)),
                    )),
                },
                [] => Err(nom::Err::Error(
                    MatchErr::new(inp).ty(&ParseErrorType::Incomplete),
                )),
            }
        }
    };
}

one_token!(newline, TokenType::NewLine);
one_token!(space, TokenType::WhiteSpace);
one_token!(comment, TokenType::Comment);
one_token!(angle_start, TokenType::AngleStart);
one_token!(angle_end, TokenType::AngleEnd);
one_token!(at, TokenType::At);
one_token!(equal, TokenType::Equal);
one_token!(semicolon, TokenType::Semicolon);
one_token!(dash, TokenType::Dash);
one_token!(character, TokenType::Char);

/// Matches the next one that might have spaces before it
pub fn err_ctx<'a, O, F>(
    ty: &'static ParseErrorType,
    mut f: F,
) -> impl Parser<TokenList<'a>, Output = O, Error = MatchErr<'a>>
where
    F: nom::Parser<TokenList<'a>, Output = O, Error = MatchErr<'a>>,
{
    move |i: TokenList<'a>| match f.parse(i) {
        Ok(o) => Ok(o),
        Err(nom::Err::Incomplete(i)) => Err(nom::Err::Incomplete(i)),
        Err(nom::Err::Error(e)) => Err(nom::Err::Error(e.ty(ty))),
        Err(nom::Err::Failure(e)) => Err(nom::Err::Failure(e.ty(ty))),
    }
}

/// Matches the next one that might have spaces before it
pub fn maybe_space<'a, O, F>(f: F) -> impl Parser<TokenList<'a>, Output = O, Error = MatchErr<'a>>
where
    F: nom::Parser<TokenList<'a>, Output = O, Error = MatchErr<'a>>,
{
    preceded(many0(space), f)
}

/// Matches the next one that might have spaces before it
pub fn after_space<'a, O, F>(f: F) -> impl Parser<TokenList<'a>, Output = O, Error = MatchErr<'a>>
where
    F: nom::Parser<TokenList<'a>, Output = O, Error = MatchErr<'a>>,
{
    preceded(many1(space), f)
}

/// Matches the next one that might have spaces, newlines or comments before it
pub fn maybe_newline<'a, O, F>(f: F) -> impl Parser<TokenList<'a>, Output = O, Error = MatchErr<'a>>
where
    F: nom::Parser<TokenList<'a>, Output = O, Error = MatchErr<'a>>,
{
    preceded(many0_newlines, f)
}

/// Matches the next one that might have spaces, newlines or comments before it
pub fn trailing_newlines<'a, O, F>(
    f: F,
) -> impl Parser<TokenList<'a>, Output = O, Error = MatchErr<'a>>
where
    F: nom::Parser<TokenList<'a>, Output = O, Error = MatchErr<'a>>,
{
    terminated(f, many0_newlines)
}

/// Matches the next one that might have spaces, newlines or comments before it
pub fn newline_terminated<'a, O, F>(
    f: F,
) -> impl Parser<TokenList<'a>, Output = O, Error = MatchErr<'a>>
where
    F: nom::Parser<TokenList<'a>, Output = O, Error = MatchErr<'a>>,
{
    terminated(f, many1_newlines)
}

pub fn many0_newlines(inp: TokenList<'_>) -> MatchRes<'_, ()> {
    value((), many0(alt((space, newline, comment)))).parse(inp)
}

pub fn many1_newlines(inp: TokenList<'_>) -> MatchRes<'_, ()> {
    value((), many1(maybe_space(alt((newline, comment))))).parse(inp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::{get_tokens, TokenList};
    use rstest::rstest;

    #[rstest] // newline
    #[case("my name is")]
    #[should_panic]
    #[case("@what the hell")]
    #[case("何か?")]
    #[case("यो काम गर्छ र")]
    fn string_val_test(#[case] txt: &str) {
        let tk = get_tokens(txt);
        let (rest, n) = string_val(TokenList::new(&tk)).unwrap();
        assert_eq!(rest, TokenList::new(&[]));
        assert_eq!(n, txt);
    }
}
