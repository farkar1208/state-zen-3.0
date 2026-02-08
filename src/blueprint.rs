use crate::aspect::{AspectId, State, StateAspect};
use crate::zone::Zone;
use crate::transition::{EventId, Transition};
use std::collections::{HashMap, HashSet};

/// A blueprint for defining a state machine without runtime execution
///
/// The StateMachineBlueprint contains all the declarative definition of a state machine:
/// - State aspects (the dimensions of the state vector)
/// - Zones (behavior areas with lifecycle handlers)
/// - Transitions (event-driven state changes)
///
/// This blueprint can be compiled/validated and then instantiated into a runnable state machine.
#[derive(Debug)]
pub struct StateMachineBlueprint {
    /// Unique identifier for this blueprint
    pub id: String,

    /// All state aspects defined in this blueprint
    aspects: HashMap<AspectId, StateAspect>,

    /// All zones defined in this blueprint
    zones: Vec<Zone>,

    /// All transitions defined in this blueprint
    transitions: Vec<Transition>,

    /// All event types referenced in this blueprint
    events: HashSet<EventId>,
}

impl StateMachineBlueprint {
    /// Create a new empty blueprint
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            aspects: HashMap::new(),
            zones: Vec::new(),
            transitions: Vec::new(),
            events: HashSet::new(),
        }
    }

    /// Add a state aspect to the blueprint
    pub fn add_aspect(&mut self, aspect: StateAspect) -> &mut Self {
        self.aspects.insert(aspect.id, aspect);
        self
    }

    /// Add a zone to the blueprint
    pub fn add_zone(&mut self, zone: Zone) -> &mut Self {
        self.zones.push(zone);
        self
    }

    /// Add a transition to the blueprint
    pub fn add_transition(&mut self, transition: Transition) -> &mut Self {
        self.events.insert(transition.event.clone());
        self.transitions.push(transition);
        self
    }

    /// Get all aspects in this blueprint
    pub fn aspects(&self) -> impl Iterator<Item = &StateAspect> {
        self.aspects.values()
    }

    /// Get an aspect by ID
    pub fn get_aspect(&self, id: AspectId) -> Option<&StateAspect> {
        self.aspects.get(&id)
    }

    /// Get all zones in this blueprint
    pub fn zones(&self) -> &[Zone] {
        &self.zones
    }

    /// Get all transitions in this blueprint
    pub fn transitions(&self) -> &[Transition] {
        &self.transitions
    }

    /// Get all event types referenced in this blueprint
    pub fn events(&self) -> &HashSet<EventId> {
        &self.events
    }

    /// Create an initial state from the blueprint's aspect defaults
    pub fn create_initial_state(&self) -> State {
        let mut builder = crate::aspect::StateBuilder::new();
        for aspect in self.aspects.values() {
            builder = builder.set(aspect.id, aspect.default_value.clone());
        }
        builder.build()
    }

    /// Validate the blueprint
    pub fn validate(&self) -> Result<ValidationResult, Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Check for duplicate zone IDs
        let mut zone_ids = HashSet::new();
        for zone in &self.zones {
            if !zone_ids.insert(&zone.id) {
                errors.push(ValidationError::DuplicateZoneId(zone.id.clone()));
            }
        }

        // Check for duplicate transition IDs
        let mut transition_ids = HashSet::new();
        for transition in &self.transitions {
            if !transition_ids.insert(&transition.id) {
                errors.push(ValidationError::DuplicateTransitionId(transition.id.clone()));
            }
        }

        // Check that all aspect IDs referenced in activeIn and update exist
        for transition in &self.transitions {
            self._validate_aspect_refs_in_active_in(&transition.active_in, &mut errors);
            self._validate_aspect_refs_in_update(&transition.update, &mut errors);
        }

        for zone in &self.zones {
            self._validate_aspect_refs_in_active_in(&zone.active_in, &mut errors);
        }

        if errors.is_empty() {
            Ok(ValidationResult::Valid)
        } else {
            Err(errors)
        }
    }

    /// Helper to validate aspect references in ActiveIn predicates
    fn _validate_aspect_refs_in_active_in(
        &self,
        _active_in: &crate::active_in::ActiveIn,
        _errors: &mut Vec<ValidationError>,
    ) {
        // Note: Since ActiveIn is a closure, we cannot inspect its internals at compile time
        // This validation would need to be done through a different mechanism
        // (e.g., custom DSL, reflection, or explicit aspect registration)
    }

    /// Helper to validate aspect references in Update operations
    fn _validate_aspect_refs_in_update(
        &self,
        _update: &crate::update::Update,
        _errors: &mut Vec<ValidationError>,
    ) {
        // Note: Same limitation as ActiveIn - Update is a closure-based operation
        // Full validation would require a different approach
    }

    /// Get the number of aspects, zones, and transitions
    pub fn stats(&self) -> BlueprintStats {
        BlueprintStats {
            aspect_count: self.aspects.len(),
            zone_count: self.zones.len(),
            transition_count: self.transitions.len(),
            event_count: self.events.len(),
        }
    }
}

/// Validation result for blueprint validation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationResult {
    Valid,
}

/// Validation error types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    DuplicateZoneId(String),
    DuplicateTransitionId(String),
    UndefinedAspect(AspectId),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::DuplicateZoneId(id) => {
                write!(f, "Duplicate zone ID: {}", id)
            }
            ValidationError::DuplicateTransitionId(id) => {
                write!(f, "Duplicate transition ID: {}", id)
            }
            ValidationError::UndefinedAspect(id) => {
                write!(f, "Undefined aspect referenced: {:?}", id)
            }
        }
    }
}

/// Statistics about a blueprint
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlueprintStats {
    pub aspect_count: usize,
    pub zone_count: usize,
    pub transition_count: usize,
    pub event_count: usize,
}

/// Builder for constructing StateMachineBlueprint instances
pub struct BlueprintBuilder {
    id: Option<String>,
    aspects: Vec<StateAspect>,
    zones: Vec<Zone>,
    transitions: Vec<Transition>,
}

impl BlueprintBuilder {
    pub fn new() -> Self {
        Self {
            id: None,
            aspects: Vec::new(),
            zones: Vec::new(),
            transitions: Vec::new(),
        }
    }

    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn aspect(mut self, aspect: StateAspect) -> Self {
        self.aspects.push(aspect);
        self
    }

    pub fn zone(mut self, zone: Zone) -> Self {
        self.zones.push(zone);
        self
    }

    pub fn transition(mut self, transition: Transition) -> Self {
        self.transitions.push(transition);
        self
    }

    pub fn build(self) -> Result<StateMachineBlueprint, String> {
        let id = self.id.ok_or("Blueprint id is required")?;

        let mut blueprint = StateMachineBlueprint::new(id);
        for aspect in self.aspects {
            blueprint.add_aspect(aspect);
        }
        for zone in self.zones {
            blueprint.add_zone(zone);
        }
        for transition in self.transitions {
            blueprint.add_transition(transition);
        }

        Ok(blueprint)
    }
}

impl Default for BlueprintBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::active_in::ActiveIn;
    use crate::aspect::StateValue;
    use crate::update::Update;

    #[test]
    fn test_blueprint_creation() {
        let blueprint = StateMachineBlueprint::new("test_machine");

        assert_eq!(blueprint.id, "test_machine");
        assert_eq!(blueprint.aspects().count(), 0);
        assert_eq!(blueprint.zones().len(), 0);
        assert_eq!(blueprint.transitions().len(), 0);
    }

    #[test]
    fn test_blueprint_add_aspect() {
        let mut blueprint = StateMachineBlueprint::new("test_machine");

        let aspect = StateAspect::new(
            AspectId(0),
            "mode",
            StateValue::String("idle".to_string()),
        );

        blueprint.add_aspect(aspect);

        assert_eq!(blueprint.aspects().count(), 1);
    }

    #[test]
    fn test_blueprint_add_zone() {
        let mut blueprint = StateMachineBlueprint::new("test_machine");

        let zone = Zone::new("test_zone", ActiveIn::always());

        blueprint.add_zone(zone);

        assert_eq!(blueprint.zones().len(), 1);
    }

    #[test]
    fn test_blueprint_add_transition() {
        let mut blueprint = StateMachineBlueprint::new("test_machine");

        let transition = Transition::new(
            "test_transition",
            ActiveIn::always(),
            EventId::new("start"),
            Update::noop(),
        );

        blueprint.add_transition(transition);

        assert_eq!(blueprint.transitions().len(), 1);
        assert!(blueprint.events().contains(&EventId::new("start")));
    }

    #[test]
    fn test_blueprint_initial_state() {
        let mut blueprint = StateMachineBlueprint::new("test_machine");

        let aspect1 = StateAspect::new(AspectId(0), "mode", StateValue::String("idle".to_string()));
        let aspect2 = StateAspect::new(AspectId(1), "count", StateValue::Integer(0));

        blueprint.add_aspect(aspect1);
        blueprint.add_aspect(aspect2);

        let state = blueprint.create_initial_state();

        assert_eq!(state.get(AspectId(0)), Some(&StateValue::String("idle".to_string())));
        assert_eq!(state.get(AspectId(1)), Some(&StateValue::Integer(0)));
    }

    #[test]
    fn test_blueprint_stats() {
        let mut blueprint = StateMachineBlueprint::new("test_machine");

        let aspect = StateAspect::new(AspectId(0), "mode", StateValue::String("idle".to_string()));
        blueprint.add_aspect(aspect);

        let zone = Zone::new("test_zone", ActiveIn::always());
        blueprint.add_zone(zone);

        let transition = Transition::new(
            "test_transition",
            ActiveIn::always(),
            EventId::new("start"),
            Update::noop(),
        );
        blueprint.add_transition(transition);

        let stats = blueprint.stats();

        assert_eq!(stats.aspect_count, 1);
        assert_eq!(stats.zone_count, 1);
        assert_eq!(stats.transition_count, 1);
        assert_eq!(stats.event_count, 1);
    }

    #[test]
    fn test_blueprint_builder() {
        let aspect = StateAspect::new(AspectId(0), "mode", StateValue::String("idle".to_string()));
        let zone = Zone::new("test_zone", ActiveIn::always());
        let transition = Transition::new(
            "test_transition",
            ActiveIn::always(),
            EventId::new("start"),
            Update::noop(),
        );

        let blueprint = BlueprintBuilder::new()
            .id("test_machine")
            .aspect(aspect)
            .zone(zone)
            .transition(transition)
            .build()
            .unwrap();

        assert_eq!(blueprint.id, "test_machine");
        assert_eq!(blueprint.aspects().count(), 1);
        assert_eq!(blueprint.zones().len(), 1);
        assert_eq!(blueprint.transitions().len(), 1);
    }
}