#pragma once

#include <string>
#include <vector>
#include <optional>
#include "clause/lexer/token.hpp"
#include "clause/common/error.hpp"

namespace clause {

class Lexer {
public:
    explicit Lexer(std::string  source, std::string  filename = "<input>");

    std::vector<Token> tokenize();
    [[nodiscard]] const std::vector<Error>& errors() const { return errors_; }

private:
    std::optional<Token> next_token();

    [[nodiscard]] char peek() const;
    [[nodiscard]] char peek_next() const;
    char advance();
    [[nodiscard]] bool is_at_end() const;

    void skip_whitespace();
    void skip_comment();

    [[nodiscard]] Token make_token(TokenType type, TokenValue value = std::monostate{}) const;
    Token number();
    Token string();
    Token identifier();

    void add_error(const std::string& message);

    std::string source_;
    std::string filename_;
    size_t current_ = 0;
    size_t line_ = 1;
    size_t column_ = 1;
    size_t token_start_ = 0;
    std::vector<Error> errors_;
};

} // namespace clause
