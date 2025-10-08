#pragma once

#include <string>
#include <variant>
#include "clause/common/source_location.hpp"

namespace clause {

enum class TokenType {
    // Literals
    Integer,
    Float,
    String,
    Identifier,

    // Keywords
    Let,
    Const,
    Fn,
    Return,
    If,
    Else,
    While,
    For,
    Class,
    Where,
    In,

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Equal,
    EqualEqual,
    Bang,
    BangEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Ampersand,
    And,
    Pipe,
    Or,
    Dot,

    // Delimiters
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Semicolon,
    Colon,
    Arrow,
    DotDot,

    // Special
    EndOfFile,
    Invalid
};

using TokenValue = std::variant<std::monostate, int64_t, double, std::string>;

class Token {
public:
    Token(TokenType type, TokenValue value, SourceLocation location);

    [[nodiscard]] TokenType type() const { return type_; }
    [[nodiscard]] const TokenValue& value() const { return value_; }
    [[nodiscard]] const SourceLocation& location() const { return location_; }

    [[nodiscard]] std::string to_string() const;

private:
    TokenType type_;
    TokenValue value_;
    SourceLocation location_;
};

} // namespace clause
