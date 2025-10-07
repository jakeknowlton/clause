#include "clause/compiler/codegen.hpp"

namespace clause {

CodeGenerator::CodeGenerator() = default;

void CodeGenerator::emit(const std::string& code) {
    output_ << code;
}

void CodeGenerator::emit_line(const std::string& code) {
    for (int i = 0; i < indent_level_; ++i) {
        output_ << "    ";
    }
    output_ << code << "\n";
}

void CodeGenerator::indent() {
    indent_level_++;
}

void CodeGenerator::dedent() {
    if (indent_level_ > 0) {
        indent_level_--;
    }
}

std::string CodeGenerator::get_output() const {
    return output_.str();
}

} // namespace clause
