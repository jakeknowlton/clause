#include "clause/common/error.hpp"
#include <sstream>
#include <utility>

namespace clause {
    Error::Error(const ErrorType type, std::string message, SourceLocation location)
        : type_(type), message_(std::move(message)), location_(std::move(location)) {
    }

std::string Error::format() const {
    std::ostringstream oss;

    const char* type_str = "Error";
    switch (type_) {
        case ErrorType::Lexical: type_str = "Lexical Error"; break;
        case ErrorType::Syntax: type_str = "Syntax Error"; break;
        case ErrorType::Semantic: type_str = "Semantic Error"; break;
        case ErrorType::Runtime: type_str = "Runtime Error"; break;
        case ErrorType::ConstraintViolation: type_str = "Constraint Violation"; break;
    }

    oss << type_str;
    if (!location_.filename.empty()) {
        oss << " at " << location_.filename << ":" << location_.line << ":" << location_.column;
    }
    oss << ": " << message_;

    return oss.str();
}

} // namespace clause
