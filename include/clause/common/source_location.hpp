#pragma once

#include <string>
#include <utility>

namespace clause {

struct SourceLocation {
    std::string filename;
    size_t line;
    size_t column;

    explicit SourceLocation(std::string file = "", const size_t ln = 0, const size_t col = 0)
        : filename(std::move(file)), line(ln), column(col) {
    }
};

} // namespace clause
