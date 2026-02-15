use crate::aspect::{AspectId, State, Aspect};
use crate::zone::{Zone, ZoneId};
use crate::transition::{EventId, Transition, TransitionId};
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
    pub fn new<T>(aspect: &Aspect<T>) -> Self
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
    pub fn add_aspect<T>(&mut self, aspect: Aspect<T>) -> &mut Self
    where
        T: Any + Send + Sync + Clone,
    {
        let descriptor = AspectDescriptor::new(&aspect);
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
                continue;
            };
            builder = builder.set(descriptor.id, cloned_value);
        }
        builder.build()
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

        let aspect: Aspect<i32> = Aspect::new(AspectId(0), "counter", 0);

        blueprint.add_aspect(aspect);

        assert_eq!(blueprint.aspects().count(), 1);
    }

    #[test]
    fn test_blueprint_add_aspect_with_bounds() {
        let mut blueprint = StateMachineBlueprint::new("test_machine");

        let aspect: Aspect<i32> = Aspect::new(AspectId(0), "counter", 50)
            .with_range(0, 100);

        blueprint.add_aspect(aspect);

        let descriptor = blueprint.get_aspect(AspectId(0)).unwrap();
        assert!(descriptor.has_min);
        assert!(descriptor.has_max);
    }

    #[test]
    fn test_blueprint_add_zone() {
        let mut blueprint = StateMachineBlueprint::new("test_machine");

        let zone = Zone::new(ZoneId(0), "test_zone", ActiveIn::always());

        blueprint.add_zone(zone);

        assert_eq!(blueprint.zones().len(), 1);
    }

    #[test]
    fn test_blueprint_add_transition() {
        let mut blueprint = StateMachineBlueprint::new("test_machine");

        let transition = Transition::new(
            TransitionId(0),
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

        let aspect1: Aspect<String> = Aspect::new(AspectId(0), "mode", "idle".to_string());
        let aspect2: Aspect<i64> = Aspect::new(AspectId(1), "count", 0i64);

        blueprint.add_aspect(aspect1);
        blueprint.add_aspect(aspect2);

        let state = blueprint.create_initial_state();

        assert_eq!(state.get_as::<String>(AspectId(0)), Some(&"idle".to_string()));
        assert_eq!(state.get_as::<i64>(AspectId(1)), Some(&0i64));
    }

    #[test]
    fn test_blueprint_builder_multiple_types() {
        let aspect1: Aspect<i32> = Aspect::new(AspectId(0), "count", 0);
        let aspect2: Aspect<f64> = Aspect::new(AspectId(1), "temperature", 20.0)
            .with_range(0.0, 100.0);
        let aspect3: Aspect<bool> = Aspect::new(AspectId(2), "enabled", true);

        let mut blueprint = StateMachineBlueprint::new("test_machine");
        blueprint.add_aspect(aspect1);
        blueprint.add_aspect(aspect2);
        blueprint.add_aspect(aspect3);

        assert_eq!(blueprint.aspects().count(), 3);
    }
}