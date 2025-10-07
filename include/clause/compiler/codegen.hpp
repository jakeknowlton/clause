#pragma once

#include <string>
#include <sstream>

namespace clause {

class CodeGenerator {
public:
    CodeGenerator();

    void emit(const std::string& code);
    void emit_line(const std::string& code);
    void indent();
    void dedent();

    std::string get_output() const;

private:
    std::stringstream output_;
    int indent_level_ = 0;
};

} // namespace clause
