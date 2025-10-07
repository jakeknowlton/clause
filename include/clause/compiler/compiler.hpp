#pragma once

#include <memory>
#include <string>
#include "clause/parser/ast.hpp"
#include "clause/common/error.hpp"

namespace clause {

class Compiler {
public:
    Compiler();

    std::string compile(const Program& program);
    const std::vector<Error>& errors() const { return errors_; }

private:
    void compile_statement(const Statement& stmt);
    void compile_expression(const Expression& expr);
    void compile_constraint_check(const Constraint& constraint);

    std::string output_;
    std::vector<Error> errors_;
};

} // namespace clause
