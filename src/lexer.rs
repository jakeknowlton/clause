use crate::token::{Token, TokenKind};
use std::fmt;

#[derive(Debug)]
pub struct LexerError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Lexer error at {}:{}: {}", self.line, self.column, self.message)
    }
}

impl std::error::Error for LexerError {}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = Vec::new();

        loop {
            self.skip_whitespace_and_comments();

            if self.is_at_end() {
                tokens.push(Token::new(TokenKind::Eof, String::new(), self.line, self.column));
                break;
            }

            let token = self.next_token()?;
            tokens.push(token);
        }

        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Token, LexerError> {
        let start_line = self.line;
        let start_column = self.column;
        let ch = self.current_char();

        match ch {
            // Delimiters
            '(' => Ok(self.make_token(TokenKind::LParen, 1, start_line, start_column)),
            ')' => Ok(self.make_token(TokenKind::RParen, 1, start_line, start_column)),
            '{' => Ok(self.make_token(TokenKind::LBrace, 1, start_line, start_column)),
            '}' => Ok(self.make_token(TokenKind::RBrace, 1, start_line, start_column)),
            ':' => Ok(self.make_token(TokenKind::Colon, 1, start_line, start_column)),

            // Multi-character operators (must check these before single-char ones)
            '<' => self.lex_less_than(start_line, start_column),
            '>' => self.lex_greater_than(start_line, start_column),
            '=' => self.lex_equals(start_line, start_column),
            '!' => self.lex_exclamation(start_line, start_column),
            '+' => self.lex_plus(start_line, start_column),
            '-' => self.lex_minus(start_line, start_column),
            '*' => self.lex_asterisk(start_line, start_column),
            '/' => self.lex_slash(start_line, start_column),
            '%' => self.lex_percent(start_line, start_column),
            '&' => self.lex_ampersand(start_line, start_column),
            '|' => self.lex_pipe(start_line, start_column),
            '^' => self.lex_caret(start_line, start_column),
            '~' => Ok(self.make_token(TokenKind::BNot, 1, start_line, start_column)),

            // Numbers
            '0'..='9' => self.lex_number(start_line, start_column),

            // Identifiers and keywords
            'a'..='z' | 'A'..='Z' | '_' => self.lex_identifier_or_keyword(start_line, start_column),

            _ => Err(LexerError {
                message: format!("Unexpected character: '{}'", ch),
                line: start_line,
                column: start_column,
            }),
        }
    }

    fn lex_less_than(&mut self, line: usize, column: usize) -> Result<Token, LexerError> {
        self.advance(); // consume '<'

        if self.match_char('<') {
            if self.match_char('=') {
                Ok(Token::new(TokenKind::LShiftAssign, "<<=".to_string(), line, column))
            } else {
                Ok(Token::new(TokenKind::LShift, "<<".to_string(), line, column))
            }
        } else if self.match_char('=') {
            Ok(Token::new(TokenKind::Le, "<=".to_string(), line, column))
        } else if self.match_char('-') {
            Ok(Token::new(TokenKind::Yield, "<-".to_string(), line, column))
        } else {
            Ok(Token::new(TokenKind::Lt, "<".to_string(), line, column))
        }
    }

    fn lex_greater_than(&mut self, line: usize, column: usize) -> Result<Token, LexerError> {
        self.advance(); // consume '>'

        if self.match_char('>') {
            if self.match_char('=') {
                Ok(Token::new(TokenKind::RShiftAssign, ">>=".to_string(), line, column))
            } else {
                Ok(Token::new(TokenKind::RShift, ">>".to_string(), line, column))
            }
        } else if self.match_char('=') {
            Ok(Token::new(TokenKind::Ge, ">=".to_string(), line, column))
        } else {
            Ok(Token::new(TokenKind::Gt, ">".to_string(), line, column))
        }
    }

    fn lex_equals(&mut self, line: usize, column: usize) -> Result<Token, LexerError> {
        self.advance(); // consume '='

        if self.match_char('=') {
            Ok(Token::new(TokenKind::Eq, "==".to_string(), line, column))
        } else {
            Ok(Token::new(TokenKind::Assign, "=".to_string(), line, column))
        }
    }

    fn lex_exclamation(&mut self, line: usize, column: usize) -> Result<Token, LexerError> {
        self.advance(); // consume '!'

        if self.match_char('=') {
            Ok(Token::new(TokenKind::Ne, "!=".to_string(), line, column))
        } else {
            Ok(Token::new(TokenKind::Not, "!".to_string(), line, column))
        }
    }

    fn lex_plus(&mut self, line: usize, column: usize) -> Result<Token, LexerError> {
        self.advance(); // consume '+'

        if self.match_char('=') {
            Ok(Token::new(TokenKind::PlusAssign, "+=".to_string(), line, column))
        } else {
            Ok(Token::new(TokenKind::Plus, "+".to_string(), line, column))
        }
    }

    fn lex_minus(&mut self, line: usize, column: usize) -> Result<Token, LexerError> {
        self.advance(); // consume '-'

        if self.match_char('=') {
            Ok(Token::new(TokenKind::MinusAssign, "-=".to_string(), line, column))
        } else {
            Ok(Token::new(TokenKind::Minus, "-".to_string(), line, column))
        }
    }

    fn lex_asterisk(&mut self, line: usize, column: usize) -> Result<Token, LexerError> {
        self.advance(); // consume '*'

        if self.match_char('=') {
            Ok(Token::new(TokenKind::MultAssign, "*=".to_string(), line, column))
        } else {
            Ok(Token::new(TokenKind::Mult, "*".to_string(), line, column))
        }
    }

    fn lex_slash(&mut self, line: usize, column: usize) -> Result<Token, LexerError> {
        self.advance(); // consume '/'

        if self.match_char('=') {
            Ok(Token::new(TokenKind::DivAssign, "/=".to_string(), line, column))
        } else {
            Ok(Token::new(TokenKind::Div, "/".to_string(), line, column))
        }
    }

    fn lex_percent(&mut self, line: usize, column: usize) -> Result<Token, LexerError> {
        self.advance(); // consume '%'

        if self.match_char('=') {
            Ok(Token::new(TokenKind::ModAssign, "%=".to_string(), line, column))
        } else {
            Ok(Token::new(TokenKind::Mod, "%".to_string(), line, column))
        }
    }

    fn lex_ampersand(&mut self, line: usize, column: usize) -> Result<Token, LexerError> {
        self.advance(); // consume '&'

        if self.match_char('&') {
            Ok(Token::new(TokenKind::LAnd, "&&".to_string(), line, column))
        } else if self.match_char('=') {
            Ok(Token::new(TokenKind::AndAssign, "&=".to_string(), line, column))
        } else {
            Ok(Token::new(TokenKind::And, "&".to_string(), line, column))
        }
    }

    fn lex_pipe(&mut self, line: usize, column: usize) -> Result<Token, LexerError> {
        self.advance(); // consume '|'

        if self.match_char('|') {
            Ok(Token::new(TokenKind::LOr, "||".to_string(), line, column))
        } else if self.match_char('=') {
            Ok(Token::new(TokenKind::OrAssign, "|=".to_string(), line, column))
        } else {
            Ok(Token::new(TokenKind::Or, "|".to_string(), line, column))
        }
    }

    fn lex_caret(&mut self, line: usize, column: usize) -> Result<Token, LexerError> {
        self.advance(); // consume '^'

        if self.match_char('=') {
            Ok(Token::new(TokenKind::XorAssign, "^=".to_string(), line, column))
        } else {
            Ok(Token::new(TokenKind::Xor, "^".to_string(), line, column))
        }
    }

    fn lex_number(&mut self, line: usize, column: usize) -> Result<Token, LexerError> {
        let start_pos = self.position;

        // Check for hex, binary, or octal
        if self.current_char() == '0' && self.position + 1 < self.input.len() {
            let next_ch = self.input[self.position + 1];
            match next_ch.to_ascii_lowercase() {
                'x' => return self.lex_hex_literal(line, column),
                'b' => return self.lex_binary_literal(line, column),
                'o' => return self.lex_octal_literal(line, column),
                _ => {}
            }
        }

        // Decimal number (integer or float)
        while self.current_char().is_ascii_digit() {
            self.advance();
        }

        // Check for float
        if self.current_char() == '.' && self.peek_ahead(1).map_or(false, |c| c.is_ascii_digit()) {
            self.advance(); // consume '.'
            while self.current_char().is_ascii_digit() {
                self.advance();
            }

            // Check for exponent
            if matches!(self.current_char(), 'e' | 'E') {
                self.lex_exponent()?;
            }

            let lexeme: String = self.input[start_pos..self.position].iter().collect();
            return Ok(Token::new(TokenKind::FloatLiteral, lexeme, line, column));
        }

        // Check for exponent (makes it a float)
        if matches!(self.current_char(), 'e' | 'E') {
            self.lex_exponent()?;
            let lexeme: String = self.input[start_pos..self.position].iter().collect();
            return Ok(Token::new(TokenKind::FloatLiteral, lexeme, line, column));
        }

        // Integer literal
        let lexeme: String = self.input[start_pos..self.position].iter().collect();
        Ok(Token::new(TokenKind::IntegerLiteral, lexeme, line, column))
    }

    fn lex_exponent(&mut self) -> Result<(), LexerError> {
        self.advance(); // consume 'e' or 'E'

        if matches!(self.current_char(), '+' | '-') {
            self.advance();
        }

        if !self.current_char().is_ascii_digit() {
            return Err(LexerError {
                message: "Expected digit after exponent".to_string(),
                line: self.line,
                column: self.column,
            });
        }

        while self.current_char().is_ascii_digit() {
            self.advance();
        }

        Ok(())
    }

    fn lex_hex_literal(&mut self, line: usize, column: usize) -> Result<Token, LexerError> {
        let start_pos = self.position;
        self.advance(); // consume '0'
        self.advance(); // consume 'x' or 'X'

        if !self.current_char().is_ascii_hexdigit() {
            return Err(LexerError {
                message: "Expected hexadecimal digit after '0x'".to_string(),
                line,
                column,
            });
        }

        while self.current_char().is_ascii_hexdigit() {
            self.advance();
        }

        let lexeme: String = self.input[start_pos..self.position].iter().collect();
        Ok(Token::new(TokenKind::IntegerLiteral, lexeme, line, column))
    }

    fn lex_binary_literal(&mut self, line: usize, column: usize) -> Result<Token, LexerError> {
        let start_pos = self.position;
        self.advance(); // consume '0'
        self.advance(); // consume 'b' or 'B'

        if !matches!(self.current_char(), '0' | '1') {
            return Err(LexerError {
                message: "Expected binary digit after '0b'".to_string(),
                line,
                column,
            });
        }

        while matches!(self.current_char(), '0' | '1') {
            self.advance();
        }

        let lexeme: String = self.input[start_pos..self.position].iter().collect();
        Ok(Token::new(TokenKind::IntegerLiteral, lexeme, line, column))
    }

    fn lex_octal_literal(&mut self, line: usize, column: usize) -> Result<Token, LexerError> {
        let start_pos = self.position;
        self.advance(); // consume '0'
        self.advance(); // consume 'o' or 'O'

        if !matches!(self.current_char(), '0'..='7') {
            return Err(LexerError {
                message: "Expected octal digit after '0o'".to_string(),
                line,
                column,
            });
        }

        while matches!(self.current_char(), '0'..='7') {
            self.advance();
        }

        let lexeme: String = self.input[start_pos..self.position].iter().collect();
        Ok(Token::new(TokenKind::IntegerLiteral, lexeme, line, column))
    }

    fn lex_identifier_or_keyword(&mut self, line: usize, column: usize) -> Result<Token, LexerError> {
        let start_pos = self.position;

        while self.current_char().is_ascii_alphanumeric() || self.current_char() == '_' {
            self.advance();
        }

        let lexeme: String = self.input[start_pos..self.position].iter().collect();

        let kind = match lexeme.as_str() {
            "let" => TokenKind::Let,
            "fix" => TokenKind::Fix,
            "true" | "false" => TokenKind::BooleanLiteral,
            "void" => TokenKind::VoidLiteral,
            _ => TokenKind::Identifier,
        };

        Ok(Token::new(kind, lexeme, line, column))
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            match self.current_char() {
                ' ' | '\t' | '\r' => {
                    self.advance();
                }
                '\n' => {
                    self.line += 1;
                    self.column = 1;
                    self.advance_position();
                }
                '/' if self.peek_ahead(1) == Some('/') => {
                    // Line comment
                    self.advance(); // consume '/'
                    self.advance(); // consume '/'
                    while self.current_char() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                }
                _ => break,
            }
        }
    }

    fn make_token(&mut self, kind: TokenKind, len: usize, line: usize, column: usize) -> Token {
        let start = self.position;
        for _ in 0..len {
            self.advance();
        }
        let lexeme: String = self.input[start..self.position].iter().collect();
        Token::new(kind, lexeme, line, column)
    }

    fn current_char(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.input[self.position]
        }
    }

    fn peek_ahead(&self, offset: usize) -> Option<char> {
        let pos = self.position + offset;
        if pos < self.input.len() {
            Some(self.input[pos])
        } else {
            None
        }
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.current_char() == expected {
            self.advance();
            true
        } else {
            false
        }
    }

    fn advance(&mut self) {
        if !self.is_at_end() {
            self.column += 1;
            self.position += 1;
        }
    }

    fn advance_position(&mut self) {
        if !self.is_at_end() {
            self.position += 1;
        }
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_tokens() {
        let mut lexer = Lexer::new("let x = 42");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::Let);
        assert_eq!(tokens[1].kind, TokenKind::Identifier);
        assert_eq!(tokens[2].kind, TokenKind::Assign);
        assert_eq!(tokens[3].kind, TokenKind::IntegerLiteral);
    }

    #[test]
    fn test_hex_literal() {
        let mut lexer = Lexer::new("0xFF");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::IntegerLiteral);
        assert_eq!(tokens[0].lexeme, "0xFF");
    }

    #[test]
    fn test_float_literal() {
        let mut lexer = Lexer::new("3.14 1.5e10");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::FloatLiteral);
        assert_eq!(tokens[1].kind, TokenKind::FloatLiteral);
    }
}
