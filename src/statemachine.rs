// Blueprint module - State machine definition layer
use crate::core::{AspectId, ClonableAny, EventId};
use crate::aspect::AspectBlueprint;
use crate::state::State;
use crate::zone::Zone;
use crate::transition::Transition;
use std::any::TypeId;
use std::collections::{HashMap, HashSet};

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
/// - Zones (behavior areas with lifecycle handlers)
/// - Transitions (event-driven state changes)
///
/// This blueprint can be compiled/validated and then instantiated into a runnable state machine.
#[derive(Debug)]
pub struct StateMachineBlueprint {
    /// Unique identifier for this blueprint
    pub id: String,

    /// All state aspects defined in this blueprint (type-erased)
    aspects: HashMap<AspectId, AspectDescriptor>,

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

    /// Add a state aspect to the blueprint using AspectBlueprint
    pub fn add_aspect(&mut self, blueprint: AspectBlueprint) -> &mut Self {
        let descriptor = AspectDescriptor::from_blueprint(&blueprint);
        self.aspects.insert(blueprint.id, descriptor);
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

    /// Get all aspect descriptors in this blueprint
    pub fn aspects(&self) -> impl Iterator<Item = &AspectDescriptor> {
        self.aspects.values()
    }

    /// Get an aspect descriptor by ID
    pub fn get_aspect(&self, id: AspectId) -> Option<&AspectDescriptor> {
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
        let mut builder = crate::state::StateBuilder::new();
        for descriptor in self.aspects.values() {
            // Use ClonableAny::clone_box() to support all types implementing ClonableAny
            builder = builder.set(descriptor.id, descriptor.default_value.clone_box());
        }
        builder.build()
    }
}

// Runtime module - State machine execution layer
use crate::zone::ZoneId;

/// Runtime state machine instance
///
/// StateMachineRuntime is the executable instance of a state machine blueprint.
/// It maintains the current state, tracks zone activations, and provides event dispatch.
pub struct StateMachineRuntime {
    /// Reference to the blueprint
    blueprint: StateMachineBlueprint,
    
    /// Current state
    state: State,
    
    /// Zone activation tracking (zone_id -> active)
    zone_activations: HashMap<ZoneId, bool>,
}

impl StateMachineRuntime {
    /// Create a new runtime instance from a blueprint
    pub fn new(blueprint: StateMachineBlueprint) -> Self {
        let state = blueprint.create_initial_state();
        let zone_activations = blueprint
            .zones()
            .iter()
            .map(|zone| (zone.id, false))
            .collect();
        
        Self {
            blueprint,
            state,
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

        // Find and apply matching transitions
        for transition in self.blueprint.transitions() {
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
        for zone in self.blueprint.zones() {
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
            .iter()
            .map(|zone| (zone.id, false))
            .collect();
        
        // Initialize zone activations
        self.update_zone_activations();
    }
}

// Tests for blueprint module
#[cfg(test)]
mod blueprint_tests {
    use super::*;
    use crate::active_in::ActiveInFactory;
    use crate::update::Update;
    use crate::zone::ZoneId;
    use crate::transition::TransitionId;

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

        let zone = Zone::new(ZoneId(0), "test_zone", ActiveInFactory::always());

        blueprint.add_zone(zone);

        assert_eq!(blueprint.zones().len(), 1);
    }

    #[test]
    fn test_blueprint_add_transition() {
        let mut blueprint = StateMachineBlueprint::new("test_machine");

        let transition = Transition::new(
            TransitionId(0),
            "test_transition",
            ActiveInFactory::always(),
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
}

// Tests for runtime module
#[cfg(test)]
mod runtime_tests {
    use super::*;
    use crate::prelude::*;
    use crate::active_in::ActiveInFactory;
    use crate::update::Update;

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

        let transition = Transition::new(
            TransitionId(0),
            "start",
            ActiveInFactory::aspect_string_eq(AspectId(0), "idle"),
            EventId::new("start"),
            Update::set_string(AspectId(0), "running"),
        );

        let mut blueprint = StateMachineBlueprint::new("test");
        blueprint.add_aspect(aspect);
        blueprint.add_transition(transition);

        let mut runtime = StateMachineRuntime::new(blueprint);

        assert!(runtime.dispatch_str("start"));
        assert_eq!(runtime.state().get_as::<String>(AspectId(0)), Some(&"running".to_string()));
    }

    #[test]
    fn test_runtime_zone_activation() {
        let mode_aspect = AspectBlueprint::new(AspectId(0), "mode", "idle".to_string());
        let battery_aspect = AspectBlueprint::new(AspectId(1), "battery", 100i64);

        let zone = Zone::new(ZoneId(0), "low_battery", ActiveInFactory::aspect_lt(AspectId(1), 20));

        let transition = Transition::new(
            TransitionId(0),
            "consume",
            ActiveInFactory::always(),
            EventId::new("consume"),
            Update::set_int(AspectId(1), 10),
        );

        let mut blueprint = StateMachineBlueprint::new("test");
        blueprint.add_aspect(mode_aspect);
        blueprint.add_aspect(battery_aspect);
        blueprint.add_zone(zone);
        blueprint.add_transition(transition);

        let mut runtime = StateMachineRuntime::new(blueprint);

        assert!(!runtime.is_zone_active(ZoneId(0)));

        // Dispatch event to lower battery
        runtime.dispatch_str("consume");

        assert!(runtime.is_zone_active(ZoneId(0)));
    }

    #[test]
    fn test_runtime_reset() {
        let aspect = AspectBlueprint::new(AspectId(0), "mode", "idle".to_string());

        let transition = Transition::new(
            TransitionId(0),
            "start",
            ActiveInFactory::always(),
            EventId::new("start"),
            Update::set_string(AspectId(0), "running"),
        );

        let zone = Zone::new(ZoneId(0), "running", ActiveInFactory::aspect_string_eq(AspectId(0), "running"));

        let mut blueprint = StateMachineBlueprint::new("test");
        blueprint.add_aspect(aspect);
        blueprint.add_transition(transition);
        blueprint.add_zone(zone);

        let mut runtime = StateMachineRuntime::new(blueprint);

        runtime.dispatch_str("start");
        assert_eq!(runtime.state().get_as::<String>(AspectId(0)), Some(&"running".to_string()));
        assert!(runtime.is_zone_active(ZoneId(0)));

        runtime.reset();
        assert_eq!(runtime.state().get_as::<String>(AspectId(0)), Some(&"idle".to_string()));
        assert!(!runtime.is_zone_active(ZoneId(0)));
    }
}