use super::ast::Ast;
use super::token;

#[derive(Debug)]
pub struct Lexer {
    position: usize,
    double_quote_open: bool,
    single_quote_open: bool,
}

impl Lexer {
    fn new() -> Self {
        Self {
            position: 0,
            double_quote_open: false,
            single_quote_open: false,
        }
    }

    pub fn lex(input: &str) -> Ast {
        let mut ast = Ast::new();
        let mut lexer = Lexer::new();

        while lexer.position < input.len() {
            let ch = input[lexer.position..].chars().next().unwrap();

            if ch == ' ' || ch == '\t' {
                let start = lexer.position;

                while lexer.position < input.len() {
                    let current = input[lexer.position..].chars().next().unwrap();

                    if current != ' ' && current != '\t' {
                        break;
                    }

                    lexer.position += current.len_utf8();
                }

                ast.tokens.push(token::Token::new(
                    token::TokenKind::Whitespace,
                    start as u16..lexer.position as u16,
                ));

                continue;
            }

            let kind = match ch {
                '\n' => token::TokenKind::EndOfLine,

                '(' => token::TokenKind::Symbol(token::SymbolKind::Paren(token::Side::Open)),
                ')' => token::TokenKind::Symbol(token::SymbolKind::Paren(token::Side::Close)),

                '{' => token::TokenKind::Symbol(token::SymbolKind::Brace(token::Side::Open)),
                '}' => token::TokenKind::Symbol(token::SymbolKind::Brace(token::Side::Close)),

                '[' => token::TokenKind::Symbol(token::SymbolKind::Square(token::Side::Open)),
                ']' => token::TokenKind::Symbol(token::SymbolKind::Square(token::Side::Close)),

                '"' => lexer.double_quote(),
                '\'' => lexer.single_quote(),

                c if c.is_alphanumeric() || c == '_' => token::TokenKind::Char,

                _ => token::TokenKind::Symbol(token::SymbolKind::Punctuation),
            };

            let start = lexer.position;
            let end = start + ch.len_utf8();

            ast.tokens
                .push(token::Token::new(kind, start as u16..end as u16));

            lexer.position = end;
        }

        ast.parse()
    }

    fn double_quote(&mut self) -> token::TokenKind {
        let side = if self.double_quote_open {
            token::Side::Close
        } else {
            token::Side::Open
        };

        self.double_quote_open = !self.double_quote_open;

        token::TokenKind::Symbol(token::SymbolKind::DoubleQuote(side))
    }

    fn single_quote(&mut self) -> token::TokenKind {
        let side = if self.single_quote_open {
            token::Side::Close
        } else {
            token::Side::Open
        };

        self.single_quote_open = !self.single_quote_open;

        token::TokenKind::Symbol(token::SymbolKind::SingleQuote(side))
    }
}
