use std::collections::HashMap;
use std::fmt;

/// Unique identifier for a StateAspect
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AspectId(pub usize);

/// Represents the value of a StateAspect
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

/// Defines a single StateAspect - an orthogonal dimension of the state vector
#[derive(Debug, Clone)]
pub struct StateAspect {
    pub id: AspectId,
    pub name: String,
    pub default_value: StateValue,
    /// Optional minimum value constraint
    pub min_value: Option<StateValue>,
    /// Optional maximum value constraint
    pub max_value: Option<StateValue>,
}

impl StateAspect {
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
#[derive(Debug, Clone, PartialEq)]
pub struct State {
    /// Map from aspect ID to its current value
    values: HashMap<AspectId, StateValue>,
}

impl State {
    /// Create a new empty state
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    /// Create a state from a map of aspect IDs to values
    pub fn from_map(values: HashMap<AspectId, StateValue>) -> Self {
        Self { values }
    }

    /// Get the value of a specific aspect
    pub fn get(&self, aspect_id: AspectId) -> Option<&StateValue> {
        self.values.get(&aspect_id)
    }

    /// Set the value of a specific aspect, returning a new state
    pub fn set(&self, aspect_id: AspectId, value: StateValue) -> Self {
        let mut new_values = self.values.clone();
        new_values.insert(aspect_id, value);
        Self { values: new_values }
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
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for constructing State instances
pub struct StateBuilder {
    values: HashMap<AspectId, StateValue>,
}

impl StateBuilder {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn set(mut self, aspect_id: AspectId, value: StateValue) -> Self {
        self.values.insert(aspect_id, value);
        self
    }

    pub fn build(self) -> State {
        State::from_map(self.values)
    }
}

impl Default for StateBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_creation() {
        let mut state = State::new();
        let id = AspectId(0);
        state = state.set(id, StateValue::Bool(true));

        assert_eq!(state.get(id), Some(&StateValue::Bool(true)));
    }

    #[test]
    fn test_state_builder() {
        let id1 = AspectId(0);
        let id2 = AspectId(1);

        let state = StateBuilder::new()
            .set(id1, StateValue::Bool(true))
            .set(id2, StateValue::Integer(42))
            .build();

        assert_eq!(state.get(id1), Some(&StateValue::Bool(true)));
        assert_eq!(state.get(id2), Some(&StateValue::Integer(42)));
    }
}