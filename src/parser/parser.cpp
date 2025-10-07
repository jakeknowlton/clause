#include "clause/parser/parser.hpp"

namespace clause {
    Parser::Parser(std::vector<Token> tokens)
        : tokens_(std::move(tokens)) {
    }

    std::unique_ptr<Program> Parser::parse() {
        std::vector<StmtPtr> statements;

        while (!is_at_end()) {
            if (auto stmt = parse_statement()) {
                statements.push_back(std::move(stmt));
            }
        }

        return std::make_unique<Program>(std::move(statements));
    }

    StmtPtr Parser::parse_statement() {
        // Implement statement parsing
        return nullptr;
    }

    ExprPtr Parser::parse_expression() {
        // Implement expression parsing
        return nullptr;
    }

    ExprPtr Parser::parse_primary() {
        // Implement primary expression parsing
        return nullptr;
    }

    std::unique_ptr<Type> Parser::parse_type() {
        // Implement type parsing with constraints
        return nullptr;
    }

    std::unique_ptr<Constraint> Parser::parse_constraint() {
        // Implement constraint parsing
        return nullptr;
    }

    bool Parser::match(const TokenType type) {
        if (check(type)) {
            advance();
            return true;
        }
        return false;
    }

    bool Parser::check(const TokenType type) const {
        if (is_at_end()) return false;
        return peek().type() == type;
    }

    const Token &Parser::advance() {
        if (!is_at_end()) current_++;
        return previous();
    }

    const Token &Parser::peek() const {
        return tokens_[current_];
    }

    const Token &Parser::previous() const {
        return tokens_[current_ - 1];
    }

    bool Parser::is_at_end() const {
        return peek().type() == TokenType::EndOfFile;
    }

    void Parser::add_error(const std::string &message) {
        errors_.emplace_back(ErrorType::Syntax, message, peek().location());
    }
} // namespace clause
