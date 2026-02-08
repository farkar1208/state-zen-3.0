use crate::aspect::{AspectId, State, StateValue};
use std::sync::Arc;

/// Represents how state evolves in response to events
///
/// An Update is a pure function that receives current state and returns new state.
/// It must be side-effect free; all side effects should be in on_tran handlers.
#[derive(Clone)]
pub struct Update {
    operation: UpdateOp,
}

/// Internal representation of update operations
#[derive(Clone)]
enum UpdateOp {
    Noop,
    Set(AspectId, StateValue),
    Modify(AspectId, Arc<dyn Fn(StateValue) -> StateValue + Send + Sync>),
    Compose(Vec<Update>),
    Conditional {
        predicate: Arc<dyn Fn(&State) -> bool + Send + Sync>,
        then_update: Box<Update>,
        else_update: Option<Box<Update>>,
    },
}

impl Update {
    /// Create a no-op update (state remains unchanged)
    pub fn noop() -> Self {
        Self {
            operation: UpdateOp::Noop,
        }
    }

    /// Set an aspect to a specific value
    pub fn set(aspect_id: AspectId, value: StateValue) -> Self {
        Self {
            operation: UpdateOp::Set(aspect_id, value),
        }
    }

    /// Modify an aspect's value using a transformation function
    pub fn modify<F>(aspect_id: AspectId, f: F) -> Self
    where
        F: Fn(StateValue) -> StateValue + Send + Sync + 'static,
    {
        Self {
            operation: UpdateOp::Modify(aspect_id, Arc::new(f)),
        }
    }

    /// Set a boolean aspect
    pub fn set_bool(aspect_id: AspectId, value: bool) -> Self {
        Self::set(aspect_id, StateValue::Bool(value))
    }

    /// Set an integer aspect
    pub fn set_int(aspect_id: AspectId, value: i64) -> Self {
        Self::set(aspect_id, StateValue::Integer(value))
    }

    /// Set a float aspect
    pub fn set_float(aspect_id: AspectId, value: f64) -> Self {
        Self::set(aspect_id, StateValue::Float(value))
    }

    /// Set a string aspect
    pub fn set_string(aspect_id: AspectId, value: impl Into<String>) -> Self {
        Self::set(aspect_id, StateValue::String(value.into()))
    }

    /// Increment an integer aspect
    pub fn increment(aspect_id: AspectId) -> Self {
        Self::modify(aspect_id, |v| match v {
            StateValue::Integer(i) => StateValue::Integer(i + 1),
            _ => v,
        })
    }

    /// Decrement an integer aspect
    pub fn decrement(aspect_id: AspectId) -> Self {
        Self::modify(aspect_id, |v| match v {
            StateValue::Integer(i) => StateValue::Integer(i - 1),
            _ => v,
        })
    }

    /// Add a delta to an integer aspect
    pub fn add(aspect_id: AspectId, delta: i64) -> Self {
        Self::modify(aspect_id, move |v| match v {
            StateValue::Integer(i) => StateValue::Integer(i + delta),
            _ => v,
        })
    }

    /// Toggle a boolean aspect
    pub fn toggle(aspect_id: AspectId) -> Self {
        Self::modify(aspect_id, |v| match v {
            StateValue::Bool(b) => StateValue::Bool(!b),
            _ => v,
        })
    }

    /// Compose multiple updates to apply sequentially
    pub fn compose(updates: Vec<Update>) -> Self {
        if updates.is_empty() {
            Self::noop()
        } else if updates.len() == 1 {
            updates.into_iter().next().unwrap()
        } else {
            Self {
                operation: UpdateOp::Compose(updates),
            }
        }
    }

    /// Create a conditional update based on state
    pub fn conditional<F>(predicate: F, then_update: Update) -> Self
    where
        F: Fn(&State) -> bool + Send + Sync + 'static,
    {
        Self {
            operation: UpdateOp::Conditional {
                predicate: Arc::new(predicate),
                then_update: Box::new(then_update),
                else_update: None,
            },
        }
    }

    /// Create a conditional update with else branch
    pub fn conditional_else<F>(
        predicate: F,
        then_update: Update,
        else_update: Update,
    ) -> Self
    where
        F: Fn(&State) -> bool + Send + Sync + 'static,
    {
        Self {
            operation: UpdateOp::Conditional {
                predicate: Arc::new(predicate),
                then_update: Box::new(then_update),
                else_update: Some(Box::new(else_update)),
            },
        }
    }

    /// Apply this update to a state, returning a new state
    pub fn apply(&self, state: State) -> State {
        match &self.operation {
            UpdateOp::Noop => state,
            UpdateOp::Set(aspect_id, value) => state.set(*aspect_id, value.clone()),
            UpdateOp::Modify(aspect_id, f) => {
                if let Some(current) = state.get(*aspect_id) {
                    state.set(*aspect_id, f(current.clone()))
                } else {
                    state
                }
            }
            UpdateOp::Compose(updates) => {
                updates
                    .iter()
                    .fold(state, |acc, update| update.apply(acc))
            }
            UpdateOp::Conditional {
                predicate,
                then_update,
                else_update,
            } => {
                if predicate(&state) {
                    then_update.apply(state)
                } else if let Some(else_update) = else_update {
                    else_update.apply(state)
                } else {
                    state
                }
            }
        }
    }
}

/// Builder for constructing Update instances
pub struct UpdateBuilder {
    operations: Vec<Update>,
}

impl UpdateBuilder {
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
        }
    }

    pub fn set(mut self, aspect_id: AspectId, value: StateValue) -> Self {
        self.operations.push(Update::set(aspect_id, value));
        self
    }

    pub fn set_bool(mut self, aspect_id: AspectId, value: bool) -> Self {
        self.operations.push(Update::set_bool(aspect_id, value));
        self
    }

    pub fn set_int(mut self, aspect_id: AspectId, value: i64) -> Self {
        self.operations.push(Update::set_int(aspect_id, value));
        self
    }

    pub fn set_float(mut self, aspect_id: AspectId, value: f64) -> Self {
        self.operations.push(Update::set_float(aspect_id, value));
        self
    }

    pub fn set_string(mut self, aspect_id: AspectId, value: impl Into<String>) -> Self {
        self.operations.push(Update::set_string(aspect_id, value));
        self
    }

    pub fn increment(mut self, aspect_id: AspectId) -> Self {
        self.operations.push(Update::increment(aspect_id));
        self
    }

    pub fn decrement(mut self, aspect_id: AspectId) -> Self {
        self.operations.push(Update::decrement(aspect_id));
        self
    }

    pub fn add(mut self, aspect_id: AspectId, delta: i64) -> Self {
        self.operations.push(Update::add(aspect_id, delta));
        self
    }

    pub fn toggle(mut self, aspect_id: AspectId) -> Self {
        self.operations.push(Update::toggle(aspect_id));
        self
    }

    pub fn modify<F>(mut self, aspect_id: AspectId, f: F) -> Self
    where
        F: Fn(StateValue) -> StateValue + Send + Sync + 'static,
    {
        self.operations.push(Update::modify(aspect_id, f));
        self
    }

    pub fn build(self) -> Update {
        Update::compose(self.operations)
    }
}

impl Default for UpdateBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aspect::{StateBuilder, StateValue};

    #[test]
    fn test_update_noop() {
        let id = AspectId(0);
        let state = StateBuilder::new()
            .set(id, StateValue::Bool(true))
            .build();

        let new_state = Update::noop().apply(state.clone());

        assert_eq!(new_state, state);
    }

    #[test]
    fn test_update_set() {
        let id = AspectId(0);
        let state = StateBuilder::new()
            .set(id, StateValue::Bool(true))
            .build();

        let new_state = Update::set_bool(id, false).apply(state);

        assert_eq!(new_state.get(id), Some(&StateValue::Bool(false)));
    }

    #[test]
    fn test_update_increment() {
        let id = AspectId(0);
        let state = StateBuilder::new()
            .set(id, StateValue::Integer(5))
            .build();

        let new_state = Update::increment(id).apply(state);

        assert_eq!(new_state.get(id), Some(&StateValue::Integer(6)));
    }

    #[test]
    fn test_update_toggle() {
        let id = AspectId(0);
        let state = StateBuilder::new()
            .set(id, StateValue::Bool(true))
            .build();

        let new_state = Update::toggle(id).apply(state);

        assert_eq!(new_state.get(id), Some(&StateValue::Bool(false)));
    }

    #[test]
    fn test_update_compose() {
        let id1 = AspectId(0);
        let id2 = AspectId(1);

        let state = StateBuilder::new()
            .set(id1, StateValue::Bool(false))
            .set(id2, StateValue::Integer(5))
            .build();

        let new_state = Update::compose(vec![
            Update::toggle(id1),
            Update::increment(id2),
        ])
        .apply(state);

        assert_eq!(new_state.get(id1), Some(&StateValue::Bool(true)));
        assert_eq!(new_state.get(id2), Some(&StateValue::Integer(6)));
    }

    #[test]
    fn test_update_conditional() {
        let id = AspectId(0);
        let state = StateBuilder::new()
            .set(id, StateValue::Integer(5))
            .build();

        let update = Update::conditional(
            move |s| s.get(id).and_then(|v| match v {
                StateValue::Integer(i) => Some(*i),
                _ => None,
            }).map_or(false, |i| i < 10),
            Update::increment(id),
        );

        let new_state = update.apply(state);

        assert_eq!(new_state.get(id), Some(&StateValue::Integer(6)));
    }

    #[test]
    fn test_update_builder() {
        let id1 = AspectId(0);
        let id2 = AspectId(1);

        let state = StateBuilder::new()
            .set(id1, StateValue::Bool(false))
            .set(id2, StateValue::Integer(5))
            .build();

        let update = UpdateBuilder::new()
            .toggle(id1)
            .increment(id2)
            .build();

        let new_state = update.apply(state);

        assert_eq!(new_state.get(id1), Some(&StateValue::Bool(true)));
        assert_eq!(new_state.get(id2), Some(&StateValue::Integer(6)));
    }
}