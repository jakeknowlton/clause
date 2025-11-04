use crate::frontend::token::{Token, TokenKind};
use crate::error::{LexerError, LexerErrorKind, Span};

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
                tokens.push(Token::new(
                    TokenKind::Eof,
                    String::new(),
                    self.line,
                    self.column,
                ));
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
            '[' => Ok(self.make_token(TokenKind::LBracket, 1, start_line, start_column)),
            ']' => Ok(self.make_token(TokenKind::RBracket, 1, start_line, start_column)),
            ':' => Ok(self.make_token(TokenKind::Colon, 1, start_line, start_column)),
            ',' => Ok(self.make_token(TokenKind::Comma, 1, start_line, start_column)),

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

            _ => Err(LexerError::new(
                LexerErrorKind::UnexpectedCharacter,
                format!("Unexpected character: '{}'", ch),
            )
            .with_span(Span::new(start_line, start_column, 1))),
        }
    }

    fn lex_less_than(&mut self, line: usize, column: usize) -> Result<Token, LexerError> {
        self.advance(); // consume '<'

        if self.match_char('<') {
            if self.match_char('=') {
                Ok(Token::new(
                    TokenKind::LShiftAssign,
                    "<<=".to_string(),
                    line,
                    column,
                ))
            } else {
                Ok(Token::new(
                    TokenKind::LShift,
                    "<<".to_string(),
                    line,
                    column,
                ))
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
                Ok(Token::new(
                    TokenKind::RShiftAssign,
                    ">>=".to_string(),
                    line,
                    column,
                ))
            } else {
                Ok(Token::new(
                    TokenKind::RShift,
                    ">>".to_string(),
                    line,
                    column,
                ))
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
            Ok(Token::new(
                TokenKind::PlusAssign,
                "+=".to_string(),
                line,
                column,
            ))
        } else {
            Ok(Token::new(TokenKind::Plus, "+".to_string(), line, column))
        }
    }

    fn lex_minus(&mut self, line: usize, column: usize) -> Result<Token, LexerError> {
        self.advance(); // consume '-'

        if self.match_char('=') {
            Ok(Token::new(
                TokenKind::MinusAssign,
                "-=".to_string(),
                line,
                column,
            ))
        } else {
            Ok(Token::new(TokenKind::Minus, "-".to_string(), line, column))
        }
    }

    fn lex_asterisk(&mut self, line: usize, column: usize) -> Result<Token, LexerError> {
        self.advance(); // consume '*'

        if self.match_char('=') {
            Ok(Token::new(
                TokenKind::MultAssign,
                "*=".to_string(),
                line,
                column,
            ))
        } else {
            Ok(Token::new(TokenKind::Mult, "*".to_string(), line, column))
        }
    }

    fn lex_slash(&mut self, line: usize, column: usize) -> Result<Token, LexerError> {
        self.advance(); // consume '/'

        if self.match_char('=') {
            Ok(Token::new(
                TokenKind::DivAssign,
                "/=".to_string(),
                line,
                column,
            ))
        } else {
            Ok(Token::new(TokenKind::Div, "/".to_string(), line, column))
        }
    }

    fn lex_percent(&mut self, line: usize, column: usize) -> Result<Token, LexerError> {
        self.advance(); // consume '%'

        if self.match_char('=') {
            Ok(Token::new(
                TokenKind::ModAssign,
                "%=".to_string(),
                line,
                column,
            ))
        } else {
            Ok(Token::new(TokenKind::Mod, "%".to_string(), line, column))
        }
    }

    fn lex_ampersand(&mut self, line: usize, column: usize) -> Result<Token, LexerError> {
        self.advance(); // consume '&'

        if self.match_char('&') {
            Ok(Token::new(TokenKind::LAnd, "&&".to_string(), line, column))
        } else if self.match_char('=') {
            Ok(Token::new(
                TokenKind::AndAssign,
                "&=".to_string(),
                line,
                column,
            ))
        } else {
            Ok(Token::new(TokenKind::And, "&".to_string(), line, column))
        }
    }

    fn lex_pipe(&mut self, line: usize, column: usize) -> Result<Token, LexerError> {
        self.advance(); // consume '|'

        if self.match_char('|') {
            Ok(Token::new(TokenKind::LOr, "||".to_string(), line, column))
        } else if self.match_char('=') {
            Ok(Token::new(
                TokenKind::OrAssign,
                "|=".to_string(),
                line,
                column,
            ))
        } else {
            Ok(Token::new(TokenKind::Or, "|".to_string(), line, column))
        }
    }

    fn lex_caret(&mut self, line: usize, column: usize) -> Result<Token, LexerError> {
        self.advance(); // consume '^'

        if self.match_char('=') {
            Ok(Token::new(
                TokenKind::XorAssign,
                "^=".to_string(),
                line,
                column,
            ))
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
        while self.current_char().is_ascii_digit() || self.current_char() == '_' {
            self.advance();
        }

        // Check for float
        if self.current_char() == '.' && self.peek_ahead(1).map_or(false, |c| c.is_ascii_digit()) {
            self.advance(); // consume '.'
            while self.current_char().is_ascii_digit() || self.current_char() == '_' {
                self.advance();
            }

            // Check for exponent
            if matches!(self.current_char(), 'e' | 'E') {
                self.lex_exponent()?;
            }

            self.validate_literal_end("float")?;

            let lexeme: String = self.input[start_pos..self.position].iter().collect();
            return Ok(Token::new(TokenKind::FloatLiteral, lexeme, line, column));
        }

        // Check for exponent (makes it a float)
        if matches!(self.current_char(), 'e' | 'E') {
            self.lex_exponent()?;
            self.validate_literal_end("float")?;

            let lexeme: String = self.input[start_pos..self.position].iter().collect();
            return Ok(Token::new(TokenKind::FloatLiteral, lexeme, line, column));
        }

        self.validate_literal_end("decimal")?;

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
            return Err(LexerError::new(
                LexerErrorKind::InvalidNumber,
                "Expected digit after exponent".to_string(),
            )
            .with_span(Span::new(self.line, self.column, 1)));
        }

        while self.current_char().is_ascii_digit() || self.current_char() == '_' {
            self.advance();
        }

        Ok(())
    }

    fn lex_hex_literal(&mut self, line: usize, column: usize) -> Result<Token, LexerError> {
        let start_pos = self.position;
        self.advance(); // consume '0'
        self.advance(); // consume 'x' or 'X'

        if !self.current_char().is_ascii_hexdigit() {
            return Err(LexerError::new(
                LexerErrorKind::InvalidNumber,
                "Expected hexadecimal digit after '0x'".to_string(),
            )
            .with_span(Span::new(line, column, 2)));
        }

        while self.current_char().is_ascii_hexdigit() || self.current_char() == '_' {
            self.advance();
        }

        self.validate_literal_end("hexadecimal")?;

        let lexeme: String = self.input[start_pos..self.position].iter().collect();
        Ok(Token::new(TokenKind::IntegerLiteral, lexeme, line, column))
    }

    fn lex_binary_literal(&mut self, line: usize, column: usize) -> Result<Token, LexerError> {
        let start_pos = self.position;
        self.advance(); // consume '0'
        self.advance(); // consume 'b' or 'B'

        if !matches!(self.current_char(), '0' | '1') {
            return Err(LexerError::new(
                LexerErrorKind::InvalidNumber,
                "Expected binary digit after '0b'".to_string(),
            )
            .with_span(Span::new(line, column, 2)));
        }

        while matches!(self.current_char(), '0' | '1' | '_') {
            self.advance();
        }

        self.validate_literal_end("binary")?;

        let lexeme: String = self.input[start_pos..self.position].iter().collect();
        Ok(Token::new(TokenKind::IntegerLiteral, lexeme, line, column))
    }

    fn lex_octal_literal(&mut self, line: usize, column: usize) -> Result<Token, LexerError> {
        let start_pos = self.position;
        self.advance(); // consume '0'
        self.advance(); // consume 'o' or 'O'

        if !matches!(self.current_char(), '0'..='7') {
            return Err(LexerError::new(
                LexerErrorKind::InvalidNumber,
                "Expected octal digit after '0o'".to_string(),
            )
            .with_span(Span::new(line, column, 2)));
        }

        while matches!(self.current_char(), '0'..='7' | '_') {
            self.advance();
        }

        self.validate_literal_end("octal")?;

        let lexeme: String = self.input[start_pos..self.position].iter().collect();
        Ok(Token::new(TokenKind::IntegerLiteral, lexeme, line, column))
    }

    fn lex_identifier_or_keyword(
        &mut self,
        line: usize,
        column: usize,
    ) -> Result<Token, LexerError> {
        let start_pos = self.position;

        while self.current_char().is_ascii_alphanumeric() || self.current_char() == '_' {
            self.advance();
        }

        let lexeme: String = self.input[start_pos..self.position].iter().collect();

        let kind = match lexeme.as_str() {
            "let" => TokenKind::Let,
            "fix" => TokenKind::Fix,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "while" => TokenKind::While,
            "break" => TokenKind::Break,
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

    /// Validates that the current character doesn't make a literal invalid.
    /// Returns an error if the current character is alphanumeric (which would indicate
    /// an attempt to create an invalid identifier starting with a number or a malformed literal).
    fn validate_literal_end(&self, literal_type: &str) -> Result<(), LexerError> {
        if self.current_char().is_ascii_alphanumeric() {
            Err(LexerError::new(
                LexerErrorKind::InvalidNumber,
                format!(
                    "Invalid character '{}' in {} literal",
                    self.current_char(),
                    literal_type
                ),
            )
            .with_span(Span::new(self.line, self.column, 1)))
        } else {
            Ok(())
        }
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

    #[test]
    fn test_invalid_hex_literal() {
        let mut lexer = Lexer::new("0xFG");
        let result = lexer.tokenize();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .message
                .contains("Invalid character 'G' in hexadecimal literal")
        );
    }

    #[test]
    fn test_invalid_binary_literal() {
        let mut lexer = Lexer::new("0b102");
        let result = lexer.tokenize();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .message
                .contains("Invalid character '2' in binary literal")
        );
    }

    #[test]
    fn test_invalid_octal_literal() {
        let mut lexer = Lexer::new("0o789");
        let result = lexer.tokenize();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .message
                .contains("Invalid character '8' in octal literal")
        );
    }

    #[test]
    fn test_valid_literals_with_operators() {
        // Make sure valid literals followed by operators still work
        let mut lexer = Lexer::new("0xFF+0b1010");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::IntegerLiteral);
        assert_eq!(tokens[1].kind, TokenKind::Plus);
        assert_eq!(tokens[2].kind, TokenKind::IntegerLiteral);
    }

    #[test]
    fn test_decimal_with_underscores() {
        let mut lexer = Lexer::new("1_000_000");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::IntegerLiteral);
        assert_eq!(tokens[0].lexeme, "1_000_000");
    }

    #[test]
    fn test_hex_with_underscores() {
        let mut lexer = Lexer::new("0xFF_AB_CD");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::IntegerLiteral);
        assert_eq!(tokens[0].lexeme, "0xFF_AB_CD");
    }

    #[test]
    fn test_binary_with_underscores() {
        let mut lexer = Lexer::new("0b1111_0000");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::IntegerLiteral);
        assert_eq!(tokens[0].lexeme, "0b1111_0000");
    }

    #[test]
    fn test_octal_with_underscores() {
        let mut lexer = Lexer::new("0o755_644");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::IntegerLiteral);
        assert_eq!(tokens[0].lexeme, "0o755_644");
    }

    #[test]
    fn test_float_with_underscores() {
        let mut lexer = Lexer::new("3.141_592 1_000.5e1_0");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::FloatLiteral);
        assert_eq!(tokens[0].lexeme, "3.141_592");
        assert_eq!(tokens[1].kind, TokenKind::FloatLiteral);
        assert_eq!(tokens[1].lexeme, "1_000.5e1_0");
    }

    #[test]
    fn test_hex_with_underscore_followed_by_invalid() {
        // 0xFF_G should error on the G, not treat _ as part of the number
        let mut lexer = Lexer::new("0xFF_G");
        let result = lexer.tokenize();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .message
                .contains("Invalid character 'G'")
        );
    }

    #[test]
    fn test_invalid_decimal_with_letters() {
        let mut lexer = Lexer::new("12abc");
        let result = lexer.tokenize();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .message
                .contains("Invalid character 'a' in decimal literal")
        );
    }

    #[test]
    fn test_invalid_float_with_letters() {
        let mut lexer = Lexer::new("3.14abc");
        let result = lexer.tokenize();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .message
                .contains("Invalid character 'a' in float literal")
        );
    }

    #[test]
    fn test_invalid_float_exponent_with_letters() {
        let mut lexer = Lexer::new("1e10abc");
        let result = lexer.tokenize();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .message
                .contains("Invalid character 'a' in float literal")
        );
    }

    #[test]
    fn test_valid_number_followed_by_operator() {
        // Make sure numbers followed by operators still work
        let mut lexer = Lexer::new("42+3.14");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::IntegerLiteral);
        assert_eq!(tokens[1].kind, TokenKind::Plus);
        assert_eq!(tokens[2].kind, TokenKind::FloatLiteral);
    }
}
