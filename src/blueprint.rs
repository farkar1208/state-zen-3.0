use crate::aspect::{AspectId, State, StateAspect, StateAspectLegacy};
use crate::zone::Zone;
use crate::transition::{EventId, Transition};
use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};

/// Type-erased aspect descriptor
#[derive(Debug)]
pub struct AspectDescriptor {
    pub id: AspectId,
    pub name: String,
    pub type_id: TypeId,
    pub default_value: Box<dyn Any + Send + Sync>,
    pub has_min: bool,
    pub has_max: bool,
}

impl Clone for AspectDescriptor {
    fn clone(&self) -> Self {
        // Clone common types
        let cloned_value: Box<dyn Any + Send + Sync> = if let Some(b) = self.default_value.downcast_ref::<bool>() {
            Box::new(*b)
        } else if let Some(i) = self.default_value.downcast_ref::<i64>() {
            Box::new(*i)
        } else if let Some(f) = self.default_value.downcast_ref::<f64>() {
            Box::new(*f)
        } else if let Some(s) = self.default_value.downcast_ref::<String>() {
            Box::new(s.clone())
        } else if let Some(i) = self.default_value.downcast_ref::<i32>() {
            Box::new(*i)
        } else {
            // For other types, can't clone, use a placeholder
            Box::new(())
        };

        Self {
            id: self.id,
            name: self.name.clone(),
            type_id: self.type_id,
            default_value: cloned_value,
            has_min: self.has_min,
            has_max: self.has_max,
        }
    }
}

impl AspectDescriptor {
    pub fn new<T>(aspect: &StateAspect<T>) -> Self
    where
        T: Any + Send + Sync + Clone,
    {
        Self {
            id: aspect.id,
            name: aspect.name.clone(),
            type_id: TypeId::of::<T>(),
            default_value: Box::new(aspect.default_value.clone()),
            has_min: aspect.bounds.min.is_some(),
            has_max: aspect.bounds.max.is_some(),
        }
    }

    pub fn from_legacy(aspect: &StateAspectLegacy) -> Self {
        Self {
            id: aspect.id,
            name: aspect.name.clone(),
            type_id: TypeId::of::<crate::aspect::StateValue>(),
            default_value: Box::new(aspect.default_value.clone()),
            has_min: aspect.min_value.is_some(),
            has_max: aspect.max_value.is_some(),
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

    /// Add a state aspect to the blueprint (generic version)
    pub fn add_aspect<T>(&mut self, aspect: StateAspect<T>) -> &mut Self
    where
        T: Any + Send + Sync + Clone,
    {
        let descriptor = AspectDescriptor::new(&aspect);
        self.aspects.insert(aspect.id, descriptor);
        self
    }

    /// Add a legacy state aspect to the blueprint
    pub fn add_aspect_legacy(&mut self, aspect: StateAspectLegacy) -> &mut Self {
        let descriptor = AspectDescriptor::from_legacy(&aspect);
        self.aspects.insert(aspect.id, descriptor);
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
        let mut builder = crate::aspect::StateBuilder::new();
        for descriptor in self.aspects.values() {
            // Clone common types
            let cloned_value: Box<dyn Any + Send + Sync> = if let Some(b) = descriptor.default_value.downcast_ref::<bool>() {
                Box::new(*b)
            } else if let Some(i) = descriptor.default_value.downcast_ref::<i64>() {
                Box::new(*i)
            } else if let Some(f) = descriptor.default_value.downcast_ref::<f64>() {
                Box::new(*f)
            } else if let Some(s) = descriptor.default_value.downcast_ref::<String>() {
                Box::new(s.clone())
            } else if let Some(i) = descriptor.default_value.downcast_ref::<i32>() {
                Box::new(*i)
            } else {
                continue; // Skip types we can't clone
            };
            builder = builder.set(descriptor.id, cloned_value);
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
    aspects: Vec<Box<dyn Any>>,
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

    /// Add a generic aspect
    pub fn aspect<T>(mut self, aspect: StateAspect<T>) -> Self
    where
        T: Any + Send + Sync + Clone,
    {
        self.aspects.push(Box::new(aspect));
        self
    }

    /// Add a legacy aspect
    pub fn aspect_legacy(mut self, aspect: StateAspectLegacy) -> Self {
        self.aspects.push(Box::new(aspect));
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

        for aspect_box in self.aspects {
            // Try to downcast to StateAspect<T> using a helper function
            let result = try_add_aspect(&mut blueprint, aspect_box);
            if let Err(e) = result {
                return Err(e);
            }
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

/// Helper function to try adding an aspect to blueprint
fn try_add_aspect(blueprint: &mut StateMachineBlueprint, aspect_box: Box<dyn Any>) -> Result<(), String> {
    // Try each type using downcast which returns Err(original) if failed
    let mut current = aspect_box;

    // Try i32
    match current.downcast::<StateAspect<i32>>() {
        Ok(aspect) => {
            blueprint.add_aspect(*aspect);
            return Ok(());
        }
        Err(rest) => current = rest,
    }

    // Try f64
    match current.downcast::<StateAspect<f64>>() {
        Ok(aspect) => {
            blueprint.add_aspect(*aspect);
            return Ok(());
        }
        Err(rest) => current = rest,
    }

    // Try bool
    match current.downcast::<StateAspect<bool>>() {
        Ok(aspect) => {
            blueprint.add_aspect(*aspect);
            return Ok(());
        }
        Err(rest) => current = rest,
    }

    // Try String
    match current.downcast::<StateAspect<String>>() {
        Ok(aspect) => {
            blueprint.add_aspect(*aspect);
            return Ok(());
        }
        Err(rest) => current = rest,
    }

    // Try legacy
    match current.downcast::<StateAspectLegacy>() {
        Ok(aspect) => {
            blueprint.add_aspect_legacy(*aspect);
            return Ok(());
        }
        Err(_) => {
            return Err(format!("Unknown aspect type"));
        }
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
    fn test_blueprint_add_aspect_generic() {
        let mut blueprint = StateMachineBlueprint::new("test_machine");

        let aspect: StateAspect<i32> = StateAspect::new(AspectId(0), "counter", 0);

        blueprint.add_aspect(aspect);

        assert_eq!(blueprint.aspects().count(), 1);
    }

    #[test]
    fn test_blueprint_add_aspect_with_bounds() {
        let mut blueprint = StateMachineBlueprint::new("test_machine");

        let aspect: StateAspect<i32> = StateAspect::new(AspectId(0), "counter", 50)
            .with_range(0, 100);

        blueprint.add_aspect(aspect);

        let descriptor = blueprint.get_aspect(AspectId(0)).unwrap();
        assert!(descriptor.has_min);
        assert!(descriptor.has_max);
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

        let aspect1: StateAspect<String> = StateAspect::new(AspectId(0), "mode", "idle".to_string());
        let aspect2: StateAspect<i64> = StateAspect::new(AspectId(1), "count", 0i64);

        blueprint.add_aspect(aspect1);
        blueprint.add_aspect(aspect2);

        let state = blueprint.create_initial_state();

        assert_eq!(state.get_as::<String>(AspectId(0)), Some(&"idle".to_string()));
        assert_eq!(state.get_as::<i64>(AspectId(1)), Some(&0i64));
    }

    #[test]
    fn test_blueprint_stats() {
        let mut blueprint = StateMachineBlueprint::new("test_machine");

        let aspect: StateAspect<String> = StateAspect::new(AspectId(0), "mode", "idle".to_string());
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
        let aspect: StateAspect<String> = StateAspect::new(AspectId(0), "mode", "idle".to_string());
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

    #[test]
    fn test_blueprint_builder_multiple_types() {
        let aspect1: StateAspect<i32> = StateAspect::new(AspectId(0), "count", 0);
        let aspect2: StateAspect<f64> = StateAspect::new(AspectId(1), "temperature", 20.0)
            .with_range(0.0, 100.0);
        let aspect3: StateAspect<bool> = StateAspect::new(AspectId(2), "enabled", true);

        let blueprint = BlueprintBuilder::new()
            .id("test_machine")
            .aspect(aspect1)
            .aspect(aspect2)
            .aspect(aspect3)
            .build()
            .unwrap();

        assert_eq!(blueprint.aspects().count(), 3);
    }
}