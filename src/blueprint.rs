use crate::core::{AspectId, ClonableAny, EventId};
use crate::aspect::AspectBlueprint;
use crate::state::State;
use crate::zone::Zone;
use crate::transition::Transition;
use std::any::{Any, TypeId};
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
        let cloned_value: Box<dyn ClonableAny> = if let Some(b) = self.default_value.as_any().downcast_ref::<bool>() {
            Box::new(*b) as Box<dyn ClonableAny>
        } else if let Some(i) = self.default_value.as_any().downcast_ref::<i64>() {
            Box::new(*i) as Box<dyn ClonableAny>
        } else if let Some(f) = self.default_value.as_any().downcast_ref::<f64>() {
            Box::new(*f) as Box<dyn ClonableAny>
        } else if let Some(s) = self.default_value.as_any().downcast_ref::<String>() {
            Box::new(s.clone()) as Box<dyn ClonableAny>
        } else if let Some(i) = self.default_value.as_any().downcast_ref::<i32>() {
            Box::new(*i) as Box<dyn ClonableAny>
        } else if let Some(u) = self.default_value.as_any().downcast_ref::<usize>() {
            Box::new(*u) as Box<dyn ClonableAny>
        } else if let Some(u) = self.default_value.as_any().downcast_ref::<u32>() {
            Box::new(*u) as Box<dyn ClonableAny>
        } else if let Some(u) = self.default_value.as_any().downcast_ref::<u64>() {
            Box::new(*u) as Box<dyn ClonableAny>
        } else if let Some(c) = self.default_value.as_any().downcast_ref::<char>() {
            Box::new(*c) as Box<dyn ClonableAny>
        } else if let Some(v) = self.default_value.as_any().downcast_ref::<Vec<u8>>() {
            Box::new(v.clone()) as Box<dyn ClonableAny>
        } else if let Some(v) = self.default_value.as_any().downcast_ref::<Vec<String>>() {
            Box::new(v.clone()) as Box<dyn ClonableAny>
        } else if let Some(v) = self.default_value.as_any().downcast_ref::<Vec<i64>>() {
            Box::new(v.clone()) as Box<dyn ClonableAny>
        } else if let Some(v) = self.default_value.as_any().downcast_ref::<Vec<f64>>() {
            Box::new(v.clone()) as Box<dyn ClonableAny>
        } else if let Some(v) = self.default_value.as_any().downcast_ref::<Vec<bool>>() {
            Box::new(v.clone()) as Box<dyn ClonableAny>
        } else {
            Box::new(()) as Box<dyn ClonableAny>
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
            let cloned_value: Box<dyn ClonableAny> = if let Some(b) = descriptor.default_value.as_any().downcast_ref::<bool>() {
                Box::new(*b) as Box<dyn ClonableAny>
            } else if let Some(i) = descriptor.default_value.as_any().downcast_ref::<i64>() {
                Box::new(*i) as Box<dyn ClonableAny>
            } else if let Some(f) = descriptor.default_value.as_any().downcast_ref::<f64>() {
                Box::new(*f) as Box<dyn ClonableAny>
            } else if let Some(s) = descriptor.default_value.as_any().downcast_ref::<String>() {
                Box::new(s.clone()) as Box<dyn ClonableAny>
            } else if let Some(i) = descriptor.default_value.as_any().downcast_ref::<i32>() {
                Box::new(*i) as Box<dyn ClonableAny>
            } else if let Some(u) = descriptor.default_value.as_any().downcast_ref::<usize>() {
                Box::new(*u) as Box<dyn ClonableAny>
            } else if let Some(u) = descriptor.default_value.as_any().downcast_ref::<u32>() {
                Box::new(*u) as Box<dyn ClonableAny>
            } else if let Some(u) = descriptor.default_value.as_any().downcast_ref::<u64>() {
                Box::new(*u) as Box<dyn ClonableAny>
            } else if let Some(c) = descriptor.default_value.as_any().downcast_ref::<char>() {
                Box::new(*c) as Box<dyn ClonableAny>
            } else if let Some(v) = descriptor.default_value.as_any().downcast_ref::<Vec<u8>>() {
                Box::new(v.clone()) as Box<dyn ClonableAny>
            } else if let Some(v) = descriptor.default_value.as_any().downcast_ref::<Vec<String>>() {
                Box::new(v.clone()) as Box<dyn ClonableAny>
            } else if let Some(v) = descriptor.default_value.as_any().downcast_ref::<Vec<i64>>() {
                Box::new(v.clone()) as Box<dyn ClonableAny>
            } else if let Some(v) = descriptor.default_value.as_any().downcast_ref::<Vec<f64>>() {
                Box::new(v.clone()) as Box<dyn ClonableAny>
            } else if let Some(v) = descriptor.default_value.as_any().downcast_ref::<Vec<bool>>() {
                Box::new(v.clone()) as Box<dyn ClonableAny>
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