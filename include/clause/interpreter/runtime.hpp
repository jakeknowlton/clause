#pragma once

#include <unordered_map>
#include <string>
#include <variant>
#include <vector>

namespace clause {

using Value = std::variant<std::monostate, int64_t, double, std::string, bool>;

class Runtime {
public:
    Runtime();

    void define_variable(const std::string& name, const Value& value);
    Value get_variable(const std::string& name) const;
    void set_variable(const std::string& name, const Value& value);

    void push_scope();
    void pop_scope();

private:
    struct Scope {
        std::unordered_map<std::string, Value> variables;
    };

    std::vector<Scope> scopes_;
};

} // namespace clause
