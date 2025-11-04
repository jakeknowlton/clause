use crate::ast::Type;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    SignedInt { value: i128, bits: u8 },
    UnsignedInt { value: u128, bits: u8 },
    Double(f64),
    Single(f32),
    Boolean(bool),
    Void,
    Array { elements: Vec<Value>, element_type: Type },
}

impl Value {
    pub fn type_name(&self) -> String {
        match self {
            Value::SignedInt { bits, .. } => format!("I{}", bits),
            Value::UnsignedInt { bits, .. } => format!("U{}", bits),
            Value::Double(_) => "F64".to_string(),
            Value::Single(_) => "F32".to_string(),
            Value::Boolean(_) => "Boolean".to_string(),
            Value::Void => "Void".to_string(),
            Value::Array { elements, element_type } => {
                format!("[{}, {}]", element_type, elements.len())
            }
        }
    }

    fn signed_min(bits: u8) -> i128 {
        if bits >= 128 {
            i128::MIN
        } else {
            -(1i128 << (bits - 1))
        }
    }

    fn signed_max(bits: u8) -> i128 {
        if bits >= 128 {
            i128::MAX
        } else {
            (1i128 << (bits - 1)) - 1
        }
    }

    fn unsigned_max(bits: u8) -> u128 {
        if bits >= 128 {
            u128::MAX
        } else {
            (1u128 << bits) - 1
        }
    }

    fn validate_signed(value: i128, bits: u8) -> Result<(), String> {
        let min = Self::signed_min(bits);
        let max = Self::signed_max(bits);
        if value < min || value > max {
            Err(format!(
                "Value {} out of range for I{} (range: {} to {})",
                value, bits, min, max
            ))
        } else {
            Ok(())
        }
    }

    fn validate_unsigned(value: u128, bits: u8) -> Result<(), String> {
        let max = Self::unsigned_max(bits);
        if value > max {
            Err(format!(
                "Value {} out of range for U{} (max: {})",
                value, bits, max
            ))
        } else {
            Ok(())
        }
    }

    fn validate_signed_operation(value: i128, bits: u8, operation: &str) -> Result<(), String> {
        let min = Self::signed_min(bits);
        let max = Self::signed_max(bits);
        if value < min || value > max {
            Err(format!(
                "I{} overflow in {}: result {} out of range (valid range: {} to {})",
                bits, operation, value, min, max
            ))
        } else {
            Ok(())
        }
    }

    fn validate_unsigned_operation(value: u128, bits: u8, operation: &str) -> Result<(), String> {
        let max = Self::unsigned_max(bits);
        if value > max {
            Err(format!(
                "U{} overflow in {}: result {} out of range (max: {})",
                bits, operation, value, max
            ))
        } else {
            Ok(())
        }
    }

    pub fn new_signed(value: i128, bits: u8) -> Result<Self, String> {
        Self::validate_signed(value, bits)?;
        Ok(Value::SignedInt { value, bits })
    }

    pub fn new_unsigned(value: u128, bits: u8) -> Result<Self, String> {
        Self::validate_unsigned(value, bits)?;
        Ok(Value::UnsignedInt { value, bits })
    }

    /// Index into an array value, returning the element at the given index
    pub fn index(&self, idx: usize) -> Result<Value, String> {
        match self {
            Value::Array { elements, .. } => {
                if idx >= elements.len() {
                    Err(format!(
                        "Array index out of bounds: index {} but length is {}",
                        idx,
                        elements.len()
                    ))
                } else {
                    Ok(elements[idx].clone())
                }
            }
            _ => Err(format!("Cannot index into non-array type {}", self.type_name())),
        }
    }

    /// Set an element in an array at the given index
    pub fn set_index(&mut self, idx: usize, value: Value) -> Result<(), String> {
        match self {
            Value::Array { elements, element_type } => {
                if idx >= elements.len() {
                    return Err(format!(
                        "Array index out of bounds: index {} but length is {}",
                        idx,
                        elements.len()
                    ));
                }

                // Verify the value matches the array's element type
                let value_type = match &value {
                    Value::SignedInt { bits, .. } => Type::Signed(*bits),
                    Value::UnsignedInt { bits, .. } => Type::Unsigned(*bits),
                    Value::Single(_) => Type::F32,
                    Value::Double(_) => Type::F64,
                    Value::Boolean(_) => Type::Bool,
                    Value::Void => Type::Void,
                    Value::Array { elements: elems, element_type: elem_ty } => {
                        Type::Array(Box::new(elem_ty.clone()), elems.len())
                    }
                };

                if &value_type != element_type {
                    return Err(format!(
                        "Type mismatch: cannot assign {} to array of {}",
                        value_type, element_type
                    ));
                }

                elements[idx] = value;
                Ok(())
            }
            _ => Err(format!("Cannot index into non-array type {}", self.type_name())),
        }
    }

    fn new_signed_from_operation(value: i128, bits: u8, operation: &str) -> Result<Self, String> {
        Self::validate_signed_operation(value, bits, operation)?;
        Ok(Value::SignedInt { value, bits })
    }

    fn new_unsigned_from_operation(value: u128, bits: u8, operation: &str) -> Result<Self, String> {
        Self::validate_unsigned_operation(value, bits, operation)?;
        Ok(Value::UnsignedInt { value, bits })
    }

    fn validate_bitwidths(
        left_bits: u8,
        right_bits: u8,
        signed: bool,
        operation: String,
    ) -> Result<(), String> {
        if left_bits != right_bits {
            return if signed {
                Err(format!(
                    "{} requires matching bitwidths, found I{} and I{}",
                    operation, left_bits, right_bits
                ))
            } else {
                Err(format!(
                    "{} requires matching bitwidths, found U{} and U{}",
                    operation, left_bits, right_bits
                ))
            };
        }
        Ok(())
    }

    pub fn add(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (
                Value::SignedInt {
                    value: a,
                    bits: bits_a,
                },
                Value::SignedInt {
                    value: b,
                    bits: bits_b,
                },
            ) => {
                Self::validate_bitwidths(*bits_a, *bits_b, true, "Integer addition".to_string())?;
                let result = a.checked_add(*b).ok_or_else(|| {
                    format!("I{} overflow in addition: operands too large", bits_a)
                })?;
                Self::new_signed_from_operation(result, *bits_a, "addition")
            }
            (
                Value::UnsignedInt {
                    value: a,
                    bits: bits_a,
                },
                Value::UnsignedInt {
                    value: b,
                    bits: bits_b,
                },
            ) => {
                Self::validate_bitwidths(*bits_a, *bits_b, false, "Integer addition".to_string())?;
                let result = a.checked_add(*b).ok_or_else(|| {
                    format!("U{} overflow in addition: operands too large", bits_a)
                })?;
                Self::new_unsigned_from_operation(result, *bits_a, "addition")
            }
            (Value::Double(a), Value::Double(b)) => Ok(Value::Double(a + b)),
            (Value::Single(a), Value::Single(b)) => Ok(Value::Single(a + b)),
            _ => Err(format!(
                "Cannot add {} and {}",
                self.type_name(),
                other.type_name()
            )),
        }
    }

    pub fn subtract(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (
                Value::SignedInt {
                    value: a,
                    bits: bits_a,
                },
                Value::SignedInt {
                    value: b,
                    bits: bits_b,
                },
            ) => {
                Self::validate_bitwidths(
                    *bits_a,
                    *bits_b,
                    true,
                    "Integer subtraction".to_string(),
                )?;
                let result = a.checked_sub(*b).ok_or_else(|| {
                    format!("I{} overflow in subtraction: operands too large", bits_a)
                })?;
                Self::new_signed_from_operation(result, *bits_a, "subtraction")
            }
            (
                Value::UnsignedInt {
                    value: a,
                    bits: bits_a,
                },
                Value::UnsignedInt {
                    value: b,
                    bits: bits_b,
                },
            ) => {
                Self::validate_bitwidths(
                    *bits_a,
                    *bits_b,
                    false,
                    "Integer subtraction".to_string(),
                )?;
                let result = a.checked_sub(*b).ok_or_else(|| {
                    format!("U{} overflow in subtraction: operands too large", bits_a)
                })?;
                Self::new_unsigned_from_operation(result, *bits_a, "subtraction")
            }
            (Value::Double(a), Value::Double(b)) => Ok(Value::Double(a - b)),
            (Value::Single(a), Value::Single(b)) => Ok(Value::Single(a - b)),
            _ => Err(format!(
                "Cannot subtract {} and {}",
                self.type_name(),
                other.type_name()
            )),
        }
    }

    pub fn multiply(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (
                Value::SignedInt {
                    value: a,
                    bits: bits_a,
                },
                Value::SignedInt {
                    value: b,
                    bits: bits_b,
                },
            ) => {
                Self::validate_bitwidths(
                    *bits_a,
                    *bits_b,
                    true,
                    "Integer multiplication".to_string(),
                )?;
                let result = a.checked_mul(*b).ok_or_else(|| {
                    format!("I{} overflow in multiplication: operands too large", bits_a)
                })?;
                Self::new_signed_from_operation(result, *bits_a, "multiplication")
            }
            (
                Value::UnsignedInt {
                    value: a,
                    bits: bits_a,
                },
                Value::UnsignedInt {
                    value: b,
                    bits: bits_b,
                },
            ) => {
                Self::validate_bitwidths(
                    *bits_a,
                    *bits_b,
                    false,
                    "Integer multiplication".to_string(),
                )?;
                let result = a.checked_mul(*b).ok_or_else(|| {
                    format!("U{} overflow in multiplication: operands too large", bits_a)
                })?;
                Self::new_unsigned_from_operation(result, *bits_a, "multiplication")
            }
            (Value::Double(a), Value::Double(b)) => Ok(Value::Double(a * b)),
            _ => Err(format!(
                "Cannot multiply {} and {}",
                self.type_name(),
                other.type_name()
            )),
        }
    }

    pub fn divide(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (
                Value::SignedInt {
                    value: a,
                    bits: bits_a,
                },
                Value::SignedInt {
                    value: b,
                    bits: bits_b,
                },
            ) => {
                Self::validate_bitwidths(*bits_a, *bits_b, true, "Integer division".to_string())?;
                if *b == 0 {
                    return Err("Division by zero".to_string());
                }
                let result = a / b;
                Self::new_signed_from_operation(result, *bits_a, "division")
            }
            (
                Value::UnsignedInt {
                    value: a,
                    bits: bits_a,
                },
                Value::UnsignedInt {
                    value: b,
                    bits: bits_b,
                },
            ) => {
                Self::validate_bitwidths(*bits_a, *bits_b, false, "Integer division".to_string())?;
                if *b == 0 {
                    return Err("Division by zero".to_string());
                }
                let result = a / b;
                Self::new_unsigned_from_operation(result, *bits_a, "division")
            }
            (Value::Double(a), Value::Double(b)) => {
                if *b == 0.0 {
                    return Err("Division by zero".to_string());
                }
                Ok(Value::Double(a / b))
            }
            (Value::Single(a), Value::Single(b)) => {
                if *b == 0.0 {
                    return Err("Division by zero".to_string());
                }
                Ok(Value::Single(a / b))
            }
            _ => Err(format!(
                "Cannot divide {} and {}",
                self.type_name(),
                other.type_name()
            )),
        }
    }

    pub fn modulo(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (
                Value::SignedInt {
                    value: a,
                    bits: bits_a,
                },
                Value::SignedInt {
                    value: b,
                    bits: bits_b,
                },
            ) => {
                Self::validate_bitwidths(*bits_a, *bits_b, true, "Integer remainder".to_string())?;
                if *b == 0 {
                    return Err("Modulo by zero".to_string());
                }
                let result = a % b;
                Self::new_signed_from_operation(result, *bits_a, "modulo")
            }
            (
                Value::UnsignedInt {
                    value: a,
                    bits: bits_a,
                },
                Value::UnsignedInt {
                    value: b,
                    bits: bits_b,
                },
            ) => {
                Self::validate_bitwidths(*bits_a, *bits_b, false, "Integer remainder".to_string())?;
                if *b == 0 {
                    return Err("Modulo by zero".to_string());
                }
                let result = a % b;
                Self::new_unsigned_from_operation(result, *bits_a, "modulo")
            }
            _ => Err(format!(
                "Cannot perform modulo on {} and {}",
                self.type_name(),
                other.type_name()
            )),
        }
    }

    // Bitwise operations (integers only)
    pub fn bitwise_and(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (
                Value::SignedInt {
                    value: a,
                    bits: bits_a,
                },
                Value::SignedInt {
                    value: b,
                    bits: bits_b,
                },
            ) => {
                Self::validate_bitwidths(*bits_a, *bits_b, true, "Bitwise AND".to_string())?;
                Ok(Value::SignedInt {
                    value: a & b,
                    bits: *bits_a,
                })
            }
            (
                Value::UnsignedInt {
                    value: a,
                    bits: bits_a,
                },
                Value::UnsignedInt {
                    value: b,
                    bits: bits_b,
                },
            ) => {
                Self::validate_bitwidths(*bits_a, *bits_b, false, "Bitwise AND".to_string())?;
                Ok(Value::UnsignedInt {
                    value: a & b,
                    bits: *bits_a,
                })
            }
            _ => Err(format!(
                "Bitwise AND requires matching integer types, found {} and {}",
                self.type_name(),
                other.type_name()
            )),
        }
    }

    pub fn bitwise_or(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (
                Value::SignedInt {
                    value: a,
                    bits: bits_a,
                },
                Value::SignedInt {
                    value: b,
                    bits: bits_b,
                },
            ) => {
                Self::validate_bitwidths(*bits_a, *bits_b, true, "Bitwise OR".to_string())?;
                Ok(Value::SignedInt {
                    value: a | b,
                    bits: *bits_a,
                })
            }
            (
                Value::UnsignedInt {
                    value: a,
                    bits: bits_a,
                },
                Value::UnsignedInt {
                    value: b,
                    bits: bits_b,
                },
            ) => {
                Self::validate_bitwidths(*bits_a, *bits_b, false, "Bitwise OR".to_string())?;
                Ok(Value::UnsignedInt {
                    value: a | b,
                    bits: *bits_a,
                })
            }
            _ => Err(format!(
                "Bitwise OR requires matching integer types, found {} and {}",
                self.type_name(),
                other.type_name()
            )),
        }
    }

    pub fn bitwise_xor(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (
                Value::SignedInt {
                    value: a,
                    bits: bits_a,
                },
                Value::SignedInt {
                    value: b,
                    bits: bits_b,
                },
            ) => {
                Self::validate_bitwidths(*bits_a, *bits_b, true, "Bitwise XOR".to_string())?;
                Ok(Value::SignedInt {
                    value: a ^ b,
                    bits: *bits_a,
                })
            }
            (
                Value::UnsignedInt {
                    value: a,
                    bits: bits_a,
                },
                Value::UnsignedInt {
                    value: b,
                    bits: bits_b,
                },
            ) => {
                Self::validate_bitwidths(*bits_a, *bits_b, false, "Bitwise XOR".to_string())?;
                Ok(Value::UnsignedInt {
                    value: a ^ b,
                    bits: *bits_a,
                })
            }
            _ => Err(format!(
                "Bitwise XOR requires matching integer types, found {} and {}",
                self.type_name(),
                other.type_name()
            )),
        }
    }

    pub fn bitwise_not(&self) -> Result<Value, String> {
        match self {
            Value::SignedInt { value, bits } => Ok(Value::SignedInt {
                value: !value,
                bits: *bits,
            }),
            Value::UnsignedInt { value, bits } => Ok(Value::UnsignedInt {
                value: !value,
                bits: *bits,
            }),
            _ => Err(format!(
                "Bitwise NOT requires integer type, found {}",
                self.type_name()
            )),
        }
    }

    fn get_shift_amount(left: &Value, right: &Value) -> Result<u128, String> {
        let shift_amount = match right {
            Value::SignedInt { value: b, .. } => {
                if *b < 0 {
                    return Err("Shift amount cannot be negative".to_string());
                }
                *b as u128
            }
            Value::UnsignedInt { value: b, .. } => *b,
            _ => {
                return Err(format!(
                    "Shift amount must be integer, found {}",
                    right.type_name()
                ));
            }
        };

        let bits = match left {
            Value::SignedInt { bits, .. } => *bits,
            Value::UnsignedInt { bits, .. } => *bits,
            _ => {
                return Err(format!(
                    "Left shift requires integer type, found {}",
                    left.type_name()
                ));
            }
        };

        if shift_amount >= (bits as u128) {
            return Err(format!(
                "Shift amount {} too large for type {}",
                shift_amount,
                left.type_name()
            ));
        }

        Ok(shift_amount)
    }

    pub fn left_shift(&self, other: &Value) -> Result<Value, String> {
        let shift_amount = Self::get_shift_amount(self, other)?;

        match self {
            Value::SignedInt { value, bits } => Ok(Value::SignedInt {
                value: *value << shift_amount,
                bits: *bits,
            }),
            Value::UnsignedInt { value, bits } => Ok(Value::UnsignedInt {
                value: *value << shift_amount,
                bits: *bits,
            }),
            _ => unreachable!(),
        }
    }

    pub fn right_shift(&self, other: &Value) -> Result<Value, String> {
        let shift_amount = Self::get_shift_amount(self, other)?;

        match self {
            Value::SignedInt { value, bits } => Ok(Value::SignedInt {
                value: *value >> shift_amount,
                bits: *bits,
            }),
            Value::UnsignedInt { value, bits } => Ok(Value::UnsignedInt {
                value: *value >> shift_amount,
                bits: *bits,
            }),
            _ => unreachable!(),
        }
    }

    // Comparison operations
    pub fn equals(&self, other: &Value) -> Result<Value, String> {
        Ok(Value::Boolean(match (self, other) {
            // Same type comparisons
            (
                Value::SignedInt {
                    value: a,
                    bits: bits_a,
                },
                Value::SignedInt {
                    value: b,
                    bits: bits_b,
                },
            ) => {
                Self::validate_bitwidths(*bits_a, *bits_b, true, "Equals comparison".to_string())?;
                a == b
            }
            (
                Value::UnsignedInt {
                    value: a,
                    bits: bits_a,
                },
                Value::UnsignedInt {
                    value: b,
                    bits: bits_b,
                },
            ) => {
                Self::validate_bitwidths(*bits_a, *bits_b, false, "Equals comparison".to_string())?;
                a == b
            }
            (Value::Double(a), Value::Double(b)) => a == b,
            (Value::Single(a), Value::Single(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Void, Value::Void) => true,
            _ => {
                return Err(format!(
                    "Cannot compare {} and {}",
                    self.type_name(),
                    other.type_name()
                ));
            }
        }))
    }

    pub fn not_equals(&self, other: &Value) -> Result<Value, String> {
        match self.equals(other) {
            Ok(v) => match v {
                Value::Boolean(b) => Ok(Value::Boolean(!b)),
                _ => unreachable!(),
            },
            Err(e) => Err(e),
        }
    }

    pub fn less_than(&self, other: &Value) -> Result<Value, String> {
        Ok(Value::Boolean(match (self, other) {
            (
                Value::SignedInt {
                    value: a,
                    bits: bits_a,
                },
                Value::SignedInt {
                    value: b,
                    bits: bits_b,
                },
            ) => {
                Self::validate_bitwidths(
                    *bits_a,
                    *bits_b,
                    true,
                    "Less than comparison".to_string(),
                )?;
                a < b
            }
            (
                Value::UnsignedInt {
                    value: a,
                    bits: bits_a,
                },
                Value::UnsignedInt {
                    value: b,
                    bits: bits_b,
                },
            ) => {
                Self::validate_bitwidths(
                    *bits_a,
                    *bits_b,
                    true,
                    "Less than comparison".to_string(),
                )?;
                a < b
            }
            (Value::Double(a), Value::Double(b)) => a < b,
            (Value::Single(a), Value::Single(b)) => a < b,
            _ => {
                return Err(format!(
                    "Cannot compare {} and {}",
                    self.type_name(),
                    other.type_name()
                ));
            }
        }))
    }

    pub fn less_equal(&self, other: &Value) -> Result<Value, String> {
        Ok(Value::Boolean(match (self, other) {
            (
                Value::SignedInt {
                    value: a,
                    bits: bits_a,
                },
                Value::SignedInt {
                    value: b,
                    bits: bits_b,
                },
            ) => {
                Self::validate_bitwidths(
                    *bits_a,
                    *bits_b,
                    true,
                    "Less than or equal to comparison".to_string(),
                )?;
                a <= b
            }
            (
                Value::UnsignedInt {
                    value: a,
                    bits: bits_a,
                },
                Value::UnsignedInt {
                    value: b,
                    bits: bits_b,
                },
            ) => {
                Self::validate_bitwidths(
                    *bits_a,
                    *bits_b,
                    true,
                    "Less than or equal to comparison".to_string(),
                )?;
                a <= b
            }
            (Value::Double(a), Value::Double(b)) => a <= b,
            (Value::Single(a), Value::Single(b)) => a <= b,
            _ => {
                return Err(format!(
                    "Cannot compare {} and {}",
                    self.type_name(),
                    other.type_name()
                ));
            }
        }))
    }

    pub fn greater_than(&self, other: &Value) -> Result<Value, String> {
        Ok(Value::Boolean(match (self, other) {
            (
                Value::SignedInt {
                    value: a,
                    bits: bits_a,
                },
                Value::SignedInt {
                    value: b,
                    bits: bits_b,
                },
            ) => {
                Self::validate_bitwidths(
                    *bits_a,
                    *bits_b,
                    true,
                    "Greater than comparison".to_string(),
                )?;
                a > b
            }
            (
                Value::UnsignedInt {
                    value: a,
                    bits: bits_a,
                },
                Value::UnsignedInt {
                    value: b,
                    bits: bits_b,
                },
            ) => {
                Self::validate_bitwidths(
                    *bits_a,
                    *bits_b,
                    true,
                    "Greater than comparison".to_string(),
                )?;
                a > b
            }
            (Value::Double(a), Value::Double(b)) => a > b,
            (Value::Single(a), Value::Single(b)) => a > b,
            _ => {
                return Err(format!(
                    "Cannot compare {} and {}",
                    self.type_name(),
                    other.type_name()
                ));
            }
        }))
    }

    pub fn greater_equal(&self, other: &Value) -> Result<Value, String> {
        Ok(Value::Boolean(match (self, other) {
            (
                Value::SignedInt {
                    value: a,
                    bits: bits_a,
                },
                Value::SignedInt {
                    value: b,
                    bits: bits_b,
                },
            ) => {
                Self::validate_bitwidths(
                    *bits_a,
                    *bits_b,
                    true,
                    "Greater than or equal to comparison".to_string(),
                )?;
                a >= b
            }
            (
                Value::UnsignedInt {
                    value: a,
                    bits: bits_a,
                },
                Value::UnsignedInt {
                    value: b,
                    bits: bits_b,
                },
            ) => {
                Self::validate_bitwidths(
                    *bits_a,
                    *bits_b,
                    true,
                    "Greater than or equal to comparison".to_string(),
                )?;
                a >= b
            }
            (Value::Double(a), Value::Double(b)) => a >= b,
            (Value::Single(a), Value::Single(b)) => a >= b,
            _ => {
                return Err(format!(
                    "Cannot compare {} and {}",
                    self.type_name(),
                    other.type_name()
                ));
            }
        }))
    }

    pub fn negate(&self) -> Result<Value, String> {
        match self {
            Value::SignedInt { value, bits } => {
                let result = value.checked_neg().ok_or_else(|| {
                    format!(
                        "I{} overflow in negation (valid range: {} to {})",
                        bits,
                        Self::signed_max(*bits),
                        Self::signed_min(*bits)
                    )
                })?;
                // "I{} overflow in {}: result {} out of range (valid range: {} to {})",
                Self::new_signed_from_operation(result, *bits, "negation")
            }
            Value::Double(value) => Ok(Value::Double(-value)),
            Value::Single(value) => Ok(Value::Single(-value)),
            _ => Err(format!("Cannot negate {}", self.type_name())),
        }
    }

    pub fn logical_not(&self) -> Result<Value, String> {
        match self {
            Value::Boolean(b) => Ok(Value::Boolean(!b)),
            _ => Err(format!(
                "Logical NOT requires boolean, found {}",
                self.type_name()
            )),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::SignedInt { value, .. } => write!(f, "{}", value),
            Value::UnsignedInt { value, .. } => write!(f, "{}", value),
            Value::Double(d) => write!(f, "{}", d),
            Value::Single(s) => write!(f, "{}", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Void => write!(f, "void"),
            Value::Array { elements, .. } => {
                write!(f, "[")?;
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", elem)?;
                }
                write!(f, "]")
            }
        }
    }
}
