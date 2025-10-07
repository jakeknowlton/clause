#include "clause/lexer/token.hpp"
#include <sstream>
#include <utility>

namespace clause {
    Token::Token(TokenType type, TokenValue value, SourceLocation location)
        : type_(type), value_(std::move(value)), location_(std::move(location)) {
    }

    std::string Token::to_string() const {
        std::ostringstream oss;
        oss << "Token(";

        // Add token type name here
        oss << static_cast<int>(type_);

        if (std::holds_alternative<int64_t>(value_)) {
            oss << ", " << std::get<int64_t>(value_);
        } else if (std::holds_alternative<double>(value_)) {
            oss << ", " << std::get<double>(value_);
        } else if (std::holds_alternative<std::string>(value_)) {
            oss << ", \"" << std::get<std::string>(value_) << "\"";
        }

        oss << ")";
        return oss.str();
    }
} // namespace clause
