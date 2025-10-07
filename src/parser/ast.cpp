#include <utility>

#include "clause/parser/ast.hpp"

namespace clause {
    Constraint::Constraint(ExprPtr expr, SourceLocation location)
        : ASTNode(std::move(location)), expr_(std::move(expr)) {
    }

    Type::Type(std::string name, std::vector<std::unique_ptr<Constraint> > constraints)
        : name_(std::move(name)), constraints_(std::move(constraints)) {
    }

    Program::Program(std::vector<StmtPtr> statements)
        : ASTNode(SourceLocation()), statements_(std::move(statements)) {
    }
} // namespace clause
