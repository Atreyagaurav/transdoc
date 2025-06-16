use crate::tokenizer::{TokenList, TokenType};
use nom::{error::ErrorKind, IResult};

#[derive(Debug, PartialEq, Clone, Default)]
pub struct ParseError {
    pub ty: ParseErrorType,
    pub line: usize,
    pub col: usize,
    pub linestr: String,
}
impl ParseError {
    pub fn new(tokens: TokenList<'_>, rest: TokenList<'_>, ty: ParseErrorType) -> Self {
        let tokens = &tokens[..(tokens.len() - rest.len())];
        let mut line = 1;
        let mut lstart = 0;
        for (i, t) in tokens.iter().enumerate() {
            if t.ty == TokenType::NewLine {
                line += 1;
                lstart = i + 1;
            }
        }
        let mut curr_line: Vec<_> = tokens[lstart..].iter().collect();
        let col = curr_line.iter().map(|t| t.content.len()).sum::<usize>() + 1;
        for t in rest.iter() {
            if t.ty == TokenType::NewLine {
                break;
            } else {
                curr_line.push(t);
            }
        }
        let mut linestr = String::new();
        curr_line.iter().for_each(|t| linestr.push_str(t.content));
        Self {
            ty,
            line,
            col,
            linestr,
        }
    }

    pub fn user_msg(&self, filename: Option<&str>) -> String {
        let mut msg = String::new();
        if let ParseErrorType::Custom(m) = &self.ty {
            msg.push_str(m);
            if let Some(fname) = filename {
                msg.push_str(&format!("  -> {fname}\n"));
            }
        } else {
            msg.push_str(&format!(
                "Error: Parse Error at Line {} Column {}\n",
                self.line, self.col
            ));
            if let Some(fname) = filename {
                msg.push_str(&format!("  -> {}:{}:{}\n", fname, self.line, self.col));
            }
            msg.push_str(&format!("  {}\n", self.linestr));
            msg.push_str(&format!("  {: >2$} {}", "^", self.ty.message(), self.col));
        }
        msg
    }
}

pub type MatchRes<'a, T> = IResult<TokenList<'a>, T, MatchErr<'a>>;

impl std::error::Error for ParseError {}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "ParseError: {} at line {} col {}",
            self.ty.message(),
            self.line,
            self.col
        )
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub enum ParseErrorType {
    LogicalError(String),
    Unclosed(String),
    Incomplete,
    #[default]
    SyntaxError,
    TokenMismatch(TokenType),
    Custom(String),
}

impl ParseErrorType {
    pub fn message(&self) -> String {
        match self {
            Self::LogicalError(v) => return format!("LogicalError: {}, please contact dev", v),
            Self::Unclosed(s) => return format!("Unclosed: Missing closing token {s:?}"),
            Self::Incomplete => "Incomplete: Parser ran out of inputs",
            Self::SyntaxError => "SyntaxError: Invalid Syntax",
            Self::TokenMismatch(t) => return format!("TokenMismatch: {t:?} unexpected here"),
            Self::Custom(msg) => msg.as_str(),
        }
        .to_string()
    }
}

#[derive(Debug)]
pub struct MatchErr<'a> {
    pub ty: ParseErrorType,
    pub internal: nom::error::Error<TokenList<'a>>,
}

impl<'a> MatchErr<'a> {
    pub fn new(inp: TokenList<'a>) -> Self {
        MatchErr {
            ty: ParseErrorType::SyntaxError,
            internal: nom::error::Error::new(inp, ErrorKind::Tag),
        }
    }

    pub fn from_nom(internal: nom::error::Error<TokenList<'a>>) -> Self {
        MatchErr {
            ty: ParseErrorType::SyntaxError,
            internal,
        }
    }

    pub fn ty(mut self, ty: &ParseErrorType) -> Self {
        self.ty = ty.clone();
        self
    }
}

impl<'a> nom::error::ParseError<TokenList<'a>> for MatchErr<'a> {
    fn from_error_kind(input: TokenList<'a>, kind: ErrorKind) -> Self {
        MatchErr {
            ty: ParseErrorType::SyntaxError,
            internal: nom::error::Error::<TokenList<'a>>::from_error_kind(input, kind),
        }
    }
    // what does it do?
    fn append(input: TokenList<'a>, kind: ErrorKind, other: Self) -> Self {
        MatchErr {
            ty: other.ty,
            internal: nom::error::Error::<TokenList<'a>>::append(input, kind, other.internal),
        }
    }

    // Provided methods
    fn from_char(input: TokenList<'a>, c: char) -> Self {
        MatchErr {
            ty: ParseErrorType::SyntaxError,
            internal: nom::error::Error::<TokenList<'a>>::from_char(input, c),
        }
    }
    fn or(self, other: Self) -> Self {
        MatchErr {
            ty: self.ty,
            internal: nom::error::Error::<TokenList<'a>>::or(other.internal, self.internal),
        }
    }
}
