#pragma once

#include <string>
#include <source_location>
#include "clause/common/source_location.hpp"

namespace clause {

enum class ErrorType {
    Lexical,
    Syntax,
    Semantic,
    Runtime,
    ConstraintViolation
};

class Error {
public:
    Error(ErrorType type, std::string  message, SourceLocation  location);

    [[nodiscard]] ErrorType type() const { return type_; }
    [[nodiscard]] const std::string& message() const { return message_; }
    [[nodiscard]] const SourceLocation& location() const { return location_; }

    [[nodiscard]] std::string format() const;

private:
    ErrorType type_;
    std::string message_;
    SourceLocation location_;
};

} // namespace clause
