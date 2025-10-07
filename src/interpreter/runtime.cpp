#include "clause/interpreter/runtime.hpp"
#include <stdexcept>

namespace clause {

Runtime::Runtime() {
    push_scope(); // Global scope
}

void Runtime::define_variable(const std::string& name, const Value& value) {
    scopes_.back().variables[name] = value;
}

Value Runtime::get_variable(const std::string& name) const {
    for (auto it = scopes_.rbegin(); it != scopes_.rend(); ++it) {
        auto var_it = it->variables.find(name);
        if (var_it != it->variables.end()) {
            return var_it->second;
        }
    }
    throw std::runtime_error("Undefined variable: " + name);
}

void Runtime::set_variable(const std::string& name, const Value& value) {
    for (auto it = scopes_.rbegin(); it != scopes_.rend(); ++it) {
        auto var_it = it->variables.find(name);
        if (var_it != it->variables.end()) {
            var_it->second = value;
            return;
        }
    }
    throw std::runtime_error("Undefined variable: " + name);
}

void Runtime::push_scope() {
    scopes_.emplace_back();
}

void Runtime::pop_scope() {
    if (!scopes_.empty()) {
        scopes_.pop_back();
    }
}

} // namespace clause
