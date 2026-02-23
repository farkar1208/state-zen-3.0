// Blueprint module - State machine definition layer
use crate::core::{AspectId, ClonableAny, EventId};
use crate::aspect::AspectBlueprint;
use crate::state::State;
use crate::zone::{Zone, ZoneBlueprint, ZoneId};
use crate::transition::{Transition, TransitionBlueprint, TransitionId};
use std::any::TypeId;
use std::collections::{HashMap, HashSet};

/// Validation errors that can occur when building a state machine blueprint
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// Duplicate aspect ID
    DuplicateAspectId { id: AspectId },
    /// Duplicate zone ID
    DuplicateZoneId { id: ZoneId },
    /// Duplicate transition ID
    DuplicateTransitionId { id: TransitionId },
    /// Zone references non-existent aspect
    ZoneReferencesUnknownAspect { zone_id: ZoneId, aspect_id: AspectId },
    /// Transition references non-existent aspect
    TransitionReferencesUnknownAspect { transition_id: TransitionId, aspect_id: AspectId },
    /// Empty blueprint ID
    EmptyBlueprintId,
    /// No aspects defined
    NoAspects,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::DuplicateAspectId { id } => write!(f, "Duplicate aspect ID: {:?}", id),
            ValidationError::DuplicateZoneId { id } => write!(f, "Duplicate zone ID: {:?}", id),
            ValidationError::DuplicateTransitionId { id } => write!(f, "Duplicate transition ID: {:?}", id),
            ValidationError::ZoneReferencesUnknownAspect { zone_id, aspect_id } => {
                write!(f, "Zone {:?} references unknown aspect {:?}", zone_id, aspect_id)
            }
            ValidationError::TransitionReferencesUnknownAspect { transition_id, aspect_id } => {
                write!(f, "Transition {:?} references unknown aspect {:?}", transition_id, aspect_id)
            }
            ValidationError::EmptyBlueprintId => write!(f, "Blueprint ID cannot be empty"),
            ValidationError::NoAspects => write!(f, "Blueprint must have at least one aspect"),
        }
    }
}

impl std::error::Error for ValidationError {}

/// Type-erased aspect descriptor
#[derive(Debug)]
pub struct AspectDescriptor {
    pub id: AspectId,
    pub name: String,
    pub type_id: TypeId,
    pub default_value: Box<dyn ClonableAny>,
    pub has_min: bool,
    pub has_max: bool,
}

impl Clone for AspectDescriptor {
    fn clone(&self) -> Self {
        // Use ClonableAny::clone_box() to support all types implementing ClonableAny
        Self {
            id: self.id,
            name: self.name.clone(),
            type_id: self.type_id,
            default_value: self.default_value.clone_box(),
            has_min: self.has_min,
            has_max: self.has_max,
        }
    }
}

impl AspectDescriptor {
    pub fn from_blueprint(blueprint: &AspectBlueprint) -> Self {
        Self {
            id: blueprint.id,
            name: blueprint.name.clone(),
            type_id: blueprint.default_type_id,
            default_value: blueprint.default_value.clone_box(),
            has_min: blueprint.bounds.as_ref().map(|b| b.min_value.is_some()).unwrap_or(false),
            has_max: blueprint.bounds.as_ref().map(|b| b.max_value.is_some()).unwrap_or(false),
        }
    }
}

/// A blueprint for defining a state machine without runtime execution
///
/// The StateMachineBlueprint contains all the declarative definition of a state machine:
/// - State aspects (the dimensions of the state vector)
/// - Zone blueprints (behavior areas without side effect handlers)
/// - Transition blueprints (event-driven state changes without side effect handlers)
///
/// This blueprint can be compiled/validated and then instantiated into a runnable state machine.
#[derive(Debug)]
pub struct StateMachineBlueprint {
    /// Unique identifier for this blueprint
    pub id: String,

    /// All state aspects defined in this blueprint (type-erased)
    aspects: HashMap<AspectId, AspectDescriptor>,

    /// All zone blueprints defined in this blueprint (indexed by ZoneId for O(1) lookup)
    zones: HashMap<ZoneId, ZoneBlueprint>,

    /// All transition blueprints defined in this blueprint (indexed by TransitionId for O(1) lookup)
    transitions: HashMap<TransitionId, TransitionBlueprint>,

    /// All event types referenced in this blueprint
    events: HashSet<EventId>,
}

impl StateMachineBlueprint {
    /// Create a new empty blueprint
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            aspects: HashMap::new(),
            zones: HashMap::new(),
            transitions: HashMap::new(),
            events: HashSet::new(),
        }
    }

    /// Add a state aspect to the blueprint using AspectBlueprint
    pub fn add_aspect(&mut self, blueprint: AspectBlueprint) -> &mut Self {
        let descriptor = AspectDescriptor::from_blueprint(&blueprint);
        self.aspects.insert(blueprint.id, descriptor);
        self
    }

    /// Add a zone blueprint to the blueprint
    pub fn add_zone(&mut self, zone: ZoneBlueprint) -> &mut Self {
        self.zones.insert(zone.id, zone);
        self
    }

    /// Add a transition blueprint to the blueprint
    pub fn add_transition(&mut self, transition: TransitionBlueprint) -> &mut Self {
        self.events.insert(transition.event.clone());
        self.transitions.insert(transition.id, transition);
        self
    }

    /// Get all aspect descriptors in this blueprint
    pub fn aspects(&self) -> impl Iterator<Item = &AspectDescriptor> {
        self.aspects.values()
    }

    /// Get an aspect descriptor by ID
    pub fn get_aspect(&self, id: AspectId) -> Option<&AspectDescriptor> {
        self.aspects.get(&id)
    }

    /// Get all zone blueprints in this blueprint
    pub fn zones(&self) -> impl Iterator<Item = &ZoneBlueprint> {
        self.zones.values()
    }

    /// Get all transition blueprints in this blueprint
    pub fn transitions(&self) -> impl Iterator<Item = &TransitionBlueprint> {
        self.transitions.values()
    }

    /// Get all event types referenced in this blueprint
    pub fn events(&self) -> &HashSet<EventId> {
        &self.events
    }

    /// Create an initial state from the blueprint's aspect defaults
    pub fn create_initial_state(&self) -> State {
        let mut builder = crate::state::StateBuilder::new();
        for descriptor in self.aspects.values() {
            // Use ClonableAny::clone_box() to support all types implementing ClonableAny
            builder = builder.set(descriptor.id, descriptor.default_value.clone_box());
        }
        builder.build()
    }

    /// Validate the blueprint for consistency and correctness
    ///
    /// This method performs comprehensive validation checks:
    /// - Ensures blueprint ID is not empty
    /// - Ensures at least one aspect is defined
    /// - Checks for duplicate IDs (aspect, zone, transition)
    /// - Verifies all referenced aspects exist
    ///
    /// # Returns
    /// `Ok(())` if validation passes, `Err(ValidationError)` if any issues are found.
    pub fn validate(&self) -> Result<(), ValidationError> {
        // Check blueprint ID
        if self.id.trim().is_empty() {
            return Err(ValidationError::EmptyBlueprintId);
        }

        // Check at least one aspect
        if self.aspects.is_empty() {
            return Err(ValidationError::NoAspects);
        }

        // Check for duplicate aspect IDs (already handled by HashMap, but verify)
        // This is redundant but provides clear error messages if the implementation changes

        // Check zone references
        for zone in self.zones.values() {
            // Extract aspect IDs referenced in the zone's ActiveIn predicate
            let referenced_aspects = zone.active_in.referenced_aspects();
            for aspect_id in referenced_aspects {
                if !self.aspects.contains_key(&aspect_id) {
                    return Err(ValidationError::ZoneReferencesUnknownAspect {
                        zone_id: zone.id,
                        aspect_id,
                    });
                }
            }
        }

        // Check transition references
        for transition in self.transitions.values() {
            // Extract aspect IDs referenced in the transition's ActiveIn predicate
            let referenced_aspects = transition.active_in.referenced_aspects();
            for aspect_id in referenced_aspects {
                if !self.aspects.contains_key(&aspect_id) {
                    return Err(ValidationError::TransitionReferencesUnknownAspect {
                        transition_id: transition.id,
                        aspect_id,
                    });
                }
            }

            // Extract aspect IDs referenced in the transition's Update operation
            let updated_aspects = transition.update.updated_aspects();
            for aspect_id in updated_aspects {
                if !self.aspects.contains_key(&aspect_id) {
                    return Err(ValidationError::TransitionReferencesUnknownAspect {
                        transition_id: transition.id,
                        aspect_id,
                    });
                }
            }
        }

        Ok(())
    }

    /// Get a zone blueprint by ID
    pub fn get_zone(&self, id: ZoneId) -> Option<&ZoneBlueprint> {
        self.zones.get(&id)
    }

    /// Get a transition blueprint by ID
    pub fn get_transition(&self, id: TransitionId) -> Option<&TransitionBlueprint> {
        self.transitions.get(&id)
    }

    /// Check if an aspect exists
    pub fn has_aspect(&self, id: AspectId) -> bool {
        self.aspects.contains_key(&id)
    }

    /// Check if a zone exists
    pub fn has_zone(&self, id: ZoneId) -> bool {
        self.zones.contains_key(&id)
    }

    /// Check if a transition exists
    pub fn has_transition(&self, id: TransitionId) -> bool {
        self.transitions.contains_key(&id)
    }
}

// Runtime module - State machine execution layer

/// Runtime state machine instance
///
/// StateMachineRuntime is the executable instance of a state machine blueprint.
/// It maintains the current state, tracks zone activations, and provides event dispatch.
pub struct StateMachineRuntime {
    /// Reference to the blueprint
    blueprint: StateMachineBlueprint,

    /// Current state
    state: State,

    /// Runtime zone instances (compiled from blueprints, indexed by ZoneId for O(1) lookup)
    zones: HashMap<ZoneId, Zone>,

    /// Runtime transition instances (compiled from blueprints, indexed by TransitionId for O(1) lookup)
    transitions: HashMap<TransitionId, Transition>,

    /// Zone activation tracking (zone_id -> active)
    zone_activations: HashMap<ZoneId, bool>,
}

impl StateMachineRuntime {
    /// Create a new runtime instance from a blueprint
    ///
    /// # Panics
    /// Panics if the blueprint validation fails. Use `validate()` method beforehand
    /// to check for validation errors in a controlled manner.
    pub fn new(blueprint: StateMachineBlueprint) -> Self {
        // Validate blueprint before creating runtime
        if let Err(e) = blueprint.validate() {
            panic!("Blueprint validation failed: {}", e);
        }

        let state = blueprint.create_initial_state();

        // Compile zone blueprints to runtime zones (indexed by ZoneId)
        let zones: HashMap<ZoneId, Zone> = blueprint
            .zones()
            .map(|zone_blueprint| (zone_blueprint.id, Zone::from_blueprint(zone_blueprint.clone())))
            .collect();

        // Compile transition blueprints to runtime transitions (indexed by TransitionId)
        let transitions: HashMap<TransitionId, Transition> = blueprint
            .transitions()
            .map(|transition_blueprint| (transition_blueprint.id, Transition::from_blueprint(transition_blueprint.clone())))
            .collect();

        let zone_activations = blueprint
            .zones()
            .map(|zone| (zone.id, false))
            .collect();

        Self {
            blueprint,
            state,
            zones,
            transitions,
            zone_activations,
        }
    }
    
    /// Get the current state
    pub fn state(&self) -> &State {
        &self.state
    }
    
    /// Get the blueprint reference
    pub fn blueprint(&self) -> &StateMachineBlueprint {
        &self.blueprint
    }
    
    /// Dispatch an event to the state machine
    ///
    /// Returns true if a transition was triggered, false otherwise
    ///
    /// # Panics
    /// Panics if a state update results in a value outside the defined range constraints.
    pub fn dispatch(&mut self, event: &EventId) -> bool {
        let mut triggered = false;

        // Find and apply matching transitions (use runtime transitions)
        for transition in self.transitions.values() {
            if transition.event == *event && transition.is_active(&self.state) {
                // Execute transition side effect
                transition.trigger();

                // Apply state update
                transition.apply(&mut self.state);

                triggered = true;
                break; // Only trigger first matching transition
            }
        }

        // Update zone activations after state change
        if triggered {
            self.update_zone_activations();
        }

        triggered
    }
    
    /// Dispatch an event by string
    pub fn dispatch_str(&mut self, event: &str) -> bool {
        self.dispatch(&EventId::new(event))
    }
    
    /// Update zone activations and trigger enter/exit handlers
    fn update_zone_activations(&mut self) {
        for zone in self.zones.values() {
            let is_active = zone.is_active(&self.state);
            let was_active = *self.zone_activations.get(&zone.id).unwrap_or(&false);

            // Zone just became active
            if is_active && !was_active {
                zone.enter();
                self.zone_activations.insert(zone.id, true);
            }
            // Zone just became inactive
            else if !is_active && was_active {
                zone.exit();
                self.zone_activations.insert(zone.id, false);
            }
        }
    }

    /// Get currently active zone IDs
    pub fn active_zones(&self) -> Vec<ZoneId> {
        self.zone_activations
            .iter()
            .filter(|(_, active)| **active)
            .map(|(id, _)| *id)
            .collect()
    }
    
    /// Check if a specific zone is active
    pub fn is_zone_active(&self, zone_id: ZoneId) -> bool {
        *self.zone_activations.get(&zone_id).unwrap_or(&false)
    }
    
    /// Reset the state machine to initial state
    pub fn reset(&mut self) {
        self.state = self.blueprint.create_initial_state();
        self.zone_activations = self.blueprint
            .zones()
            .map(|zone| (zone.id, false))
            .collect();

        // Initialize zone activations
        self.update_zone_activations();
    }

    /// Add an on_enter handler to a zone by ID
    ///
    /// This allows attaching side effects to zones after runtime creation.
    pub fn with_zone_on_enter<F>(mut self, zone_id: ZoneId, handler: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        if let Some(zone) = self.zones.get_mut(&zone_id) {
            zone.on_enter = Some(Box::new(handler));
        }
        self
    }

    /// Add an on_exit handler to a zone by ID
    ///
    /// This allows attaching side effects to zones after runtime creation.
    pub fn with_zone_on_exit<F>(mut self, zone_id: ZoneId, handler: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        if let Some(zone) = self.zones.get_mut(&zone_id) {
            zone.on_exit = Some(Box::new(handler));
        }
        self
    }

    /// Add an on_tran handler to a transition by ID
    ///
    /// This allows attaching side effects to transitions after runtime creation.
    pub fn with_transition_on_tran<F>(mut self, transition_id: crate::transition::TransitionId, handler: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        if let Some(transition) = self.transitions.get_mut(&transition_id) {
            transition.on_tran = Some(Box::new(handler));
        }
        self
    }

    /// Replace the update operation for a transition by ID
    ///
    /// This allows customizing the update logic for transitions after runtime creation.
    pub fn with_transition_update(mut self, transition_id: crate::transition::TransitionId, update: crate::update::Update) -> Self {
        if let Some(transition) = self.transitions.get_mut(&transition_id) {
            transition.update = update;
        }
        self
    }
}

// Tests for blueprint module
#[cfg(test)]
mod blueprint_tests {
    use super::*;
    use crate::active_in::ActiveInBlueprint;
    use crate::update::UpdateBlueprint;
    use crate::zone::ZoneId;
    use crate::transition::TransitionId;

    #[test]
    fn test_blueprint_creation() {
        let blueprint = StateMachineBlueprint::new("test_machine");

        assert_eq!(blueprint.id, "test_machine");
        assert_eq!(blueprint.aspects().count(), 0);
        assert_eq!(blueprint.zones().count(), 0);
        assert_eq!(blueprint.transitions().count(), 0);
    }

    #[test]
    fn test_blueprint_add_aspect() {
        let mut blueprint = StateMachineBlueprint::new("test_machine");

        let aspect_blueprint = AspectBlueprint::new(AspectId(0), "counter", 0i32);

        blueprint.add_aspect(aspect_blueprint);

        assert_eq!(blueprint.aspects().count(), 1);
    }

    #[test]
    fn test_blueprint_add_aspect_with_bounds() {
        let mut blueprint = StateMachineBlueprint::new("test_machine");

        let aspect_blueprint = AspectBlueprint::new(AspectId(0), "counter", 50i32)
            .with_range(0, 100);

        blueprint.add_aspect(aspect_blueprint);

        let descriptor = blueprint.get_aspect(AspectId(0)).unwrap();
        assert!(descriptor.has_min);
        assert!(descriptor.has_max);
    }

    #[test]
    fn test_blueprint_add_zone() {
        let mut blueprint = StateMachineBlueprint::new("test_machine");

        let zone_blueprint = ZoneBlueprint::new(ZoneId(0), "test_zone", ActiveInBlueprint::always());

        blueprint.add_zone(zone_blueprint);

        assert_eq!(blueprint.zones().count(), 1);
    }

    #[test]
    fn test_blueprint_add_transition() {
        let mut blueprint = StateMachineBlueprint::new("test_machine");

        let transition_blueprint = TransitionBlueprint::new(
            TransitionId(0),
            "test_transition",
            ActiveInBlueprint::always(),
            EventId::new("start"),
            UpdateBlueprint::noop(),
        );

        blueprint.add_transition(transition_blueprint);

        assert_eq!(blueprint.transitions().count(), 1);
        assert!(blueprint.events().contains(&EventId::new("start")));
    }

    #[test]
    fn test_blueprint_initial_state() {
        let mut blueprint = StateMachineBlueprint::new("test_machine");

        let aspect1 = AspectBlueprint::new(AspectId(0), "mode", "idle".to_string());
        let aspect2 = AspectBlueprint::new(AspectId(1), "count", 0i64);

        blueprint.add_aspect(aspect1);
        blueprint.add_aspect(aspect2);

        let state = blueprint.create_initial_state();

        assert_eq!(state.get_as::<String>(AspectId(0)), Some(&"idle".to_string()));
        assert_eq!(state.get_as::<i64>(AspectId(1)), Some(&0i64));
    }

    #[test]
    fn test_blueprint_builder_multiple_types() {
        let aspect1 = AspectBlueprint::new(AspectId(0), "count", 0i32);
        let aspect2 = AspectBlueprint::new(AspectId(1), "temperature", 20.0f64)
            .with_range(0.0, 100.0);
        let aspect3 = AspectBlueprint::new(AspectId(2), "enabled", true);

        let mut blueprint = StateMachineBlueprint::new("test_machine");
        blueprint.add_aspect(aspect1);
        blueprint.add_aspect(aspect2);
        blueprint.add_aspect(aspect3);

        assert_eq!(blueprint.aspects().count(), 3);
    }

    #[test]
    fn test_aspect_descriptor_from_blueprint() {
        let blueprint = AspectBlueprint::new(AspectId(0), "counter", 42i32)
            .with_range(0, 100);

        let descriptor = AspectDescriptor::from_blueprint(&blueprint);

        assert_eq!(descriptor.id, AspectId(0));
        assert_eq!(descriptor.name, "counter");
        assert_eq!(descriptor.type_id, TypeId::of::<i32>());
        assert!(descriptor.has_min);
        assert!(descriptor.has_max);
    }

    #[test]
    fn test_validate_empty_blueprint_id() {
        let blueprint = StateMachineBlueprint::new("");

        let result = blueprint.validate();
        assert_eq!(result, Err(ValidationError::EmptyBlueprintId));
    }

    #[test]
    fn test_validate_no_aspects() {
        let blueprint = StateMachineBlueprint::new("test");

        let result = blueprint.validate();
        assert_eq!(result, Err(ValidationError::NoAspects));
    }

    #[test]
    fn test_validate_zone_references_unknown_aspect() {
        let mut blueprint = StateMachineBlueprint::new("test");
        blueprint.add_aspect(AspectBlueprint::new(AspectId(0), "mode", "idle".to_string()));

        // Zone references AspectId(99) which doesn't exist
        let zone = ZoneBlueprint::new(
            ZoneId(0),
            "test_zone",
            ActiveInBlueprint::aspect_bool(AspectId(99), true),
        );
        blueprint.add_zone(zone);

        let result = blueprint.validate();
        assert!(matches!(
            result,
            Err(ValidationError::ZoneReferencesUnknownAspect { .. })
        ));
    }

    #[test]
    fn test_validate_transition_references_unknown_aspect() {
        let mut blueprint = StateMachineBlueprint::new("test");
        blueprint.add_aspect(AspectBlueprint::new(AspectId(0), "mode", "idle".to_string()));

        // Transition references AspectId(99) which doesn't exist
        let transition = TransitionBlueprint::new(
            TransitionId(0),
            "test_transition",
            ActiveInBlueprint::aspect_bool(AspectId(99), true),
            EventId::new("start"),
            UpdateBlueprint::noop(),
        );
        blueprint.add_transition(transition);

        let result = blueprint.validate();
        assert!(matches!(
            result,
            Err(ValidationError::TransitionReferencesUnknownAspect { .. })
        ));
    }

    #[test]
    fn test_validate_valid_blueprint() {
        let mut blueprint = StateMachineBlueprint::new("test");
        blueprint.add_aspect(AspectBlueprint::new(AspectId(0), "mode", "idle".to_string()));
        blueprint.add_aspect(AspectBlueprint::new(AspectId(1), "battery", 100i64));

        let zone = ZoneBlueprint::new(
            ZoneId(0),
            "low_battery",
            ActiveInBlueprint::aspect_lt(AspectId(1), 20),
        );
        blueprint.add_zone(zone);

        let transition = TransitionBlueprint::new(
            TransitionId(0),
            "start",
            ActiveInBlueprint::aspect_string_eq(AspectId(0), "idle"),
            EventId::new("start"),
            UpdateBlueprint::set_string(AspectId(0), "running"),
        );
        blueprint.add_transition(transition);

        let result = blueprint.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_has_aspect() {
        let mut blueprint = StateMachineBlueprint::new("test");
        blueprint.add_aspect(AspectBlueprint::new(AspectId(0), "mode", "idle".to_string()));

        assert!(blueprint.has_aspect(AspectId(0)));
        assert!(!blueprint.has_aspect(AspectId(99)));
    }

    #[test]
    fn test_has_zone() {
        let mut blueprint = StateMachineBlueprint::new("test");
        blueprint.add_aspect(AspectBlueprint::new(AspectId(0), "mode", "idle".to_string()));
        blueprint.add_zone(ZoneBlueprint::new(
            ZoneId(0),
            "test_zone",
            ActiveInBlueprint::always(),
        ));

        assert!(blueprint.has_zone(ZoneId(0)));
        assert!(!blueprint.has_zone(ZoneId(99)));
    }

    #[test]
    fn test_has_transition() {
        let mut blueprint = StateMachineBlueprint::new("test");
        blueprint.add_aspect(AspectBlueprint::new(AspectId(0), "mode", "idle".to_string()));
        blueprint.add_transition(TransitionBlueprint::new(
            TransitionId(0),
            "test_transition",
            ActiveInBlueprint::always(),
            EventId::new("start"),
            UpdateBlueprint::noop(),
        ));

        assert!(blueprint.has_transition(TransitionId(0)));
        assert!(!blueprint.has_transition(TransitionId(99)));
    }

    #[test]
    fn test_get_zone() {
        let mut blueprint = StateMachineBlueprint::new("test");
        blueprint.add_aspect(AspectBlueprint::new(AspectId(0), "mode", "idle".to_string()));
        let zone = ZoneBlueprint::new(ZoneId(0), "test_zone", ActiveInBlueprint::always());
        blueprint.add_zone(zone.clone());

        let retrieved = blueprint.get_zone(ZoneId(0));
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, ZoneId(0));

        assert!(blueprint.get_zone(ZoneId(99)).is_none());
    }

    #[test]
    fn test_get_transition() {
        let mut blueprint = StateMachineBlueprint::new("test");
        blueprint.add_aspect(AspectBlueprint::new(AspectId(0), "mode", "idle".to_string()));
        let transition = TransitionBlueprint::new(
            TransitionId(0),
            "test_transition",
            ActiveInBlueprint::always(),
            EventId::new("start"),
            UpdateBlueprint::noop(),
        );
        blueprint.add_transition(transition.clone());

        let retrieved = blueprint.get_transition(TransitionId(0));
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, TransitionId(0));

        assert!(blueprint.get_transition(TransitionId(99)).is_none());
    }
}

// Tests for runtime module
#[cfg(test)]
mod runtime_tests {
    use super::*;
    use crate::prelude::*;

    #[test]
    fn test_runtime_creation() {
        let aspect = AspectBlueprint::new(AspectId(0), "mode", "idle".to_string());

        let mut blueprint = StateMachineBlueprint::new("test");
        blueprint.add_aspect(aspect);

        let runtime = StateMachineRuntime::new(blueprint);

        assert_eq!(runtime.state().get_as::<String>(AspectId(0)), Some(&"idle".to_string()));
    }

    #[test]
    fn test_runtime_dispatch() {
        let aspect = AspectBlueprint::new(AspectId(0), "mode", "idle".to_string());

        let transition_blueprint = TransitionBlueprint::new(
            TransitionId(0),
            "start",
            ActiveInBlueprint::aspect_string_eq(AspectId(0), "idle"),
            EventId::new("start"),
            UpdateBlueprint::set_string(AspectId(0), "running"),
        );

        let mut blueprint = StateMachineBlueprint::new("test");
        blueprint.add_aspect(aspect);
        blueprint.add_transition(transition_blueprint);

        let mut runtime = StateMachineRuntime::new(blueprint);

        assert!(runtime.dispatch_str("start"));
        assert_eq!(runtime.state().get_as::<String>(AspectId(0)), Some(&"running".to_string()));
    }

    #[test]
    fn test_runtime_zone_activation() {
        let mode_aspect = AspectBlueprint::new(AspectId(0), "mode", "idle".to_string());
        let battery_aspect = AspectBlueprint::new(AspectId(1), "battery", 100i64);

        let zone_blueprint = ZoneBlueprint::new(ZoneId(0), "low_battery", ActiveInBlueprint::aspect_lt(AspectId(1), 20));

        let transition_blueprint = TransitionBlueprint::new(
            TransitionId(0),
            "consume",
            ActiveInBlueprint::always(),
            EventId::new("consume"),
            UpdateBlueprint::set_int(AspectId(1), 10),
        );

        let mut blueprint = StateMachineBlueprint::new("test");
        blueprint.add_aspect(mode_aspect);
        blueprint.add_aspect(battery_aspect);
        blueprint.add_zone(zone_blueprint);
        blueprint.add_transition(transition_blueprint);

        let mut runtime = StateMachineRuntime::new(blueprint);

        assert!(!runtime.is_zone_active(ZoneId(0)));

        // Dispatch event to lower battery
        runtime.dispatch_str("consume");

        assert!(runtime.is_zone_active(ZoneId(0)));
    }

    #[test]
    fn test_runtime_reset() {
        let aspect = AspectBlueprint::new(AspectId(0), "mode", "idle".to_string());

        let transition_blueprint = TransitionBlueprint::new(
            TransitionId(0),
            "start",
            ActiveInBlueprint::always(),
            EventId::new("start"),
            UpdateBlueprint::set_string(AspectId(0), "running"),
        );

        let zone_blueprint = ZoneBlueprint::new(ZoneId(0), "running", ActiveInBlueprint::aspect_string_eq(AspectId(0), "running"));

        let mut blueprint = StateMachineBlueprint::new("test");
        blueprint.add_aspect(aspect);
        blueprint.add_transition(transition_blueprint);
        blueprint.add_zone(zone_blueprint);

        let mut runtime = StateMachineRuntime::new(blueprint);

        runtime.dispatch_str("start");
        assert_eq!(runtime.state().get_as::<String>(AspectId(0)), Some(&"running".to_string()));
        assert!(runtime.is_zone_active(ZoneId(0)));

        runtime.reset();
        assert_eq!(runtime.state().get_as::<String>(AspectId(0)), Some(&"idle".to_string()));
        assert!(!runtime.is_zone_active(ZoneId(0)));
    }

    #[test]
    #[should_panic(expected = "Blueprint validation failed")]
    fn test_runtime_creation_with_invalid_blueprint_panics() {
        let blueprint = StateMachineBlueprint::new("test"); // No aspects added
        StateMachineRuntime::new(blueprint); // Should panic
    }

    #[test]
    fn test_runtime_creation_with_valid_blueprint() {
        let mut blueprint = StateMachineBlueprint::new("test");
        blueprint.add_aspect(AspectBlueprint::new(AspectId(0), "mode", "idle".to_string()));

        // This should not panic
        let runtime = StateMachineRuntime::new(blueprint);
        assert_eq!(runtime.state().get_as::<String>(AspectId(0)), Some(&"idle".to_string()));
    }
}