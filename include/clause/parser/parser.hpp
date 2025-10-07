#pragma once

#include <vector>
#include <memory>
#include "clause/lexer/token.hpp"
#include "clause/parser/ast.hpp"
#include "clause/common/error.hpp"

namespace clause {

class Parser {
public:
    explicit Parser(std::vector<Token> tokens);

    std::unique_ptr<Program> parse();
    [[nodiscard]] const std::vector<Error>& errors() const { return errors_; }

private:
    StmtPtr parse_statement();
    ExprPtr parse_expression();
    ExprPtr parse_primary();

    std::unique_ptr<Type> parse_type();
    std::unique_ptr<Constraint> parse_constraint();

    bool match(TokenType type);
    [[nodiscard]] bool check(TokenType type) const;
    const Token& advance();
    [[nodiscard]] const Token& peek() const;
    [[nodiscard]] const Token& previous() const;
    [[nodiscard]] bool is_at_end() const;

    void add_error(const std::string& message);

    std::vector<Token> tokens_;
    size_t current_ = 0;
    std::vector<Error> errors_;
};

} // namespace clause
