use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take},
    combinator::{map, recognize},
    multi::{many0, many1},
    sequence::pair,
    IResult, Needed, Parser,
};

#[derive(Clone, PartialEq, Debug, Copy)]
pub struct Token<'a> {
    pub ty: TokenType,
    pub content: &'a str,
}

impl<'a> Token<'a> {
    fn new(ty: TokenType, content: &'a str) -> Self {
        Self { ty, content }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct TokenList<'a>(&'a [Token<'a>]);

impl<'a> std::ops::Deref for TokenList<'a> {
    type Target = &'a [Token<'a>];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> TokenList<'a> {
    pub fn new(val: &'a [Token<'a>]) -> Self {
        Self(val)
    }

    pub fn internal(&self) -> &'a [Token<'a>] {
        self.0
    }
}

impl<'a> nom::Input for TokenList<'a> {
    type Item = Token<'a>;
    type Iter = std::iter::Copied<std::slice::Iter<'a, Token<'a>>>;
    type IterIndices = std::iter::Enumerate<Self::Iter>;

    fn input_len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    fn take(&self, index: usize) -> Self {
        Self(&self.0[0..index])
    }

    fn take_from(&self, index: usize) -> Self {
        Self(&self.0[index..])
    }

    #[inline]
    fn take_split(&self, index: usize) -> (Self, Self) {
        let (prefix, suffix) = self.0.split_at(index);
        (Self(suffix), Self(prefix))
    }

    #[inline]
    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        self.0.iter().position(|b| predicate(*b))
    }

    #[inline]
    fn iter_elements(&self) -> Self::Iter {
        self.0.iter().copied()
    }

    #[inline]
    fn iter_indices(&self) -> Self::IterIndices {
        self.iter_elements().enumerate()
    }

    #[inline]
    fn slice_index(&self, count: usize) -> Result<usize, Needed> {
        if self.0.len() >= count {
            Ok(count)
        } else {
            Err(Needed::new(count - self.0.len()))
        }
    }

    #[inline(always)]
    fn split_at_position<P, E: nom::error::ParseError<Self>>(
        &self,
        predicate: P,
    ) -> IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        match self.0.iter().position(|c| predicate(*c)) {
            Some(i) => Ok(self.take_split(i)),

            None => Err(nom::Err::Incomplete(Needed::new(1))),
        }
    }
}

#[derive(Clone, PartialEq, Debug, Copy)]
pub enum TokenType {
    NewLine,
    WhiteSpace,
    Comment,
    AngleStart,
    AngleEnd,
    At,
    Equal,
    Semicolon,
    Dash,
    Char,
}

pub(crate) type TokenRes<'a> = IResult<&'a str, Token<'a>>;
pub(crate) type VecTokenRes<'a> = IResult<&'a str, Vec<Token<'a>>>;

fn whitespace(i: &str) -> TokenRes<'_> {
    map(recognize(many1(alt((tag("\t"), tag(" "))))), |s| {
        Token::new(TokenType::WhiteSpace, s)
    })
    .parse(i)
}

fn newline(i: &str) -> TokenRes<'_> {
    // only unix, mac and windows line end supported for now
    map(alt((tag("\n\r"), tag("\r\n"), tag("\n"))), |s| {
        Token::new(TokenType::NewLine, s)
    })
    .parse(i)
}

fn comment(i: &str) -> TokenRes<'_> {
    map(recognize(pair(tag("#"), many0(is_not("\n\r")))), |s| {
        Token::new(TokenType::Comment, s)
    })
    .parse(i)
}

fn symbols(i: &str) -> TokenRes<'_> {
    alt((
        map(tag("<<"), |s| Token::new(TokenType::AngleStart, s)),
        map(tag(">>"), |s| Token::new(TokenType::AngleEnd, s)),
        map(tag("@"), |s| Token::new(TokenType::At, s)),
        map(tag("="), |s| Token::new(TokenType::Equal, s)),
        map(tag(";"), |s| Token::new(TokenType::Semicolon, s)),
        map(tag("---"), |s| Token::new(TokenType::Dash, s)),
    ))
    .parse(i)
}

fn known_token(i: &str) -> TokenRes<'_> {
    alt((whitespace, newline, comment, symbols)).parse(i)
}

fn character(i: &str) -> TokenRes<'_> {
    map(take(1usize), |s| Token::new(TokenType::Char, s)).parse(i)
}

fn all_tokens(i: &str) -> VecTokenRes<'_> {
    many0(alt((known_token, character))).parse(i)
}

pub fn get_tokens(txt: &str) -> Vec<Token> {
    let (res, tokens) = all_tokens(txt).expect("Parser shouldn't error out");
    if !res.is_empty() {
        println!("{res:?}");
        panic!("Logic Error on Parser, there shouldn't be anything left")
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest] // whitespace
    #[case(" ", TokenType::WhiteSpace, "")]
    #[case("\t", TokenType::WhiteSpace, "")] // tab whitespace
    #[case("   \n", TokenType::WhiteSpace, "\n")] // multiple spaces
    fn whitespace_test(#[case] txt: &str, #[case] value: TokenType, #[case] reminder: &str) {
        let (rest, n) = whitespace(txt).unwrap();
        assert_eq!(rest, reminder);
        assert_eq!(n.ty, value);
    }

    #[rstest]
    #[case("# comment", TokenType::Comment, "")]
    #[case("# comment\n", TokenType::Comment, "\n")]
    #[case("# comment\n123", TokenType::Comment, "\n123")]
    fn comment_test(#[case] txt: &str, #[case] value: TokenType, #[case] reminder: &str) {
        let (rest, n) = comment(txt).unwrap();
        assert_eq!(rest, reminder);
        assert_eq!(n.ty, value);
    }

    #[rstest] // newline
    #[case("\n", TokenType::NewLine, "")]
    #[should_panic]
    #[case("\\\n", TokenType::NewLine, "")] // escaped newline should be escaped
    #[case("\n   ", TokenType::NewLine, "   ")]
    fn newline_test(#[case] txt: &str, #[case] value: TokenType, #[case] reminder: &str) {
        let (rest, n) = newline(txt).unwrap();
        assert_eq!(rest, reminder);
        assert_eq!(n.ty, value);
    }

    #[rstest] // newline
    #[case("my name is", TokenType::Char, "")]
    #[case("@what the hell", TokenType::At, "")]
    #[case("何か?", TokenType::Char, "")]
    #[case("यो काम गर्छ र", TokenType::Char, "")]
    fn maybe_string_test(#[case] txt: &str, #[case] value: TokenType, #[case] reminder: &str) {
        let (rest, n) = all_tokens(txt).unwrap();
        assert_eq!(rest, reminder);
        assert_eq!(n[0].ty, value);
    }
}
