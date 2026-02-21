use crate::aspect::{AspectId, State};
use std::ops::Not;
use std::sync::Arc;

// ============================================================================
// BLUEPRINT LAYER - AST for ActiveIn predicates
// ============================================================================

/// AST node types for ActiveIn predicates (blueprint layer)
#[derive(Debug, Clone)]
pub enum ActiveInBlueprint {
    /// Always evaluates to true
    Always,

    /// Always evaluates to false
    Never,

    /// Check if a boolean aspect equals a value
    AspectBool { aspect_id: AspectId, value: bool },

    /// Check if an i64 aspect equals a value
    AspectEq { aspect_id: AspectId, value: i64 },

    /// Check if an i64 aspect is less than a value
    AspectLt { aspect_id: AspectId, value: i64 },

    /// Check if an i64 aspect is greater than a value
    AspectGt { aspect_id: AspectId, value: i64 },

    /// Check if an i64 aspect is in a range [min, max]
    AspectInRange { aspect_id: AspectId, min: i64, max: i64 },

    /// Check if a String aspect equals a value
    AspectStringEq { aspect_id: AspectId, value: String },

    /// Logical AND of multiple predicates
    And(Vec<ActiveInBlueprint>),

    /// Logical OR of multiple predicates
    Or(Vec<ActiveInBlueprint>),

    /// Logical NOT of a predicate
    Not(Box<ActiveInBlueprint>),
}

impl ActiveInBlueprint {
    /// Create an always-true predicate
    pub fn always() -> Self {
        ActiveInBlueprint::Always
    }

    /// Create an always-false predicate
    pub fn never() -> Self {
        ActiveInBlueprint::Never
    }

    /// Check if a boolean aspect equals a value
    pub fn aspect_bool(aspect_id: AspectId, value: bool) -> Self {
        ActiveInBlueprint::AspectBool { aspect_id, value }
    }

    /// Check if an i64 aspect equals a value
    pub fn aspect_eq(aspect_id: AspectId, value: i64) -> Self {
        ActiveInBlueprint::AspectEq { aspect_id, value }
    }

    /// Check if an i64 aspect is less than a value
    pub fn aspect_lt(aspect_id: AspectId, value: i64) -> Self {
        ActiveInBlueprint::AspectLt { aspect_id, value }
    }

    /// Check if an i64 aspect is greater than a value
    pub fn aspect_gt(aspect_id: AspectId, value: i64) -> Self {
        ActiveInBlueprint::AspectGt { aspect_id, value }
    }

    /// Check if an i64 aspect is in a range
    pub fn aspect_in_range(aspect_id: AspectId, min: i64, max: i64) -> Self {
        ActiveInBlueprint::AspectInRange { aspect_id, min, max }
    }

    /// Check if a String aspect equals a value
    pub fn aspect_string_eq(aspect_id: AspectId, value: impl Into<String>) -> Self {
        ActiveInBlueprint::AspectStringEq {
            aspect_id,
            value: value.into(),
        }
    }

    /// Logical AND of two predicates
    pub fn and(self, other: ActiveInBlueprint) -> Self {
        match (self, other) {
            (ActiveInBlueprint::And(mut preds), ActiveInBlueprint::And(mut other_preds)) => {
                preds.append(&mut other_preds);
                ActiveInBlueprint::And(preds)
            }
            (ActiveInBlueprint::And(mut preds), other) => {
                preds.push(other);
                ActiveInBlueprint::And(preds)
            }
            (this, ActiveInBlueprint::And(mut preds)) => {
                preds.insert(0, this);
                ActiveInBlueprint::And(preds)
            }
            (this, other) => ActiveInBlueprint::And(vec![this, other]),
        }
    }

    /// Logical OR of two predicates
    pub fn or(self, other: ActiveInBlueprint) -> Self {
        match (self, other) {
            (ActiveInBlueprint::Or(mut preds), ActiveInBlueprint::Or(mut other_preds)) => {
                preds.append(&mut other_preds);
                ActiveInBlueprint::Or(preds)
            }
            (ActiveInBlueprint::Or(mut preds), other) => {
                preds.push(other);
                ActiveInBlueprint::Or(preds)
            }
            (this, ActiveInBlueprint::Or(mut preds)) => {
                preds.insert(0, this);
                ActiveInBlueprint::Or(preds)
            }
            (this, other) => ActiveInBlueprint::Or(vec![this, other]),
        }
    }

    /// Logical AND of multiple predicates (flattened)
    pub fn all(predicates: Vec<ActiveInBlueprint>) -> Self {
        ActiveInBlueprint::And(predicates)
    }

    /// Logical OR of multiple predicates (flattened)
    pub fn any(predicates: Vec<ActiveInBlueprint>) -> Self {
        ActiveInBlueprint::Or(predicates)
    }
}

impl Not for ActiveInBlueprint {
    type Output = Self;

    fn not(self) -> Self {
        ActiveInBlueprint::Not(Box::new(self))
    }
}

// ============================================================================
// RUNTIME LAYER - Compiled predicates with closure-based evaluation
// ============================================================================

/// A predicate function that evaluates whether a behavior is active in a given state
pub type Predicate = Arc<dyn Fn(&State) -> bool + Send + Sync>;

/// Represents the activeIn condition for a behavior (Zone or Transition) - Runtime layer
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

    /// Compile an ActiveInBlueprint into a runtime ActiveIn (using closure nesting)
    pub fn from_blueprint(blueprint: ActiveInBlueprint) -> Self {
        Self::new(move |state| evaluate_blueprint(&blueprint, state))
    }

    /// Evaluate whether this ActiveIn is true for the given state
    pub fn evaluate(&self, state: &State) -> bool {
        (self.predicate)(state)
    }
}

// ============================================================================
// FACTORY - Create runtime ActiveIn predicates
// ============================================================================

/// Factory for creating runtime ActiveIn predicates
pub struct ActiveInFactory;

impl ActiveInFactory {
    /// Create an always-true predicate
    pub fn always() -> ActiveIn {
        ActiveIn::new(|_| true)
    }

    /// Create an always-false predicate
    pub fn never() -> ActiveIn {
        ActiveIn::new(|_| false)
    }

    /// Check if an aspect has a specific boolean value
    pub fn aspect_bool(aspect_id: AspectId, value: bool) -> ActiveIn {
        ActiveIn::new(move |state| {
            state
                .get_as::<bool>(aspect_id)
                .map_or(false, |b| *b == value)
        })
    }

    /// Check if an aspect has a specific integer value (i64)
    pub fn aspect_eq(aspect_id: AspectId, value: i64) -> ActiveIn {
        ActiveIn::new(move |state| {
            state
                .get_as::<i64>(aspect_id)
                .map_or(false, |i| *i == value)
        })
    }

    /// Check if an aspect integer is less than a value
    pub fn aspect_lt(aspect_id: AspectId, value: i64) -> ActiveIn {
        ActiveIn::new(move |state| {
            state
                .get_as::<i64>(aspect_id)
                .map_or(false, |i| *i < value)
        })
    }

    /// Check if an aspect integer is greater than a value
    pub fn aspect_gt(aspect_id: AspectId, value: i64) -> ActiveIn {
        ActiveIn::new(move |state| {
            state
                .get_as::<i64>(aspect_id)
                .map_or(false, |i| *i > value)
        })
    }

    /// Check if an aspect integer is in a range
    pub fn aspect_in_range(aspect_id: AspectId, min: i64, max: i64) -> ActiveIn {
        ActiveIn::new(move |state| {
            state
                .get_as::<i64>(aspect_id)
                .map_or(false, |i| *i >= min && *i <= max)
        })
    }

    /// Check if an aspect string equals a value
    pub fn aspect_string_eq(aspect_id: AspectId, value: impl Into<String> + Clone) -> ActiveIn {
        let value = value.into();
        ActiveIn::new(move |state| {
            state
                .get_as::<String>(aspect_id)
                .map_or(false, |s| *s == value)
        })
    }

    /// Generic comparison for any PartialOrd type
    pub fn aspect_lt_typed<T>(aspect_id: AspectId, value: T) -> ActiveIn
    where
        T: std::cmp::PartialOrd + Send + Sync + 'static,
    {
        ActiveIn::new(move |state| {
            state
                .get_as::<T>(aspect_id)
                .map_or(false, |v| *v < value)
        })
    }

    /// Generic comparison for any PartialOrd type
    pub fn aspect_gt_typed<T>(aspect_id: AspectId, value: T) -> ActiveIn
    where
        T: std::cmp::PartialOrd + Send + Sync + 'static,
    {
        ActiveIn::new(move |state| {
            state
                .get_as::<T>(aspect_id)
                .map_or(false, |v| *v > value)
        })
    }

    /// Generic comparison for any PartialEq type
    pub fn aspect_eq_typed<T>(aspect_id: AspectId, value: T) -> ActiveIn
    where
        T: std::cmp::PartialEq + Send + Sync + 'static,
    {
        ActiveIn::new(move |state| {
            state
                .get_as::<T>(aspect_id)
                .map_or(false, |v| *v == value)
        })
    }

    /// Logical AND of two predicates
    pub fn and(left: ActiveIn, right: ActiveIn) -> ActiveIn {
        ActiveIn::new(move |state| left.evaluate(state) && right.evaluate(state))
    }

    /// Logical OR of two predicates
    pub fn or(left: ActiveIn, right: ActiveIn) -> ActiveIn {
        ActiveIn::new(move |state| left.evaluate(state) || right.evaluate(state))
    }

    /// Logical AND of multiple predicates
    pub fn all(predicates: Vec<ActiveIn>) -> ActiveIn {
        ActiveIn::new(move |state| predicates.iter().all(|p| p.evaluate(state)))
    }

    /// Logical OR of multiple predicates
    pub fn any(predicates: Vec<ActiveIn>) -> ActiveIn {
        ActiveIn::new(move |state| predicates.iter().any(|p| p.evaluate(state)))
    }

    /// Logical NOT of a predicate
    pub fn not(predicate: ActiveIn) -> ActiveIn {
        ActiveIn::new(move |state| !predicate.evaluate(state))
    }
}

/// Evaluate an ActiveInBlueprint AST and return the result
pub(crate) fn evaluate_blueprint(blueprint: &ActiveInBlueprint, state: &State) -> bool {
    match blueprint {
        ActiveInBlueprint::Always => true,
        ActiveInBlueprint::Never => false,
        ActiveInBlueprint::AspectBool { aspect_id, value } => state
            .get_as::<bool>(*aspect_id)
            .map_or(false, |b| *b == *value),
        ActiveInBlueprint::AspectEq { aspect_id, value } => state
            .get_as::<i64>(*aspect_id)
            .map_or(false, |i| *i == *value),
        ActiveInBlueprint::AspectLt { aspect_id, value } => state
            .get_as::<i64>(*aspect_id)
            .map_or(false, |i| *i < *value),
        ActiveInBlueprint::AspectGt { aspect_id, value } => state
            .get_as::<i64>(*aspect_id)
            .map_or(false, |i| *i > *value),
        ActiveInBlueprint::AspectInRange { aspect_id, min, max } => state
            .get_as::<i64>(*aspect_id)
            .map_or(false, |i| *i >= *min && *i <= *max),
        ActiveInBlueprint::AspectStringEq { aspect_id, value } => state
            .get_as::<String>(*aspect_id)
            .map_or(false, |s| *s == *value),
        ActiveInBlueprint::And(predicates) => {
            predicates.iter().all(|p| evaluate_blueprint(p, state))
        }
        ActiveInBlueprint::Or(predicates) => {
            predicates.iter().any(|p| evaluate_blueprint(p, state))
        }
        ActiveInBlueprint::Not(predicate) => !evaluate_blueprint(predicate, state),
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aspect::StateBuilder;

    #[test]
    fn test_blueprint_always_never() {
        assert!(evaluate_blueprint(&ActiveInBlueprint::always(), &State::new()));
        assert!(!evaluate_blueprint(&ActiveInBlueprint::never(), &State::new()));
    }

    #[test]
    fn test_blueprint_aspect_bool() {
        let id = AspectId(0);
        let state = StateBuilder::new()
            .set_bool(id, true)
            .build();

        let blueprint = ActiveInBlueprint::aspect_bool(id, true);
        assert!(evaluate_blueprint(&blueprint, &state));

        let blueprint = ActiveInBlueprint::aspect_bool(id, false);
        assert!(!evaluate_blueprint(&blueprint, &state));
    }

    #[test]
    fn test_blueprint_composition() {
        let id1 = AspectId(0);
        let id2 = AspectId(1);

        let state = StateBuilder::new()
            .set_bool(id1, true)
            .set_int(id2, 5)
            .build();

        let blueprint = ActiveInBlueprint::aspect_bool(id1, true)
            .and(ActiveInBlueprint::aspect_lt(id2, 10));
        assert!(evaluate_blueprint(&blueprint, &state));

        let blueprint = ActiveInBlueprint::aspect_bool(id1, false)
            .or(ActiveInBlueprint::aspect_gt(id2, 0));
        assert!(evaluate_blueprint(&blueprint, &state));
    }

    #[test]
    fn test_blueprint_not() {
        let id = AspectId(0);
        let state = StateBuilder::new()
            .set_bool(id, true)
            .build();

        let blueprint = ActiveInBlueprint::aspect_bool(id, true).not();
        assert!(!evaluate_blueprint(&blueprint, &state));
    }

    #[test]
    fn test_runtime_from_blueprint() {
        let id = AspectId(0);
        let state = StateBuilder::new()
            .set_bool(id, true)
            .build();

        let blueprint = ActiveInBlueprint::aspect_bool(id, true);
        let runtime = ActiveIn::from_blueprint(blueprint);

        assert!(runtime.evaluate(&state));
    }

    #[test]
    fn test_runtime_factory_api() {
        let id = AspectId(0);
        let state = StateBuilder::new()
            .set_bool(id, true)
            .build();

        let active_in = ActiveInFactory::aspect_bool(id, true);
        assert!(active_in.evaluate(&state));

        let active_in = ActiveInFactory::aspect_bool(id, false);
        assert!(!active_in.evaluate(&state));
    }

    #[test]
    fn test_runtime_composition() {
        let id1 = AspectId(0);
        let id2 = AspectId(1);

        let state = StateBuilder::new()
            .set_bool(id1, true)
            .set_int(id2, 5)
            .build();

        let active_in = ActiveInFactory::and(
            ActiveInFactory::aspect_bool(id1, true),
            ActiveInFactory::aspect_lt(id2, 10)
        );
        assert!(active_in.evaluate(&state));

        let active_in = ActiveInFactory::or(
            ActiveInFactory::aspect_bool(id1, false),
            ActiveInFactory::aspect_gt(id2, 0)
        );
        assert!(active_in.evaluate(&state));
    }

    #[test]
    fn test_runtime_factory_not() {
        let id = AspectId(0);
        let state = StateBuilder::new()
            .set_bool(id, true)
            .build();

        // 测试 ActiveInFactory::not()
        let always = ActiveInFactory::always();
        let never = ActiveInFactory::not(always.clone());
        assert!(always.evaluate(&state));
        assert!(!never.evaluate(&state));

        // 测试 !ActiveInBlueprint::never()
        let not_never = ActiveInFactory::not(ActiveInFactory::never());
        assert!(not_never.evaluate(&state));

        // 测试 !ActiveInBlueprint::aspect_bool()
        let is_true = ActiveInFactory::aspect_bool(id, true);
        let not_true = ActiveInFactory::not(is_true.clone());
        assert!(is_true.evaluate(&state));
        assert!(!not_true.evaluate(&state));

        // 测试双重否定
        let double_negated = ActiveInFactory::not(ActiveInFactory::not(is_true));
        assert!(double_negated.evaluate(&state));
    }
}