use crate::value::Value;
use std::collections::HashMap;

/// Represents a variable binding with its value and mutability
#[derive(Debug, Clone)]
struct Binding {
    value: Value,
    mutable: bool,
}

/// Environment for managing variable bindings with scoped lookup
#[derive(Debug, Clone)]
pub struct Environment {
    /// Stack of scopes, each scope is a map of variable names to bindings
    scopes: Vec<HashMap<String, Binding>>,
}

impl Environment {
    /// Create a new environment with a single global scope
    pub fn new() -> Self {
        Environment {
            scopes: vec![HashMap::new()],
        }
    }

    /// Push a new scope onto the environment (entering a block)
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Pop the current scope from the environment (exiting a block)
    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    /// Define a new variable in the current scope
    /// Returns an error if the variable already exists in the current scope
    pub fn define(&mut self, name: String, value: Value, mutable: bool) -> Result<(), String> {
        let current_scope = self.scopes.last_mut().unwrap();

        if current_scope.contains_key(&name) {
            return Err(format!(
                "Variable '{}' is already defined in this scope",
                name
            ));
        }

        current_scope.insert(name, Binding { value, mutable });
        Ok(())
    }

    /// Get the value of a variable by searching from the innermost scope outward
    pub fn get(&self, name: &str) -> Result<Value, String> {
        for scope in self.scopes.iter().rev() {
            if let Some(binding) = scope.get(name) {
                return Ok(binding.value.clone());
            }
        }
        Err(format!("Undefined variable '{}'", name))
    }

    /// Assign a new value to an existing variable
    /// Returns an error if the variable doesn't exist or is immutable
    pub fn assign(&mut self, name: &str, value: Value) -> Result<(), String> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(binding) = scope.get_mut(name) {
                if !binding.mutable {
                    return Err(format!("Cannot assign to immutable variable '{}'", name));
                }
                binding.value = value;
                return Ok(());
            }
        }
        Err(format!("Undefined variable '{}'", name))
    }

    /// Check if a variable exists in any scope
    pub fn exists(&self, name: &str) -> bool {
        self.scopes
            .iter()
            .rev()
            .any(|scope| scope.contains_key(name))
    }

    /// Get the number of scopes in the environment
    pub fn scope_depth(&self) -> usize {
        self.scopes.len()
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_define_and_get() {
        let mut env = Environment::new();
        env.define(
            "x".to_string(),
            Value::SignedInt {
                value: 42,
                bits: 64,
            },
            true,
        )
        .unwrap();
        assert_eq!(
            env.get("x").unwrap(),
            Value::SignedInt {
                value: 42,
                bits: 64
            }
        );
    }

    #[test]
    fn test_define_duplicate_error() {
        let mut env = Environment::new();
        env.define(
            "x".to_string(),
            Value::SignedInt {
                value: 42,
                bits: 64,
            },
            true,
        )
        .unwrap();
        let result = env.define(
            "x".to_string(),
            Value::SignedInt {
                value: 10,
                bits: 64,
            },
            true,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already defined"));
    }

    #[test]
    fn test_get_undefined_error() {
        let env = Environment::new();
        let result = env.get("x");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Undefined variable"));
    }

    #[test]
    fn test_assign_mutable() {
        let mut env = Environment::new();
        env.define(
            "x".to_string(),
            Value::SignedInt {
                value: 42,
                bits: 64,
            },
            true,
        )
        .unwrap();
        env.assign(
            "x",
            Value::SignedInt {
                value: 100,
                bits: 64,
            },
        )
        .unwrap();
        assert_eq!(
            env.get("x").unwrap(),
            Value::SignedInt {
                value: 100,
                bits: 64
            }
        );
    }

    #[test]
    fn test_assign_immutable_error() {
        let mut env = Environment::new();
        env.define(
            "x".to_string(),
            Value::SignedInt {
                value: 42,
                bits: 64,
            },
            false,
        )
        .unwrap();
        let result = env.assign(
            "x",
            Value::SignedInt {
                value: 100,
                bits: 64,
            },
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("immutable"));
    }

    #[test]
    fn test_assign_undefined_error() {
        let mut env = Environment::new();
        let result = env.assign(
            "x",
            Value::SignedInt {
                value: 100,
                bits: 64,
            },
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Undefined variable"));
    }

    #[test]
    fn test_scopes() {
        let mut env = Environment::new();

        // Define in global scope
        env.define(
            "x".to_string(),
            Value::SignedInt { value: 1, bits: 64 },
            true,
        )
        .unwrap();

        // Push new scope
        env.push_scope();
        env.define(
            "y".to_string(),
            Value::SignedInt { value: 2, bits: 64 },
            true,
        )
        .unwrap();

        // Both variables should be accessible
        assert_eq!(
            env.get("x").unwrap(),
            Value::SignedInt { value: 1, bits: 64 }
        );
        assert_eq!(
            env.get("y").unwrap(),
            Value::SignedInt { value: 2, bits: 64 }
        );

        // Pop scope
        env.pop_scope();

        // x should still be accessible, y should not
        assert_eq!(
            env.get("x").unwrap(),
            Value::SignedInt { value: 1, bits: 64 }
        );
        assert!(env.get("y").is_err());
    }

    #[test]
    fn test_shadowing() {
        let mut env = Environment::new();

        // Define x in global scope
        env.define(
            "x".to_string(),
            Value::SignedInt { value: 1, bits: 64 },
            true,
        )
        .unwrap();

        // Push new scope and shadow x
        env.push_scope();
        env.define(
            "x".to_string(),
            Value::SignedInt { value: 2, bits: 64 },
            true,
        )
        .unwrap();

        // Inner x should be visible
        assert_eq!(
            env.get("x").unwrap(),
            Value::SignedInt { value: 2, bits: 64 }
        );

        // Pop scope
        env.pop_scope();

        // Outer x should be visible again
        assert_eq!(
            env.get("x").unwrap(),
            Value::SignedInt { value: 1, bits: 64 }
        );
    }

    #[test]
    fn test_assign_in_outer_scope() {
        let mut env = Environment::new();

        // Define mutable variable in global scope
        env.define(
            "x".to_string(),
            Value::SignedInt { value: 1, bits: 64 },
            true,
        )
        .unwrap();

        // Push new scope
        env.push_scope();

        // Assign to variable from outer scope
        env.assign(
            "x",
            Value::SignedInt {
                value: 42,
                bits: 64,
            },
        )
        .unwrap();
        assert_eq!(
            env.get("x").unwrap(),
            Value::SignedInt {
                value: 42,
                bits: 64
            }
        );

        // Pop scope
        env.pop_scope();

        // Assignment should persist
        assert_eq!(
            env.get("x").unwrap(),
            Value::SignedInt {
                value: 42,
                bits: 64
            }
        );
    }

    #[test]
    fn test_exists() {
        let mut env = Environment::new();
        assert!(!env.exists("x"));

        env.define(
            "x".to_string(),
            Value::SignedInt {
                value: 42,
                bits: 64,
            },
            true,
        )
        .unwrap();
        assert!(env.exists("x"));

        env.push_scope();
        assert!(env.exists("x")); // Still visible from outer scope

        env.define(
            "y".to_string(),
            Value::SignedInt {
                value: 10,
                bits: 64,
            },
            true,
        )
        .unwrap();
        assert!(env.exists("y"));

        env.pop_scope();
        assert!(!env.exists("y")); // y is no longer in scope
        assert!(env.exists("x")); // x still exists
    }

    #[test]
    fn test_scope_depth() {
        let mut env = Environment::new();
        assert_eq!(env.scope_depth(), 1);

        env.push_scope();
        assert_eq!(env.scope_depth(), 2);

        env.push_scope();
        assert_eq!(env.scope_depth(), 3);

        env.pop_scope();
        assert_eq!(env.scope_depth(), 2);

        env.pop_scope();
        assert_eq!(env.scope_depth(), 1);
    }
}
