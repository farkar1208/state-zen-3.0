use std::any::Any;

// ============================================================================
// CLONABLE ANY TRAIT - Type-safe clone and compare for type-erased values
// ============================================================================

/// Trait for type-erased values that support cloning and equality comparison.
///
/// This trait is automatically implemented for all types that satisfy:
/// - Any + Send + Sync + Clone + PartialEq + 'static
///
/// Users can enable support for custom types by simply deriving Clone and PartialEq:
/// ```rust
/// #[derive(Clone, PartialEq)]
/// struct MyType { field: i32 }
/// ```
pub trait ClonableAny: Any + Send + Sync + std::fmt::Debug {
    /// Clone this type-erased value into a new boxed value
    fn clone_box(&self) -> Box<dyn ClonableAny>;

    /// Compare this type-erased value with another for equality
    ///
    /// Returns false if the other value is of a different type
    fn eq_box(&self, other: &dyn ClonableAny) -> bool;

    /// Helper method to access the underlying Any trait for downcasting
    fn as_any(&self) -> &dyn Any;
}

// Blanket implementation for all types that satisfy the bounds
impl<T: Any + Send + Sync + Clone + PartialEq + std::fmt::Debug + 'static> ClonableAny for T {
    fn clone_box(&self) -> Box<dyn ClonableAny> {
        Box::new(self.clone())
    }

    fn eq_box(&self, other: &dyn ClonableAny) -> bool {
        // Use Any::downcast_ref from the Any trait
        other
            .as_any()
            .downcast_ref::<T>()
            .map(|other| self == other)
            .unwrap_or(false)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ============================================================================
// ID TYPES
// ============================================================================

/// Unique identifier for an Aspect
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AspectId(pub usize);

/// Unique identifier for an event type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EventId(pub String);

impl EventId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clonable_any_bool() {
        let value = true;
        let boxed: Box<dyn ClonableAny> = Box::new(value);

        let cloned = boxed.clone_box();
        assert!(cloned.eq_box(boxed.as_ref()));
    }

    #[test]
    fn test_clonable_any_int() {
        let value = 42i32;
        let boxed: Box<dyn ClonableAny> = Box::new(value);

        let cloned = boxed.clone_box();
        assert!(cloned.eq_box(boxed.as_ref()));
    }

    #[test]
    fn test_clonable_any_string() {
        let value = "hello".to_string();
        let boxed: Box<dyn ClonableAny> = Box::new(value.clone());

        let cloned = boxed.clone_box();
        assert!(cloned.eq_box(boxed.as_ref()));
    }

    #[test]
    fn test_clonable_any_type_mismatch() {
        let bool_value: Box<dyn ClonableAny> = Box::new(true);
        let int_value: Box<dyn ClonableAny> = Box::new(42i32);

        assert!(!bool_value.eq_box(int_value.as_ref()));
    }

    #[test]
    fn test_aspect_id_copy() {
        let id1 = AspectId(42);
        let id2 = id1;

        assert_eq!(id1, id2);
    }

    #[test]
    fn test_aspect_id_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(AspectId(0));
        set.insert(AspectId(1));
        set.insert(AspectId(0)); // Duplicate

        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_event_id_creation() {
        let event1 = EventId::new("start");
        let event2 = EventId::new(String::from("stop"));

        assert_eq!(event1, EventId("start".to_string()));
        assert_eq!(event2, EventId("stop".to_string()));
    }

    #[test]
    fn test_event_id_equality() {
        let event1 = EventId::new("click");
        let event2 = EventId::new("click");
        let event3 = EventId::new("hover");

        assert_eq!(event1, event2);
        assert_ne!(event1, event3);
    }
}