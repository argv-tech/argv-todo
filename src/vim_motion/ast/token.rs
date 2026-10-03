use std::ops::Range;

#[derive(Debug, PartialEq)]
pub enum TokenKind {
    Char,
    Word,
    Whitespace,
    EndOfLine,
    Symbol(SymbolKind),
}

#[derive(Debug, PartialEq)]
pub enum Side {
    Open,
    Close,
}

#[derive(Debug, PartialEq)]
pub enum SymbolKind {
    Paren(Side),
    Brace(Side),
    Square(Side),
    DoubleQuote(Side),
    SingleQuote(Side),
    Punctuation,
}

impl TokenKind {
    pub fn expected_close(&self) -> Option<TokenKind> {
        match self {
            TokenKind::Symbol(SymbolKind::Paren(Side::Open)) => {
                Some(TokenKind::Symbol(SymbolKind::Paren(Side::Close)))
            }

            TokenKind::Symbol(SymbolKind::Brace(Side::Open)) => {
                Some(TokenKind::Symbol(SymbolKind::Brace(Side::Close)))
            }

            TokenKind::Symbol(SymbolKind::Square(Side::Open)) => {
                Some(TokenKind::Symbol(SymbolKind::Square(Side::Close)))
            }

            TokenKind::Symbol(SymbolKind::DoubleQuote(Side::Open)) => {
                Some(TokenKind::Symbol(SymbolKind::DoubleQuote(Side::Close)))
            }

            TokenKind::Symbol(SymbolKind::SingleQuote(Side::Open)) => {
                Some(TokenKind::Symbol(SymbolKind::SingleQuote(Side::Close)))
            }

            _ => None,
        }
    }

    pub fn matches_close(&self, expected: &TokenKind) -> bool {
        self == expected
    }
}

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub range: Range<u16>,
    pub children: Vec<Token>,
    pub close: Option<Box<Token>>,
}

impl Token {
    pub fn new(kind: TokenKind, range: Range<u16>) -> Self {
        Self {
            kind,
            range,
            children: vec![],
            close: None,
        }
    }
}
