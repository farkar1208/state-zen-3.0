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
#[derive(Debug, Clone)]
pub struct AspectBoundsBlueprint {
    pub type_id: TypeId,
    pub type_name: String,
    /// Serialized min value as bytes
    pub min_bytes: Option<Vec<u8>>,
    /// Serialized max value as bytes
    pub max_bytes: Option<Vec<u8>>,
}

impl AspectBoundsBlueprint {
    pub fn new<T: 'static>() -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            type_name: std::any::type_name::<T>().to_string(),
            min_bytes: None,
            max_bytes: None,
        }
    }

    pub fn with_min<T: 'static>(mut self, min: T) -> Self {
        self.min_bytes = Some(serialize_value(&min));
        self
    }

    pub fn with_max<T: 'static>(mut self, max: T) -> Self {
        self.max_bytes = Some(serialize_value(&max));
        self
    }

    pub fn with_range<T: 'static>(mut self, min: T, max: T) -> Self {
        self.min_bytes = Some(serialize_value(&min));
        self.max_bytes = Some(serialize_value(&max));
        self
    }
}

/// Blueprint for defining an Aspect (declaration layer)
///
/// AspectBlueprint describes an orthogonal dimension of the state vector
/// without including validation logic or runtime behavior.
#[derive(Debug, Clone)]
pub struct AspectBlueprint {
    pub id: AspectId,
    pub name: String,
    /// Type-erased default value
    pub default_value_bytes: Vec<u8>,
    pub default_type_id: TypeId,
    pub default_type_name: String,
    /// Type-erased bounds
    pub bounds: Option<AspectBoundsBlueprint>,
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
            default_value_bytes: serialize_value(&default_value),
            default_type_id: TypeId::of::<T>(),
            default_type_name: std::any::type_name::<T>().to_string(),
            bounds: None,
        }
    }

    /// Set bounds for this aspect
    pub fn with_bounds<T: 'static>(mut self, bounds: AspectBoundsBlueprint) -> Self {
        self.bounds = Some(bounds);
        self
    }

    /// Set min and max bounds (convenience method)
    pub fn with_range<T: 'static>(mut self, min: T, max: T) -> Self {
        let bounds = AspectBoundsBlueprint::new::<T>().with_range(min, max);
        self.bounds = Some(bounds);
        self
    }

    /// Set min bound only
    pub fn with_min<T: 'static>(mut self, min: T) -> Self {
        let bounds = AspectBoundsBlueprint::new::<T>().with_min(min);
        self.bounds = Some(bounds);
        self
    }

    /// Set max bound only
    pub fn with_max<T: 'static>(mut self, max: T) -> Self {
        let bounds = AspectBoundsBlueprint::new::<T>().with_max(max);
        self.bounds = Some(bounds);
        self
    }
}

/// Builder for constructing AspectBlueprint instances
pub struct AspectBlueprintBuilder {
    id: Option<AspectId>,
    name: Option<String>,
    default_value_bytes: Option<Vec<u8>>,
    default_type_id: Option<TypeId>,
    default_type_name: Option<String>,
    bounds: Option<AspectBoundsBlueprint>,
}

impl AspectBlueprintBuilder {
    pub fn new() -> Self {
        Self {
            id: None,
            name: None,
            default_value_bytes: None,
            default_type_id: None,
            default_type_name: None,
            bounds: None,
        }
    }

    pub fn id(mut self, id: AspectId) -> Self {
        self.id = Some(id);
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn default_value<T: Any + Send + Sync + 'static>(mut self, value: T) -> Self {
        self.default_value_bytes = Some(serialize_value(&value));
        self.default_type_id = Some(TypeId::of::<T>());
        self.default_type_name = Some(std::any::type_name::<T>().to_string());
        self
    }

    pub fn bounds(mut self, bounds: AspectBoundsBlueprint) -> Self {
        self.bounds = Some(bounds);
        self
    }

    pub fn build(self) -> Result<AspectBlueprint, String> {
        let id = self.id.ok_or("Aspect id is required")?;
        let name = self.name.ok_or("Aspect name is required")?;
        let default_value_bytes = self.default_value_bytes.ok_or("Default value is required")?;
        let default_type_id = self.default_type_id.ok_or("Default type is required")?;
        let default_type_name = self.default_type_name.ok_or("Default type name is required")?;

        Ok(AspectBlueprint {
            id,
            name,
            default_value_bytes,
            default_type_id,
            default_type_name,
            bounds: self.bounds,
        })
    }
}

impl Default for AspectBlueprintBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Serialize a value to bytes
fn serialize_value<T: ?Sized>(value: &T) -> Vec<u8> {
    unsafe {
        let bytes = std::slice::from_raw_parts(
            value as *const T as *const u8,
            std::mem::size_of_val(value),
        );
        bytes.to_vec()
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

        // Deserialize default value
        let default_value = deserialize_bytes::<T>(&blueprint.default_value_bytes)
            .ok_or("Failed to deserialize default value")?
            .clone();

        // Deserialize bounds if present
        let bounds = if let Some(bounds_blueprint) = blueprint.bounds {
            // Check if both min and max are present
            let has_both = bounds_blueprint.min_bytes.is_some() && bounds_blueprint.max_bytes.is_some();
            let has_min = bounds_blueprint.min_bytes.is_some();
            let has_max = bounds_blueprint.max_bytes.is_some();

            if has_both {
                let min_bytes = bounds_blueprint.min_bytes.unwrap();
                let max_bytes = bounds_blueprint.max_bytes.unwrap();
                let min = deserialize_bytes::<T>(&min_bytes).ok_or("Failed to deserialize min bound")?;
                let max = deserialize_bytes::<T>(&max_bytes).ok_or("Failed to deserialize max bound")?;
                Bounds::new().with_range(min.clone(), max.clone())
            } else if has_min {
                let min_bytes = bounds_blueprint.min_bytes.unwrap();
                let min = deserialize_bytes::<T>(&min_bytes).ok_or("Failed to deserialize min bound")?;
                Bounds::new().with_min(min.clone())
            } else if has_max {
                let max_bytes = bounds_blueprint.max_bytes.unwrap();
                let max = deserialize_bytes::<T>(&max_bytes).ok_or("Failed to deserialize max bound")?;
                Bounds::new().with_max(max.clone())
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

/// Deserialize bytes back to a value reference
fn deserialize_bytes<T>(bytes: &[u8]) -> Option<&T> {
    if bytes.len() != std::mem::size_of::<T>() {
        return None;
    }
    unsafe {
        Some(&*(bytes.as_ptr() as *const T))
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

// ============================================================================
// TESTS
// ============================================================================

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
        assert!(bounds.min_bytes.is_some());
        assert!(bounds.max_bytes.is_some());
    }

    #[test]
    fn test_aspect_blueprint_builder() {
        let id = AspectId(0);
        let blueprint = AspectBlueprintBuilder::new()
            .id(id)
            .name("counter")
            .default_value(42i32)
            .bounds(AspectBoundsBlueprint::new::<i32>().with_range(0, 100))
            .build()
            .unwrap();

        assert_eq!(blueprint.id, id);
        assert_eq!(blueprint.name, "counter");
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

        // Trying to create Aspect<f64> from i32 blueprint should fail
        let result: Result<Aspect<f64>, _> = Aspect::from_blueprint(blueprint);
        assert!(result.is_err());
    }
}