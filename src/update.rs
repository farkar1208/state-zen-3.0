use crate::aspect::{AspectId, State};
use crate::active_in::ActiveInBlueprint;
use std::any::Any;
use std::sync::Arc;

// ============================================================================
// BLUEPRINT LAYER - AST for Update operations
// ============================================================================

/// AST node types for Update operations (blueprint layer)
#[derive(Debug, Clone)]
pub enum UpdateBlueprint {
    /// No operation
    Noop,

    /// Set an aspect to a specific value
    Set {
        aspect_id: AspectId,
        value: BlueprintValue,
    },

    /// Modify an aspect's value using a transformation function (type-erased)
    Modify {
        aspect_id: AspectId,
        type_id: std::any::TypeId,
        op: ModifyOp,
    },

    /// Compose multiple updates to apply sequentially
    Compose(Vec<UpdateBlueprint>),

    /// Conditional update based on state predicate
    Conditional {
        predicate: ActiveInBlueprint,
        then_update: Box<UpdateBlueprint>,
        else_update: Option<Box<UpdateBlueprint>>,
    },
}

/// Blueprint value (type-erased representation)
#[derive(Debug, Clone)]
pub enum BlueprintValue {
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Typed {
        type_id: std::any::TypeId,
        bytes: Vec<u8>,
    },
}

/// Modify operation types
#[derive(Debug, Clone)]
pub enum ModifyOp {
    /// Increment by 1
    Increment,
    /// Decrement by 1
    Decrement,
    /// Add a delta
    Add(i64),
    /// Toggle boolean
    Toggle,
    /// Custom modification (type-erased)
    Custom {
        op_type: String, // For identification
        bytes: Vec<u8>,
    },
}

impl UpdateBlueprint {
    /// Create a no-op update
    pub fn noop() -> Self {
        UpdateBlueprint::Noop
    }

    /// Set an aspect to a specific value (type-erased)
    pub fn set(aspect_id: AspectId, value: BlueprintValue) -> Self {
        UpdateBlueprint::Set { aspect_id, value }
    }

    /// Set a boolean aspect
    pub fn set_bool(aspect_id: AspectId, value: bool) -> Self {
        UpdateBlueprint::Set {
            aspect_id,
            value: BlueprintValue::Bool(value),
        }
    }

    /// Set an integer aspect
    pub fn set_int(aspect_id: AspectId, value: i64) -> Self {
        UpdateBlueprint::Set {
            aspect_id,
            value: BlueprintValue::Integer(value),
        }
    }

    /// Set a float aspect
    pub fn set_float(aspect_id: AspectId, value: f64) -> Self {
        UpdateBlueprint::Set {
            aspect_id,
            value: BlueprintValue::Float(value),
        }
    }

    /// Set a string aspect
    pub fn set_string(aspect_id: AspectId, value: impl Into<String>) -> Self {
        UpdateBlueprint::Set {
            aspect_id,
            value: BlueprintValue::String(value.into()),
        }
    }

    /// Increment an integer aspect
    pub fn increment(aspect_id: AspectId) -> Self {
        UpdateBlueprint::Modify {
            aspect_id,
            type_id: std::any::TypeId::of::<i64>(),
            op: ModifyOp::Increment,
        }
    }

    /// Decrement an integer aspect
    pub fn decrement(aspect_id: AspectId) -> Self {
        UpdateBlueprint::Modify {
            aspect_id,
            type_id: std::any::TypeId::of::<i64>(),
            op: ModifyOp::Decrement,
        }
    }

    /// Add a delta to an integer aspect
    pub fn add(aspect_id: AspectId, delta: i64) -> Self {
        UpdateBlueprint::Modify {
            aspect_id,
            type_id: std::any::TypeId::of::<i64>(),
            op: ModifyOp::Add(delta),
        }
    }

    /// Toggle a boolean aspect
    pub fn toggle(aspect_id: AspectId) -> Self {
        UpdateBlueprint::Modify {
            aspect_id,
            type_id: std::any::TypeId::of::<bool>(),
            op: ModifyOp::Toggle,
        }
    }

    /// Generic modify for any type
    pub fn modify_typed<T>(aspect_id: AspectId) -> Self
    where
        T: Any + Send + Sync + Clone + 'static,
    {
        UpdateBlueprint::Modify {
            aspect_id,
            type_id: std::any::TypeId::of::<T>(),
            op: ModifyOp::Custom {
                op_type: std::any::type_name::<T>().to_string(),
                bytes: Vec::new(), // Placeholder for serialized closure
            },
        }
    }

    /// Compose multiple updates to apply sequentially
    pub fn compose(updates: Vec<UpdateBlueprint>) -> Self {
        if updates.is_empty() {
            UpdateBlueprint::Noop
        } else if updates.len() == 1 {
            updates.into_iter().next().unwrap()
        } else {
            UpdateBlueprint::Compose(updates)
        }
    }

    /// Create a conditional update based on state predicate
    pub fn conditional(
        predicate: ActiveInBlueprint,
        then_update: UpdateBlueprint,
    ) -> Self {
        UpdateBlueprint::Conditional {
            predicate,
            then_update: Box::new(then_update),
            else_update: None,
        }
    }

    /// Create a conditional update with else branch
    pub fn conditional_else(
        predicate: ActiveInBlueprint,
        then_update: UpdateBlueprint,
        else_update: UpdateBlueprint,
    ) -> Self {
        UpdateBlueprint::Conditional {
            predicate,
            then_update: Box::new(then_update),
            else_update: Some(Box::new(else_update)),
        }
    }

    /// Count the total number of AST nodes in this update tree
    pub fn node_count(&self) -> usize {
        match self {
            UpdateBlueprint::Noop => 1,
            UpdateBlueprint::Set { .. } => 1,
            UpdateBlueprint::Modify { .. } => 1,
            UpdateBlueprint::Compose(updates) => {
                1 + updates.iter().map(|u| u.node_count()).sum::<usize>()
            }
            UpdateBlueprint::Conditional {
                predicate,
                then_update,
                else_update,
            } => {
                1 + predicate.node_count()
                    + then_update.node_count()
                    + else_update.as_ref().map(|u| u.node_count()).unwrap_or(0)
            }
        }
    }
}

// ============================================================================
// BUILDER FOR UPDATE BLUEPRINT
// ============================================================================

/// Builder for constructing UpdateBlueprint instances
pub struct UpdateBlueprintBuilder {
    operations: Vec<UpdateBlueprint>,
}

impl UpdateBlueprintBuilder {
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
        }
    }

    pub fn set(mut self, aspect_id: AspectId, value: BlueprintValue) -> Self {
        self.operations.push(UpdateBlueprint::set(aspect_id, value));
        self
    }

    pub fn set_bool(mut self, aspect_id: AspectId, value: bool) -> Self {
        self.operations.push(UpdateBlueprint::set_bool(aspect_id, value));
        self
    }

    pub fn set_int(mut self, aspect_id: AspectId, value: i64) -> Self {
        self.operations.push(UpdateBlueprint::set_int(aspect_id, value));
        self
    }

    pub fn set_float(mut self, aspect_id: AspectId, value: f64) -> Self {
        self.operations.push(UpdateBlueprint::set_float(aspect_id, value));
        self
    }

    pub fn set_string(mut self, aspect_id: AspectId, value: impl Into<String>) -> Self {
        self.operations.push(UpdateBlueprint::set_string(aspect_id, value));
        self
    }

    pub fn increment(mut self, aspect_id: AspectId) -> Self {
        self.operations.push(UpdateBlueprint::increment(aspect_id));
        self
    }

    pub fn decrement(mut self, aspect_id: AspectId) -> Self {
        self.operations.push(UpdateBlueprint::decrement(aspect_id));
        self
    }

    pub fn add(mut self, aspect_id: AspectId, delta: i64) -> Self {
        self.operations.push(UpdateBlueprint::add(aspect_id, delta));
        self
    }

    pub fn toggle(mut self, aspect_id: AspectId) -> Self {
        self.operations.push(UpdateBlueprint::toggle(aspect_id));
        self
    }

    pub fn build(self) -> UpdateBlueprint {
        UpdateBlueprint::compose(self.operations)
    }
}

impl Default for UpdateBlueprintBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// RUNTIME LAYER - Compiled updates with closure-based evaluation
// ============================================================================

/// Represents how state evolves in response to events
///
/// An Update is a pure function that receives current state and returns new state.
/// It must be side-effect free; all side effects should be in on_tran handlers.
pub struct Update {
    operation: Arc<UpdateOp>,
}

impl Clone for Update {
    fn clone(&self) -> Self {
        Update {
            operation: Arc::clone(&self.operation),
        }
    }
}

/// Internal representation of update operations (runtime layer)
enum UpdateOp {
    Noop,
    Set(AspectId, Box<dyn Any + Send + Sync>),
    Modify(AspectId, Arc<dyn Fn(Box<dyn Any + Send + Sync>) -> Box<dyn Any + Send + Sync> + Send + Sync>),
    Compose(Vec<Update>),
    Conditional {
        predicate: Arc<dyn Fn(&State) -> bool + Send + Sync>,
        then_update: Update,
        else_update: Option<Update>,
    },
}

impl Update {
    /// Create a no-op update (state remains unchanged)
    pub fn noop() -> Self {
        Self {
            operation: Arc::new(UpdateOp::Noop),
        }
    }

    /// Compile an UpdateBlueprint into a runtime Update
    pub fn from_blueprint(blueprint: UpdateBlueprint) -> Self {
        Self {
            operation: Arc::new(compile_update(blueprint)),
        }
    }

    /// Set an aspect to a specific value (type-erased)
    pub fn set(aspect_id: AspectId, value: Box<dyn Any + Send + Sync>) -> Self {
        Self {
            operation: Arc::new(UpdateOp::Set(aspect_id, value)),
        }
    }

    /// Set a typed value
    pub fn set_typed<T: Any + Send + Sync>(aspect_id: AspectId, value: T) -> Self {
        Self::set(aspect_id, Box::new(value))
    }

    /// Modify an aspect's value using a transformation function
    pub fn modify<F>(aspect_id: AspectId, f: F) -> Self
    where
        F: Fn(Box<dyn Any + Send + Sync>) -> Box<dyn Any + Send + Sync> + Send + Sync + 'static,
    {
        Self {
            operation: Arc::new(UpdateOp::Modify(aspect_id, Arc::new(f))),
        }
    }

    /// Set a boolean aspect
    pub fn set_bool(aspect_id: AspectId, value: bool) -> Self {
        Self::set_typed(aspect_id, value)
    }

    /// Set an integer aspect
    pub fn set_int(aspect_id: AspectId, value: i64) -> Self {
        Self::set_typed(aspect_id, value)
    }

    /// Set a float aspect
    pub fn set_float(aspect_id: AspectId, value: f64) -> Self {
        Self::set_typed(aspect_id, value)
    }

    /// Set a string aspect
    pub fn set_string(aspect_id: AspectId, value: impl Into<String>) -> Self {
        Self::set_typed(aspect_id, value.into())
    }

    /// Increment an integer aspect
    pub fn increment(aspect_id: AspectId) -> Self {
        Self::modify(aspect_id, |boxed| {
            if let Some(i) = boxed.downcast_ref::<i64>() {
                Box::new(*i + 1)
            } else {
                boxed
            }
        })
    }

    /// Decrement an integer aspect
    pub fn decrement(aspect_id: AspectId) -> Self {
        Self::modify(aspect_id, |boxed| {
            if let Some(i) = boxed.downcast_ref::<i64>() {
                Box::new(*i - 1)
            } else {
                boxed
            }
        })
    }

    /// Add a delta to an integer aspect
    pub fn add(aspect_id: AspectId, delta: i64) -> Self {
        Self::modify(aspect_id, move |boxed| {
            if let Some(i) = boxed.downcast_ref::<i64>() {
                Box::new(*i + delta)
            } else {
                boxed
            }
        })
    }

    /// Toggle a boolean aspect
    pub fn toggle(aspect_id: AspectId) -> Self {
        Self::modify(aspect_id, |boxed| {
            if let Some(b) = boxed.downcast_ref::<bool>() {
                Box::new(!*b)
            } else {
                boxed
            }
        })
    }

    /// Generic modify for any type
    pub fn modify_typed<T, F>(aspect_id: AspectId, f: F) -> Self
    where
        T: Any + Send + Sync + Clone,
        F: Fn(T) -> T + Send + Sync + 'static,
    {
        Self::modify(aspect_id, move |boxed| {
            if let Some(value) = boxed.downcast_ref::<T>() {
                Box::new(f(value.clone()))
            } else {
                boxed
            }
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
                operation: Arc::new(UpdateOp::Compose(updates)),
            }
        }
    }

    /// Create a conditional update based on state
    pub fn conditional<F>(predicate: F, then_update: Update) -> Self
    where
        F: Fn(&State) -> bool + Send + Sync + 'static,
    {
        Self {
            operation: Arc::new(UpdateOp::Conditional {
                predicate: Arc::new(predicate),
                then_update,
                else_update: None,
            }),
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
            operation: Arc::new(UpdateOp::Conditional {
                predicate: Arc::new(predicate),
                then_update,
                else_update: Some(else_update),
            }),
        }
    }

    /// Apply this update to a state, returning a new state
    pub fn apply(&self, state: State) -> State {
        match &*self.operation {
            UpdateOp::Noop => state,
            UpdateOp::Set(aspect_id, value) => {
                // Clone common types
                if let Some(b) = value.downcast_ref::<bool>() {
                    state.set_typed(*aspect_id, *b)
                } else if let Some(i) = value.downcast_ref::<i64>() {
                    state.set_typed(*aspect_id, *i)
                } else if let Some(f) = value.downcast_ref::<f64>() {
                    state.set_typed(*aspect_id, *f)
                } else if let Some(s) = value.downcast_ref::<String>() {
                    state.set_typed(*aspect_id, s.clone())
                } else if let Some(i) = value.downcast_ref::<i32>() {
                    state.set_typed(*aspect_id, *i)
                } else {
                    state // Can't clone other types
                }
            }
            UpdateOp::Modify(aspect_id, f) => {
                // Clone state first to avoid borrow issues
                let state_cloned = state.clone();
                if let Some(v) = state_cloned.get(*aspect_id) {
                    // Create a boxed clone of the Any reference
                    let boxed_clone: Box<dyn Any + Send + Sync> = if let Some(b) = v.downcast_ref::<bool>() {
                        Box::new(*b)
                    } else if let Some(i) = v.downcast_ref::<i64>() {
                        Box::new(*i)
                    } else if let Some(f) = v.downcast_ref::<f64>() {
                        Box::new(*f)
                    } else if let Some(s) = v.downcast_ref::<String>() {
                        Box::new(s.clone())
                    } else if let Some(i) = v.downcast_ref::<i32>() {
                        Box::new(*i)
                    } else {
                        // For other types, we can't clone, so return original
                        return state_cloned;
                    };
                    state_cloned.set(*aspect_id, f(boxed_clone))
                } else {
                    state_cloned
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

/// Compile an UpdateBlueprint AST into a runtime UpdateOp
fn compile_update(blueprint: UpdateBlueprint) -> UpdateOp {
    match blueprint {
        UpdateBlueprint::Noop => UpdateOp::Noop,
        UpdateBlueprint::Set { aspect_id, value } => {
            let boxed: Box<dyn Any + Send + Sync> = match value {
                BlueprintValue::Bool(b) => Box::new(b),
                BlueprintValue::Integer(i) => Box::new(i),
                BlueprintValue::Float(f) => Box::new(f),
                BlueprintValue::String(s) => Box::new(s),
                BlueprintValue::Typed { type_id, bytes } => {
                    // For simplicity, only support i32 in typed values
                    if type_id == std::any::TypeId::of::<i32>() {
                        if let Some(&v) = deserialize_bytes::<i32>(&bytes) {
                            Box::new(v)
                        } else {
                            Box::new(())
                        }
                    } else {
                        Box::new(())
                    }
                }
            };
            UpdateOp::Set(aspect_id, boxed)
        }
        UpdateBlueprint::Modify { aspect_id, type_id: _, op } => {
            let f: Arc<dyn Fn(Box<dyn Any + Send + Sync>) -> Box<dyn Any + Send + Sync> + Send + Sync> =
                match op {
                    ModifyOp::Increment => Arc::new(|boxed| {
                        if let Some(i) = boxed.downcast_ref::<i64>() {
                            Box::new(*i + 1)
                        } else {
                            boxed
                        }
                    }),
                    ModifyOp::Decrement => Arc::new(|boxed| {
                        if let Some(i) = boxed.downcast_ref::<i64>() {
                            Box::new(*i - 1)
                        } else {
                            boxed
                        }
                    }),
                    ModifyOp::Add(delta) => Arc::new(move |boxed| {
                        if let Some(i) = boxed.downcast_ref::<i64>() {
                            Box::new(*i + delta)
                        } else {
                            boxed
                        }
                    }),
                    ModifyOp::Toggle => Arc::new(|boxed| {
                        if let Some(b) = boxed.downcast_ref::<bool>() {
                            Box::new(!*b)
                        } else {
                            boxed
                        }
                    }),
                    ModifyOp::Custom { .. } => {
                        // Placeholder for custom modifications
                        Arc::new(|boxed| boxed)
                    }
                };
            UpdateOp::Modify(aspect_id, f)
        }
        UpdateBlueprint::Compose(updates) => {
            let compiled: Vec<Update> = updates
                .into_iter()
                .map(|u| Update::from_blueprint(u))
                .collect();
            UpdateOp::Compose(compiled)
        }
        UpdateBlueprint::Conditional {
            predicate,
            then_update,
            else_update,
        } => {
            let predicate_fn: Arc<dyn Fn(&State) -> bool + Send + Sync> =
                Arc::new(move |state| crate::active_in::evaluate_blueprint(&predicate, state));
            UpdateOp::Conditional {
                predicate: predicate_fn,
                then_update: Update::from_blueprint(*then_update),
                else_update: else_update.map(|u| Update::from_blueprint(*u)),
            }
        }
    }
}

/// Deserialize bytes back to a value
fn deserialize_bytes<T>(bytes: &[u8]) -> Option<&T> {
    if bytes.len() != std::mem::size_of::<T>() {
        return None;
    }
    unsafe {
        Some(&*(bytes.as_ptr() as *const T))
    }
}

// ============================================================================
// BUILDER FOR RUNTIME UPDATE
// ============================================================================

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

    pub fn set(mut self, aspect_id: AspectId, value: Box<dyn Any + Send + Sync>) -> Self {
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

    pub fn set_typed<T: Any + Send + Sync>(mut self, aspect_id: AspectId, value: T) -> Self {
        self.operations.push(Update::set_typed(aspect_id, value));
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
        F: Fn(Box<dyn Any + Send + Sync>) -> Box<dyn Any + Send + Sync> + Send + Sync + 'static,
    {
        self.operations.push(Update::modify(aspect_id, f));
        self
    }

    pub fn modify_typed<T, F>(mut self, aspect_id: AspectId, f: F) -> Self
    where
        T: Any + Send + Sync + Clone,
        F: Fn(T) -> T + Send + Sync + 'static,
    {
        self.operations.push(Update::modify_typed(aspect_id, f));
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

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aspect::StateBuilder;

    #[test]
    fn test_blueprint_noop() {
        let blueprint = UpdateBlueprint::noop();
        assert_eq!(blueprint.node_count(), 1);
    }

    #[test]
    fn test_blueprint_set() {
        let id = AspectId(0);
        let blueprint = UpdateBlueprint::set_bool(id, true);
        assert_eq!(blueprint.node_count(), 1);
    }

    #[test]
    fn test_blueprint_increment() {
        let id = AspectId(0);
        let blueprint = UpdateBlueprint::increment(id);
        assert_eq!(blueprint.node_count(), 1);
    }

    #[test]
    fn test_blueprint_compose() {
        let id1 = AspectId(0);
        let id2 = AspectId(1);

        let blueprint = UpdateBlueprint::compose(vec![
            UpdateBlueprint::set_bool(id1, true),
            UpdateBlueprint::increment(id2),
        ]);

        assert_eq!(blueprint.node_count(), 3); // Compose + 2 leaf nodes
    }

    #[test]
    fn test_blueprint_conditional() {
        let id = AspectId(0);
        let predicate = crate::active_in::ActiveInBlueprint::aspect_bool(id, true);
        let blueprint = UpdateBlueprint::conditional(predicate, UpdateBlueprint::increment(id));

        assert_eq!(blueprint.node_count(), 3); // Conditional + predicate + then_update
    }

    #[test]
    fn test_blueprint_builder() {
        let id1 = AspectId(0);
        let id2 = AspectId(1);

        let blueprint = UpdateBlueprintBuilder::new()
            .set_bool(id1, true)
            .increment(id2)
            .build();

        assert_eq!(blueprint.node_count(), 3);
    }

    #[test]
    fn test_runtime_from_blueprint() {
        let id = AspectId(0);
        let state = StateBuilder::new()
            .set_bool(id, true)
            .build();

        let blueprint = UpdateBlueprint::set_bool(id, false);
        let runtime = Update::from_blueprint(blueprint);

        let new_state = runtime.apply(state);
        assert_eq!(new_state.get_as::<bool>(id), Some(&false));
    }

    #[test]
    fn test_runtime_noop() {
        let id = AspectId(0);
        let state = StateBuilder::new()
            .set_bool(id, true)
            .build();

        let new_state = Update::noop().apply(state.clone());

        assert_eq!(new_state, state);
    }

    #[test]
    fn test_runtime_set() {
        let id = AspectId(0);
        let state = StateBuilder::new()
            .set_bool(id, true)
            .build();

        let new_state = Update::set_bool(id, false).apply(state);

        assert_eq!(new_state.get_as::<bool>(id), Some(&false));
    }

    #[test]
    fn test_runtime_increment() {
        let id = AspectId(0);
        let state = StateBuilder::new()
            .set_int(id, 5)
            .build();

        let new_state = Update::increment(id).apply(state);

        assert_eq!(new_state.get_as::<i64>(id), Some(&6));
    }

    #[test]
    fn test_runtime_toggle() {
        let id = AspectId(0);
        let state = StateBuilder::new()
            .set_bool(id, true)
            .build();

        let new_state = Update::toggle(id).apply(state);

        assert_eq!(new_state.get_as::<bool>(id), Some(&false));
    }

    #[test]
    fn test_runtime_compose() {
        let id1 = AspectId(0);
        let id2 = AspectId(1);

        let state = StateBuilder::new()
            .set_bool(id1, false)
            .set_int(id2, 5)
            .build();

        let new_state = Update::compose(vec![
            Update::toggle(id1),
            Update::increment(id2),
        ])
        .apply(state);

        assert_eq!(new_state.get_as::<bool>(id1), Some(&true));
        assert_eq!(new_state.get_as::<i64>(id2), Some(&6));
    }

    #[test]
    fn test_runtime_conditional() {
        let id = AspectId(0);
        let state = StateBuilder::new()
            .set_int(id, 5)
            .build();

        let update = Update::conditional(
            move |s| s.get_as::<i64>(id).map_or(false, |&i| i < 10),
            Update::increment(id),
        );

        let new_state = update.apply(state);

        assert_eq!(new_state.get_as::<i64>(id), Some(&6));
    }

    #[test]
    fn test_runtime_builder() {
        let id1 = AspectId(0);
        let id2 = AspectId(1);

        let state = StateBuilder::new()
            .set_bool(id1, false)
            .set_int(id2, 5)
            .build();

        let update = UpdateBuilder::new()
            .toggle(id1)
            .increment(id2)
            .build();

        let new_state = update.apply(state);

        assert_eq!(new_state.get_as::<bool>(id1), Some(&true));
        assert_eq!(new_state.get_as::<i64>(id2), Some(&6));
    }

    #[test]
    fn test_runtime_typed() {
        let id = AspectId(0);
        let state = StateBuilder::new()
            .set_typed(id, 42i32)
            .build();

        let update = Update::modify_typed(id, |v: i32| v + 10);
        let new_state = update.apply(state);

        assert_eq!(new_state.get_as::<i32>(id), Some(&52));
    }
}