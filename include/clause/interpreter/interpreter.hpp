#pragma once

#include <memory>
#include "clause/parser/ast.hpp"
#include "clause/interpreter/runtime.hpp"
#include "clause/common/error.hpp"

namespace clause {

class Interpreter {
public:
    Interpreter();

    void execute(const Program& program);
    [[nodiscard]] const std::vector<Error>& errors() const { return errors_; }

private:
    void execute_statement(const Statement& stmt);
    Value evaluate_expression(const Expression& expr);
    bool check_constraint(const Value& value, const Constraint& constraint);

    Runtime runtime_;
    std::vector<Error> errors_;
};

} // namespace clause
