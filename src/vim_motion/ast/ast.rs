use super::token::{Token, TokenKind};

#[derive(Debug, Default)]
pub struct Ast {
    pub tokens: Vec<Token>,
}

impl Ast {
    pub fn new() -> Self {
        Self { tokens: vec![] }
    }

    pub fn parse(mut self) -> Ast {
        let tokens = self.parse_until(None);

        Ast { tokens }
    }

    fn parse_until(&mut self, expected: Option<TokenKind>) -> Vec<Token> {
        let mut result = Vec::new();

        while !self.tokens.is_empty() {
            let mut token = self.tokens.remove(0);

            // Closing token for current parent
            if let Some(expected) = &expected {
                if token.kind.matches_close(expected) {
                    // Put it back so the parent can consume it
                    self.tokens.insert(0, token);
                    break;
                }
            }

            // Opening container
            if let Some(close_kind) = token.kind.expected_close() {
                token.children = self.parse_until(Some(close_kind));

                // Consume its closing token and store it on the opener
                if !self.tokens.is_empty() {
                    let close = self.tokens.remove(0);
                    token.range.end = close.range.end;
                    token.close = Some(Box::new(close));
                }

                result.push(token);
                continue;
            }

            // Group consecutive chars into a word
            if token.kind == TokenKind::Char {
                let start = token.range.start;
                let mut end = token.range.end;
                let mut chars = vec![token];

                while matches!(self.tokens.first(), Some(next) if next.kind == TokenKind::Char) {
                    let next = self.tokens.remove(0);
                    end = next.range.end;
                    chars.push(next);
                }

                let mut word = Token::new(TokenKind::Word, start..end);
                word.children = chars;
                result.push(word);
                continue;
            }

            result.push(token);
        }

        result
    }
}
