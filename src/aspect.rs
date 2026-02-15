use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt;

// ============================================================================
// BLUEPRINT LAYER - AspectBlueprint (declarations without logic)
// ============================================================================

/// Unique identifier for an Aspect
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AspectId(pub usize);

/// Value bounds for type-constrained aspects (blueprint layer)
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

/// Type-erased representation of aspect constraints for the blueprint
#[derive(Debug)]
pub struct AspectBoundsBlueprint {
    pub type_id: TypeId,
    pub type_name: String,
    /// Min value as Any (type-erased)
    pub min_value: Option<Box<dyn Any + Send + Sync>>,
    /// Max value as Any (type-erased)
    pub max_value: Option<Box<dyn Any + Send + Sync>>,
}

impl Clone for AspectBoundsBlueprint {
    fn clone(&self) -> Self {
        Self {
            type_id: self.type_id,
            type_name: self.type_name.clone(),
            min_value: self.min_value.as_ref().map(|v| {
                if let Some(b) = v.downcast_ref::<bool>() {
                    Box::new(*b) as Box<dyn Any + Send + Sync>
                } else if let Some(i) = v.downcast_ref::<i64>() {
                    Box::new(*i) as Box<dyn Any + Send + Sync>
                } else if let Some(f) = v.downcast_ref::<f64>() {
                    Box::new(*f) as Box<dyn Any + Send + Sync>
                } else if let Some(s) = v.downcast_ref::<String>() {
                    Box::new(s.clone()) as Box<dyn Any + Send + Sync>
                } else if let Some(i) = v.downcast_ref::<i32>() {
                    Box::new(*i) as Box<dyn Any + Send + Sync>
                } else {
                    Box::new(()) as Box<dyn Any + Send + Sync>
                }
            }),
            max_value: self.max_value.as_ref().map(|v| {
                if let Some(b) = v.downcast_ref::<bool>() {
                    Box::new(*b) as Box<dyn Any + Send + Sync>
                } else if let Some(i) = v.downcast_ref::<i64>() {
                    Box::new(*i) as Box<dyn Any + Send + Sync>
                } else if let Some(f) = v.downcast_ref::<f64>() {
                    Box::new(*f) as Box<dyn Any + Send + Sync>
                } else if let Some(s) = v.downcast_ref::<String>() {
                    Box::new(s.clone()) as Box<dyn Any + Send + Sync>
                } else if let Some(i) = v.downcast_ref::<i32>() {
                    Box::new(*i) as Box<dyn Any + Send + Sync>
                } else {
                    Box::new(()) as Box<dyn Any + Send + Sync>
                }
            }),
        }
    }
}

impl AspectBoundsBlueprint {
    pub fn new<T: 'static>() -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            type_name: std::any::type_name::<T>().to_string(),
            min_value: None,
            max_value: None,
        }
    }

    pub fn with_min<T: 'static + Send + Sync>(mut self, min: T) -> Self {
        self.min_value = Some(Box::new(min));
        self
    }

    pub fn with_max<T: 'static + Send + Sync>(mut self, max: T) -> Self {
        self.max_value = Some(Box::new(max));
        self
    }

    pub fn with_range<T: 'static + Send + Sync>(mut self, min: T, max: T) -> Self {
        self.min_value = Some(Box::new(min));
        self.max_value = Some(Box::new(max));
        self
    }
}

/// Blueprint for defining an Aspect (declaration layer)
///
/// AspectBlueprint describes an orthogonal dimension of the state vector
/// without including validation logic or runtime behavior.
#[derive(Debug)]
pub struct AspectBlueprint {
    pub id: AspectId,
    pub name: String,
    /// Default value as Any (type-erased)
    pub default_value: Box<dyn Any + Send + Sync>,
    pub default_type_id: TypeId,
    pub default_type_name: String,
    /// Type-erased bounds
    pub bounds: Option<AspectBoundsBlueprint>,
}

impl Clone for AspectBlueprint {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            name: self.name.clone(),
            default_value: if let Some(b) = self.default_value.downcast_ref::<bool>() {
                Box::new(*b) as Box<dyn Any + Send + Sync>
            } else if let Some(i) = self.default_value.downcast_ref::<i64>() {
                Box::new(*i) as Box<dyn Any + Send + Sync>
            } else if let Some(f) = self.default_value.downcast_ref::<f64>() {
                Box::new(*f) as Box<dyn Any + Send + Sync>
            } else if let Some(s) = self.default_value.downcast_ref::<String>() {
                Box::new(s.clone()) as Box<dyn Any + Send + Sync>
            } else if let Some(i) = self.default_value.downcast_ref::<i32>() {
                Box::new(*i) as Box<dyn Any + Send + Sync>
            } else {
                Box::new(()) as Box<dyn Any + Send + Sync>
            },
            default_type_id: self.default_type_id,
            default_type_name: self.default_type_name.clone(),
            bounds: self.bounds.clone(),
        }
    }
}

impl AspectBlueprint {
    /// Create a new AspectBlueprint
    pub fn new<T: Any + Send + Sync + 'static>(
        id: AspectId,
        name: impl Into<String>,
        default_value: T,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            default_value: Box::new(default_value),
            default_type_id: TypeId::of::<T>(),
            default_type_name: std::any::type_name::<T>().to_string(),
            bounds: None,
        }
    }

    /// Set bounds for this aspect
    pub fn with_bounds(mut self, bounds: AspectBoundsBlueprint) -> Self {
        self.bounds = Some(bounds);
        self
    }

    /// Set min and max bounds (convenience method)
    pub fn with_range<T: 'static + Send + Sync>(mut self, min: T, max: T) -> Self {
        let bounds = AspectBoundsBlueprint::new::<T>().with_range(min, max);
        self.bounds = Some(bounds);
        self
    }

    /// Set min bound only
    pub fn with_min<T: 'static + Send + Sync>(mut self, min: T) -> Self {
        let bounds = AspectBoundsBlueprint::new::<T>().with_min(min);
        self.bounds = Some(bounds);
        self
    }

    /// Set max bound only
    pub fn with_max<T: 'static + Send + Sync>(mut self, max: T) -> Self {
        let bounds = AspectBoundsBlueprint::new::<T>().with_max(max);
        self.bounds = Some(bounds);
        self
    }
}

// ============================================================================
// RUNTIME LAYER - Aspect with validation logic
// ============================================================================

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

/// Defines a single Aspect - an orthogonal dimension of the state vector
/// Generic version that supports any type (runtime layer with validation)
#[derive(Debug, Clone)]
pub struct Aspect<T> {
    pub id: AspectId,
    pub name: String,
    pub default_value: T,
    pub bounds: Bounds<T>,
}

impl<T> Aspect<T>
where
    T: Clone + Send + Sync + 'static,
{
    /// Create a new Aspect from blueprint
    pub fn from_blueprint(blueprint: AspectBlueprint) -> Result<Self, String>
    where
        T: Clone,
    {
        // Verify type compatibility
        if blueprint.default_type_id != TypeId::of::<T>() {
            return Err(format!(
                "Type mismatch: blueprint type is {}, but expected {}",
                blueprint.default_type_name,
                std::any::type_name::<T>()
            ));
        }

        // Extract default value
        let default_value = blueprint.default_value
            .downcast::<T>()
            .map(|boxed| *boxed)
            .unwrap_or_else(|_| panic!("Failed to extract default value"));

        // Extract bounds if present
        let bounds = if let Some(bounds_blueprint) = blueprint.bounds {
            let has_both = bounds_blueprint.min_value.is_some() && bounds_blueprint.max_value.is_some();
            let has_min = bounds_blueprint.min_value.is_some();
            let has_max = bounds_blueprint.max_value.is_some();

            if has_both {
                let min_value = bounds_blueprint.min_value.unwrap();
                let max_value = bounds_blueprint.max_value.unwrap();
                let min = min_value.downcast::<T>().map(|boxed| *boxed).unwrap_or_else(|_| panic!("Failed to extract min bound"));
                let max = max_value.downcast::<T>().map(|boxed| *boxed).unwrap_or_else(|_| panic!("Failed to extract max bound"));
                Bounds::new().with_range(min, max)
            } else if has_min {
                let min_value = bounds_blueprint.min_value.unwrap();
                let min = min_value.downcast::<T>().map(|boxed| *boxed).unwrap_or_else(|_| panic!("Failed to extract min bound"));
                Bounds::new().with_min(min)
            } else if has_max {
                let max_value = bounds_blueprint.max_value.unwrap();
                let max = max_value.downcast::<T>().map(|boxed| *boxed).unwrap_or_else(|_| panic!("Failed to extract max bound"));
                Bounds::new().with_max(max)
            } else {
                Bounds::new()
            }
        } else {
            Bounds::new()
        };

        Ok(Self {
            id: blueprint.id,
            name: blueprint.name,
            default_value,
            bounds,
        })
    }

    /// Create a new Aspect directly
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

// ============================================================================
// STATE - Runtime state container
// ============================================================================

/// Represents the complete system state as a high-dimensional vector
#[derive(Debug, Default)]
pub struct State {
    /// Map from aspect ID to its current value (type-erased)
    values: HashMap<AspectId, Box<dyn Any + Send + Sync>>,
}

impl Clone for State {
    fn clone(&self) -> Self {
        let mut new_state = State::new();
        for (key, value) in &self.values {
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
                if value.type_id() != other_value.type_id() {
                    return false;
                }
                if let (Some(a), Some(b)) = (
                    value.downcast_ref::<bool>(),
                    other_value.downcast_ref::<bool>(),
                ) {
                    if a != b { return false; }
                } else if let (Some(a), Some(b)) = (
                    value.downcast_ref::<i64>(),
                    other_value.downcast_ref::<i64>(),
                ) {
                    if a != b { return false; }
                } else if let (Some(a), Some(b)) = (
                    value.downcast_ref::<f64>(),
                    other_value.downcast_ref::<f64>(),
                ) {
                    if a != b { return false; }
                } else if let (Some(a), Some(b)) = (
                    value.downcast_ref::<String>(),
                    other_value.downcast_ref::<String>(),
                ) {
                    if a != b { return false; }
                } else if let (Some(a), Some(b)) = (
                    value.downcast_ref::<i32>(),
                    other_value.downcast_ref::<i32>(),
                ) {
                    if a != b { return false; }
                }
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

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_aspect_blueprint_creation() {
        let id = AspectId(0);
        let blueprint = AspectBlueprint::new(id, "counter", 0i32);

        assert_eq!(blueprint.id, id);
        assert_eq!(blueprint.name, "counter");
        assert_eq!(blueprint.default_type_id, TypeId::of::<i32>());
        assert!(blueprint.bounds.is_none());
    }

    #[test]
    fn test_aspect_blueprint_with_bounds() {
        let id = AspectId(0);
        let blueprint = AspectBlueprint::new(id, "temperature", 20i32)
            .with_range(0, 100);

        assert!(blueprint.bounds.is_some());
        let bounds = blueprint.bounds.unwrap();
        assert_eq!(bounds.type_id, TypeId::of::<i32>());
        assert!(bounds.min_value.is_some());
        assert!(bounds.max_value.is_some());
    }

    #[test]
    fn test_aspect_from_blueprint() {
        let id = AspectId(0);
        let blueprint = AspectBlueprint::new(id, "counter", 0i32)
            .with_range(0, 100);

        let aspect: Aspect<i32> = Aspect::from_blueprint(blueprint).unwrap();

        assert_eq!(aspect.id, id);
        assert_eq!(aspect.name, "counter");
        assert_eq!(aspect.default_value, 0);
        assert_eq!(aspect.bounds.min, Some(0));
        assert_eq!(aspect.bounds.max, Some(100));
    }

    #[test]
    fn test_aspect_creation() {
        let id = AspectId(0);
        let aspect = Aspect::new(id, "counter", 0i32);

        assert_eq!(aspect.id, id);
        assert_eq!(aspect.name, "counter");
        assert_eq!(aspect.default_value, 0);
        assert!(aspect.bounds.min.is_none());
        assert!(aspect.bounds.max.is_none());
    }

    #[test]
    fn test_aspect_with_bounds() {
        let id = AspectId(0);
        let aspect: Aspect<i32> = Aspect::new(id, "temperature", 20)
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
        let aspect: Aspect<Temperature> = Aspect::new(
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
        let aspect: Aspect<i32> = Aspect::new(id, "value", 50)
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

    #[test]
    fn test_aspect_blueprint_type_mismatch() {
        let id = AspectId(0);
        let blueprint = AspectBlueprint::new(id, "counter", 42i32);

        let result: Result<Aspect<f64>, _> = Aspect::from_blueprint(blueprint);
        assert!(result.is_err());
    }
}