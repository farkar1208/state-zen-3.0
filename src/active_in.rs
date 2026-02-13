use crate::aspect::{AspectId, State};
use std::sync::Arc;

/// A predicate function that evaluates whether a behavior is active in a given state
pub type Predicate = Arc<dyn Fn(&State) -> bool + Send + Sync>;

/// Represents the activeIn condition for a behavior (Zone or Transition)
#[derive(Clone)]
pub struct ActiveIn {
    predicate: Predicate,
}

impl ActiveIn {
    /// Create a new ActiveIn from a predicate function
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&State) -> bool + Send + Sync + 'static,
    {
        Self {
            predicate: Arc::new(f),
        }
    }

    /// Evaluate whether this ActiveIn is true for the given state
    pub fn evaluate(&self, state: &State) -> bool {
        (self.predicate)(state)
    }

    /// Always true predicate
    pub fn always() -> Self {
        Self::new(|_| true)
    }

    /// Always false predicate
    pub fn never() -> Self {
        Self::new(|_| false)
    }

    /// Check if an aspect has a specific boolean value
    pub fn aspect_bool(aspect_id: AspectId, value: bool) -> Self {
        Self::new(move |state| {
            state
                .get_as::<bool>(aspect_id)
                .map_or(false, |b| *b == value)
        })
    }

    /// Check if an aspect has a specific integer value (i64)
    pub fn aspect_eq(aspect_id: AspectId, value: i64) -> Self {
        Self::new(move |state| {
            state
                .get_as::<i64>(aspect_id)
                .map_or(false, |i| *i == value)
        })
    }

    /// Check if an aspect integer is less than a value
    pub fn aspect_lt(aspect_id: AspectId, value: i64) -> Self {
        Self::new(move |state| {
            state
                .get_as::<i64>(aspect_id)
                .map_or(false, |i| *i < value)
        })
    }

    /// Check if an aspect integer is greater than a value
    pub fn aspect_gt(aspect_id: AspectId, value: i64) -> Self {
        Self::new(move |state| {
            state
                .get_as::<i64>(aspect_id)
                .map_or(false, |i| *i > value)
        })
    }

    /// Check if an aspect integer is in a range
    pub fn aspect_in_range(aspect_id: AspectId, min: i64, max: i64) -> Self {
        Self::new(move |state| {
            state
                .get_as::<i64>(aspect_id)
                .map_or(false, |i| *i >= min && *i <= max)
        })
    }

    /// Check if an aspect string equals a value
    pub fn aspect_string_eq(aspect_id: AspectId, value: impl Into<String> + Clone) -> Self {
        let value = value.into();
        Self::new(move |state| {
            state
                .get_as::<String>(aspect_id)
                .map_or(false, |s| *s == value)
        })
    }

    /// Generic comparison for any PartialOrd type
    pub fn aspect_lt_typed<T>(aspect_id: AspectId, value: T) -> Self
    where
        T: std::cmp::PartialOrd + Send + Sync + 'static,
    {
        Self::new(move |state| {
            state
                .get_as::<T>(aspect_id)
                .map_or(false, |v| *v < value)
        })
    }

    /// Generic comparison for any PartialOrd type
    pub fn aspect_gt_typed<T>(aspect_id: AspectId, value: T) -> Self
    where
        T: std::cmp::PartialOrd + Send + Sync + 'static,
    {
        Self::new(move |state| {
            state
                .get_as::<T>(aspect_id)
                .map_or(false, |v| *v > value)
        })
    }

    /// Generic comparison for any PartialEq type
    pub fn aspect_eq_typed<T>(aspect_id: AspectId, value: T) -> Self
    where
        T: std::cmp::PartialEq + Send + Sync + 'static,
    {
        Self::new(move |state| {
            state
                .get_as::<T>(aspect_id)
                .map_or(false, |v| *v == value)
        })
    }

    /// Logical AND of two predicates
    pub fn and(self, other: ActiveIn) -> Self {
        Self::new(move |state| self.evaluate(state) && other.evaluate(state))
    }

    /// Logical OR of two predicates
    pub fn or(self, other: ActiveIn) -> Self {
        Self::new(move |state| self.evaluate(state) || other.evaluate(state))
    }

    /// Logical NOT of a predicate
    pub fn not(self) -> Self {
        Self::new(move |state| !self.evaluate(state))
    }

    /// Logical AND of multiple predicates
    pub fn all(predicates: Vec<ActiveIn>) -> Self {
        Self::new(move |state| predicates.iter().all(|p| p.evaluate(state)))
    }

    /// Logical OR of multiple predicates
    pub fn any(predicates: Vec<ActiveIn>) -> Self {
        Self::new(move |state| predicates.iter().any(|p| p.evaluate(state)))
    }
}

/// Builder for constructing ActiveIn predicates
pub struct ActiveInBuilder {
    predicates: Vec<ActiveIn>,
    op: BuilderOp,
}

#[derive(Clone, Copy)]
enum BuilderOp {
    And,
    Or,
}

impl ActiveInBuilder {
    pub fn new() -> Self {
        Self {
            predicates: Vec::new(),
            op: BuilderOp::And,
        }
    }

    pub fn with_all() -> Self {
        Self {
            predicates: Vec::new(),
            op: BuilderOp::And,
        }
    }

    pub fn with_any() -> Self {
        Self {
            predicates: Vec::new(),
            op: BuilderOp::Or,
        }
    }

    pub fn add(mut self, predicate: ActiveIn) -> Self {
        self.predicates.push(predicate);
        self
    }

    pub fn aspect_bool(self, aspect_id: AspectId, value: bool) -> Self {
        self.add(ActiveIn::aspect_bool(aspect_id, value))
    }

    pub fn aspect_eq(self, aspect_id: AspectId, value: i64) -> Self {
        self.add(ActiveIn::aspect_eq(aspect_id, value))
    }

    pub fn aspect_lt(self, aspect_id: AspectId, value: i64) -> Self {
        self.add(ActiveIn::aspect_lt(aspect_id, value))
    }

    pub fn aspect_gt(self, aspect_id: AspectId, value: i64) -> Self {
        self.add(ActiveIn::aspect_gt(aspect_id, value))
    }

    pub fn aspect_in_range(self, aspect_id: AspectId, min: i64, max: i64) -> Self {
        self.add(ActiveIn::aspect_in_range(aspect_id, min, max))
    }

    pub fn aspect_string_eq(self, aspect_id: AspectId, value: impl Into<String> + Clone) -> Self {
        self.add(ActiveIn::aspect_string_eq(aspect_id, value))
    }

    /// Generic typed comparison
    pub fn aspect_eq_typed<T>(self, aspect_id: AspectId, value: T) -> Self
    where
        T: std::cmp::PartialEq + Send + Sync + 'static,
    {
        self.add(ActiveIn::aspect_eq_typed(aspect_id, value))
    }

    pub fn aspect_lt_typed<T>(self, aspect_id: AspectId, value: T) -> Self
    where
        T: std::cmp::PartialOrd + Send + Sync + 'static,
    {
        self.add(ActiveIn::aspect_lt_typed(aspect_id, value))
    }

    pub fn aspect_gt_typed<T>(self, aspect_id: AspectId, value: T) -> Self
    where
        T: std::cmp::PartialOrd + Send + Sync + 'static,
    {
        self.add(ActiveIn::aspect_gt_typed(aspect_id, value))
    }

    pub fn build(self) -> ActiveIn {
        match self.op {
            BuilderOp::And => ActiveIn::all(self.predicates),
            BuilderOp::Or => ActiveIn::any(self.predicates),
        }
    }
}

impl Default for ActiveInBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aspect::StateBuilder;

    #[test]
    fn test_active_in_bool() {
        let id = AspectId(0);
        let state = StateBuilder::new()
            .set_bool(id, true)
            .build();

        let active_in = ActiveIn::aspect_bool(id, true);
        assert!(active_in.evaluate(&state));

        let active_in = ActiveIn::aspect_bool(id, false);
        assert!(!active_in.evaluate(&state));
    }

    #[test]
    fn test_active_in_composition() {
        let id1 = AspectId(0);
        let id2 = AspectId(1);

        let state = StateBuilder::new()
            .set_bool(id1, true)
            .set_int(id2, 5)
            .build();

        let active_in = ActiveIn::aspect_bool(id1, true)
            .and(ActiveIn::aspect_lt(id2, 10));
        assert!(active_in.evaluate(&state));

        let active_in = ActiveIn::aspect_bool(id1, false)
            .or(ActiveIn::aspect_gt(id2, 0));
        assert!(active_in.evaluate(&state));
    }

    #[test]
    fn test_active_in_builder() {
        let id1 = AspectId(0);
        let id2 = AspectId(1);

        let state = StateBuilder::new()
            .set_bool(id1, true)
            .set_int(id2, 5)
            .build();

        let active_in = ActiveInBuilder::with_all()
            .aspect_bool(id1, true)
            .aspect_lt(id2, 10)
            .build();

        assert!(active_in.evaluate(&state));
    }

    #[test]
    fn test_active_in_typed() {
        let id = AspectId(0);
        let state = StateBuilder::new()
            .set_typed(id, 42i32)
            .build();

        let active_in = ActiveIn::aspect_eq_typed(id, 42i32);
        assert!(active_in.evaluate(&state));

        let active_in = ActiveIn::aspect_lt_typed(id, 50i32);
        assert!(active_in.evaluate(&state));

        let active_in = ActiveIn::aspect_gt_typed(id, 30i32);
        assert!(active_in.evaluate(&state));
    }
}