use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt;

/// Unique identifier for a StateAspect
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AspectId(pub usize);

/// Represents the value of a StateAspect (legacy type for backward compatibility)
#[derive(Debug, Clone, PartialEq)]
pub enum StateValue {
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
}

impl StateValue {
    /// Validate this value against constraints
    pub fn validate(&self, min: Option<&StateValue>, max: Option<&StateValue>) -> Result<(), String> {
        if let (Some(min_val), Some(max_val)) = (min, max) {
            match (self, min_val, max_val) {
                (StateValue::Integer(v), StateValue::Integer(min_v), StateValue::Integer(max_v)) => {
                    if v < min_v || v > max_v {
                        return Err(format!(
                            "Integer value {} out of range [{}, {}]",
                            v, min_v, max_v
                        ));
                    }
                }
                (StateValue::Float(v), StateValue::Float(min_v), StateValue::Float(max_v)) => {
                    if v < min_v || v > max_v {
                        return Err(format!(
                            "Float value {} out of range [{}, {}]",
                            v, min_v, max_v
                        ));
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}

impl fmt::Display for StateValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StateValue::Bool(b) => write!(f, "{}", b),
            StateValue::Integer(i) => write!(f, "{}", i),
            StateValue::Float(v) => write!(f, "{}", v),
            StateValue::String(s) => write!(f, "\"{}\"", s),
        }
    }
}

/// Value bounds for type-constrained aspects
#[derive(Debug, Clone)]
pub struct Bounds<T> {
    pub min: Option<T>,
    pub max: Option<T>,
}

impl<T> Bounds<T> {
    pub fn new() -> Self {
        Self { min: None, max: None }
    }

    pub fn with_min(mut self, min: T) -> Self {
        self.min = Some(min);
        self
    }

    pub fn with_max(mut self, max: T) -> Self {
        self.max = Some(max);
        self
    }

    pub fn with_range(mut self, min: T, max: T) -> Self {
        self.min = Some(min);
        self.max = Some(max);
        self
    }
}

impl<T> Default for Bounds<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Validates a value against bounds
pub fn validate_bounds<T>(value: &T, bounds: &Bounds<T>) -> Result<(), String>
where
    T: PartialOrd + fmt::Display,
{
    if let Some(ref min) = bounds.min {
        if value < min {
            return Err(format!("Value {} is below minimum {}", value, min));
        }
    }
    if let Some(ref max) = bounds.max {
        if value > max {
            return Err(format!("Value {} exceeds maximum {}", value, max));
        }
    }
    Ok(())
}

/// Defines a single StateAspect - an orthogonal dimension of the state vector
/// Generic version that supports any type
#[derive(Debug, Clone)]
pub struct StateAspect<T> {
    pub id: AspectId,
    pub name: String,
    pub default_value: T,
    pub bounds: Bounds<T>,
}

impl<T> StateAspect<T>
where
    T: Clone + Send + Sync + 'static,
{
    pub fn new(id: AspectId, name: impl Into<String>, default_value: T) -> Self {
        Self {
            id,
            name: name.into(),
            default_value,
            bounds: Bounds::new(),
        }
    }

    /// Set minimum value constraint
    pub fn with_min(mut self, min: T) -> Self {
        self.bounds.min = Some(min);
        self
    }

    /// Set maximum value constraint
    pub fn with_max(mut self, max: T) -> Self {
        self.bounds.max = Some(max);
        self
    }

    /// Set both min and max value constraints
    pub fn with_range(mut self, min: T, max: T) -> Self {
        self.bounds.min = Some(min);
        self.bounds.max = Some(max);
        self
    }

    /// Set bounds using Bounds struct
    pub fn with_bounds(mut self, bounds: Bounds<T>) -> Self {
        self.bounds = bounds;
        self
    }

    /// Validate a value against this aspect's constraints
    pub fn validate_value(&self, value: &T) -> Result<(), String>
    where
        T: PartialOrd + fmt::Display,
    {
        validate_bounds(value, &self.bounds)
    }
}

/// Legacy StateAspect using StateValue enum (for backward compatibility)
#[derive(Debug, Clone)]
pub struct StateAspectLegacy {
    pub id: AspectId,
    pub name: String,
    pub default_value: StateValue,
    /// Optional minimum value constraint
    pub min_value: Option<StateValue>,
    /// Optional maximum value constraint
    pub max_value: Option<StateValue>,
}

impl StateAspectLegacy {
    pub fn new(id: AspectId, name: impl Into<String>, default_value: StateValue) -> Self {
        Self {
            id,
            name: name.into(),
            default_value,
            min_value: None,
            max_value: None,
        }
    }

    /// Set minimum value constraint
    pub fn with_min(mut self, min: StateValue) -> Self {
        self.min_value = Some(min);
        self
    }

    /// Set maximum value constraint
    pub fn with_max(mut self, max: StateValue) -> Self {
        self.max_value = Some(max);
        self
    }

    /// Set both min and max value constraints
    pub fn with_range(mut self, min: StateValue, max: StateValue) -> Self {
        self.min_value = Some(min);
        self.max_value = Some(max);
        self
    }

    /// Validate a value against this aspect's constraints
    pub fn validate_value(&self, value: &StateValue) -> Result<(), String> {
        value.validate(self.min_value.as_ref(), self.max_value.as_ref())
    }
}

/// Represents the complete system state as a high-dimensional vector
#[derive(Debug, Default)]
pub struct State {
    /// Map from aspect ID to its current value (type-erased)
    values: HashMap<AspectId, Box<dyn Any + Send + Sync>>,
}

impl Clone for State {
    fn clone(&self) -> Self {
        // Since we can't clone Box<dyn Any> directly, we create a new State
        // and use a more efficient approach - copy only what we need
        let mut new_state = State::new();
        for (key, value) in &self.values {
            // Use reflection to clone common types
            if let Some(b) = value.downcast_ref::<bool>() {
                new_state.values.insert(*key, Box::new(*b));
            } else if let Some(i) = value.downcast_ref::<i64>() {
                new_state.values.insert(*key, Box::new(*i));
            } else if let Some(f) = value.downcast_ref::<f64>() {
                new_state.values.insert(*key, Box::new(*f));
            } else if let Some(s) = value.downcast_ref::<String>() {
                new_state.values.insert(*key, Box::new(s.clone()));
            } else if let Some(i) = value.downcast_ref::<i32>() {
                new_state.values.insert(*key, Box::new(*i));
            }
            // For other types, we'd need a trait-based approach
        }
        new_state
    }
}

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        if self.values.len() != other.values.len() {
            return false;
        }
        for (key, value) in &self.values {
            if let Some(other_value) = other.values.get(key) {
                // Compare by TypeId first
                if value.type_id() != other_value.type_id() {
                    return false;
                }
                // For common types, do actual comparison
                if let (Some(a), Some(b)) = (
                    value.downcast_ref::<bool>(),
                    other_value.downcast_ref::<bool>(),
                ) {
                    if a != b {
                        return false;
                    }
                } else if let (Some(a), Some(b)) = (
                    value.downcast_ref::<i64>(),
                    other_value.downcast_ref::<i64>(),
                ) {
                    if a != b {
                        return false;
                    }
                } else if let (Some(a), Some(b)) = (
                    value.downcast_ref::<f64>(),
                    other_value.downcast_ref::<f64>(),
                ) {
                    if a != b {
                        return false;
                    }
                } else if let (Some(a), Some(b)) = (
                    value.downcast_ref::<String>(),
                    other_value.downcast_ref::<String>(),
                ) {
                    if a != b {
                        return false;
                    }
                } else if let (Some(a), Some(b)) = (
                    value.downcast_ref::<i32>(),
                    other_value.downcast_ref::<i32>(),
                ) {
                    if a != b {
                        return false;
                    }
                }
                // For other types, we can't compare, so assume equal if TypeId matches
            } else {
                return false;
            }
        }
        true
    }
}

impl State {
    /// Create a new empty state
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    /// Get the type-erased value of a specific aspect
    pub fn get(&self, aspect_id: AspectId) -> Option<&(dyn Any + Send + Sync)> {
        self.values.get(&aspect_id).map(|boxed| boxed.as_ref())
    }

    /// Get the value of a specific aspect as a specific type
    pub fn get_as<T: 'static>(&self, aspect_id: AspectId) -> Option<&T> {
        self.get(aspect_id).and_then(|boxed| boxed.downcast_ref())
    }

    /// Set the value of a specific aspect, returning a new state
    pub fn set(&self, aspect_id: AspectId, value: Box<dyn Any + Send + Sync>) -> Self {
        let mut new_state = self.clone();
        new_state.values.insert(aspect_id, value);
        new_state
    }

    /// Set a typed value, returning a new state
    pub fn set_typed<T: Any + Send + Sync>(&self, aspect_id: AspectId, value: T) -> Self {
        self.set(aspect_id, Box::new(value))
    }

    /// Check if the state contains a specific aspect
    pub fn has(&self, aspect_id: AspectId) -> bool {
        self.values.contains_key(&aspect_id)
    }

    /// Get all aspect IDs in this state
    pub fn aspect_ids(&self) -> impl Iterator<Item = AspectId> + '_ {
        self.values.keys().copied()
    }

    /// Get the number of aspects in this state
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Check if the state is empty
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Get the TypeId of a value at a specific aspect
    pub fn get_type_id(&self, aspect_id: AspectId) -> Option<TypeId> {
        self.get(aspect_id).map(|boxed| boxed.type_id())
    }
}

/// Builder for constructing State instances
pub struct StateBuilder {
    values: HashMap<AspectId, Box<dyn Any + Send + Sync>>,
}

impl StateBuilder {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    /// Set a type-erased value
    pub fn set(mut self, aspect_id: AspectId, value: Box<dyn Any + Send + Sync>) -> Self {
        self.values.insert(aspect_id, value);
        self
    }

    /// Set a typed value
    pub fn set_typed<T: Any + Send + Sync>(mut self, aspect_id: AspectId, value: T) -> Self {
        self.values.insert(aspect_id, Box::new(value));
        self
    }

    /// Set a boolean value (convenience method)
    pub fn set_bool(mut self, aspect_id: AspectId, value: bool) -> Self {
        self.values.insert(aspect_id, Box::new(value));
        self
    }

    /// Set an integer value (convenience method)
    pub fn set_int(mut self, aspect_id: AspectId, value: i64) -> Self {
        self.values.insert(aspect_id, Box::new(value));
        self
    }

    /// Set a float value (convenience method)
    pub fn set_float(mut self, aspect_id: AspectId, value: f64) -> Self {
        self.values.insert(aspect_id, Box::new(value));
        self
    }

    /// Set a string value (convenience method)
    pub fn set_string(mut self, aspect_id: AspectId, value: impl Into<String>) -> Self {
        self.values.insert(aspect_id, Box::new(value.into()));
        self
    }

    pub fn build(self) -> State {
        State { values: self.values }
    }
}

impl Default for StateBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to create a Box<dyn Any + Send + Sync> from common types
pub fn any_value<T: Any + Send + Sync>(value: T) -> Box<dyn Any + Send + Sync> {
    Box::new(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Example user-defined struct
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
    struct Temperature {
        celsius: i32,
    }

    impl Temperature {
        fn new(celsius: i32) -> Self {
            Self { celsius }
        }
    }

    impl fmt::Display for Temperature {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}°C", self.celsius)
        }
    }

    // Example user-defined struct without PartialOrd
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Status {
        code: u32,
        message: String,
    }

    impl Status {
        fn new(code: u32, message: impl Into<String>) -> Self {
            Self {
                code,
                message: message.into(),
            }
        }
    }

    #[test]
    fn test_generic_aspect_creation() {
        let id = AspectId(0);
        let aspect = StateAspect::new(id, "counter", 0i32);

        assert_eq!(aspect.id, id);
        assert_eq!(aspect.name, "counter");
        assert_eq!(aspect.default_value, 0);
        assert!(aspect.bounds.min.is_none());
        assert!(aspect.bounds.max.is_none());
    }

    #[test]
    fn test_generic_aspect_with_bounds() {
        let id = AspectId(0);
        let aspect: StateAspect<i32> = StateAspect::new(id, "temperature", 20)
            .with_range(0, 100);

        assert_eq!(aspect.bounds.min, Some(0));
        assert_eq!(aspect.bounds.max, Some(100));
    }

    #[test]
    fn test_validate_bounds() {
        let bounds = Bounds::new().with_range(0i32, 100);
        assert!(validate_bounds(&50, &bounds).is_ok());
        assert!(validate_bounds(&0, &bounds).is_ok());
        assert!(validate_bounds(&100, &bounds).is_ok());
        assert!(validate_bounds(&-1, &bounds).is_err());
        assert!(validate_bounds(&101, &bounds).is_err());
    }

    #[test]
    fn test_custom_struct_with_bounds() {
        let id = AspectId(0);
        let aspect: StateAspect<Temperature> = StateAspect::new(
            id,
            "temp",
            Temperature::new(20),
        )
        .with_range(Temperature::new(0), Temperature::new(100));

        assert!(aspect.validate_value(&Temperature::new(50)).is_ok());
        assert!(aspect.validate_value(&Temperature::new(0)).is_ok());
        assert!(aspect.validate_value(&Temperature::new(-10)).is_err());
    }

    #[test]
    fn test_state_with_typed_values() {
        let id1 = AspectId(0);
        let id2 = AspectId(1);
        let id3 = AspectId(2);

        let state = StateBuilder::new()
            .set_typed(id1, 42i32)
            .set_typed(id2, Temperature::new(25))
            .set_typed(id3, Status::new(200, "OK"))
            .build();

        assert_eq!(state.get_as::<i32>(id1), Some(&42));
        assert_eq!(state.get_as::<Temperature>(id2), Some(&Temperature::new(25)));
        assert_eq!(
            state.get_as::<Status>(id3),
            Some(&Status::new(200, "OK"))
        );
    }

    #[test]
    fn test_state_set_typed() {
        let id = AspectId(0);
        let state = State::new();

        let new_state = state.set_typed(id, 42i32);
        assert_eq!(new_state.get_as::<i32>(id), Some(&42));

        let new_state = new_state.set_typed(id, 100i32);
        assert_eq!(new_state.get_as::<i32>(id), Some(&100));
    }

    #[test]
    fn test_state_type_mismatch() {
        let id = AspectId(0);
        let state = StateBuilder::new()
            .set_typed(id, 42i32)
            .build();

        // Trying to get as wrong type returns None
        assert_eq!(state.get_as::<String>(id), None);
        assert_eq!(state.get_as::<bool>(id), None);
    }

    #[test]
    fn test_state_type_id() {
        let id1 = AspectId(0);
        let id2 = AspectId(1);
        let id3 = AspectId(2);

        let state = StateBuilder::new()
            .set_typed(id1, 42i32)
            .set_typed(id2, Temperature::new(25))
            .set_typed(id3, Status::new(200, "OK"))
            .build();

        assert_eq!(state.get_type_id(id1), Some(TypeId::of::<i32>()));
        assert_eq!(state.get_type_id(id2), Some(TypeId::of::<Temperature>()));
        assert_eq!(state.get_type_id(id3), Some(TypeId::of::<Status>()));
    }

    #[test]
    fn test_bounds_float() {
        let bounds = Bounds::new().with_range(0.0f64, 1.0);
        assert!(validate_bounds(&0.5, &bounds).is_ok());
        assert!(validate_bounds(&0.0, &bounds).is_ok());
        assert!(validate_bounds(&1.0, &bounds).is_ok());
        assert!(validate_bounds(&1.5, &bounds).is_err());
    }

    #[test]
    fn test_aspect_validate_value() {
        let id = AspectId(0);
        let aspect: StateAspect<i32> = StateAspect::new(id, "value", 50)
            .with_range(0, 100);

        assert!(aspect.validate_value(&50).is_ok());
        assert!(aspect.validate_value(&0).is_ok());
        assert!(aspect.validate_value(&100).is_ok());
        assert!(aspect.validate_value(&-1).is_err());
    }

    #[test]
    fn test_convenience_setters() {
        let id1 = AspectId(0);
        let id2 = AspectId(1);
        let id3 = AspectId(2);
        let id4 = AspectId(3);

        let state = StateBuilder::new()
            .set_bool(id1, true)
            .set_int(id2, 42)
            .set_float(id3, 3.14)
            .set_string(id4, "hello")
            .build();

        assert_eq!(state.get_as::<bool>(id1), Some(&true));
        assert_eq!(state.get_as::<i64>(id2), Some(&42));
        assert_eq!(state.get_as::<f64>(id3), Some(&3.14));
        assert_eq!(state.get_as::<String>(id4), Some(&"hello".to_string()));
    }

    // Legacy tests for backward compatibility
    #[test]
    fn test_legacy_aspect() {
        let id = AspectId(0);
        let aspect = StateAspectLegacy::new(id, "mode", StateValue::String("idle".to_string()));

        assert_eq!(aspect.id, id);
        assert_eq!(aspect.name, "mode");
    }

    #[test]
    fn test_legacy_aspect_with_range() {
        let id = AspectId(0);
        let aspect = StateAspectLegacy::new(id, "count", StateValue::Integer(0))
            .with_range(StateValue::Integer(0), StateValue::Integer(100));

        assert!(aspect.validate_value(&StateValue::Integer(50)).is_ok());
        assert!(aspect.validate_value(&StateValue::Integer(101)).is_err());
    }
}