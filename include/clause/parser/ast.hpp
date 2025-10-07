#pragma once

#include <memory>
#include <utility>
#include <vector>
#include <string>
#include "clause/common/source_location.hpp"

namespace clause {

// Forward declarations
class Expression;
class Statement;

using ExprPtr = std::unique_ptr<Expression>;
using StmtPtr = std::unique_ptr<Statement>;

// Base AST node
class ASTNode {
public:
    virtual ~ASTNode() = default;

    [[nodiscard]] const SourceLocation& location() const { return location_; }

protected:
    explicit ASTNode(SourceLocation location) : location_(std::move(location)) {}

    SourceLocation location_;
};

// Expression base class
class Expression final : public ASTNode {
public:
    using ASTNode::ASTNode;
};

// Statement base class
class Statement final : public ASTNode {
public:
    using ASTNode::ASTNode;
};

// Constraint expression (e.g., x > 0, x < 100)
class Constraint final : public ASTNode {
public:
    explicit Constraint(ExprPtr expr, SourceLocation location);

    const Expression* expression() const { return expr_.get(); }

private:
    ExprPtr expr_;
};

// Type with optional constraints
class Type {
public:
    explicit Type(std::string  name, std::vector<std::unique_ptr<Constraint>> constraints = {});

    [[nodiscard]] const std::string& name() const { return name_; }
    [[nodiscard]] const std::vector<std::unique_ptr<Constraint>>& constraints() const { return constraints_; }

private:
    std::string name_;
    std::vector<std::unique_ptr<Constraint>> constraints_;
};

// Program node (root)
class Program final : public ASTNode {
public:
    explicit Program(std::vector<StmtPtr> statements);

    [[nodiscard]] const std::vector<StmtPtr>& statements() const { return statements_; }

private:
    std::vector<StmtPtr> statements_;
};

} // namespace clause
