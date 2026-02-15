use crate::blueprint::StateMachineBlueprint;
use crate::transition::EventId;
use crate::zone::ZoneId;
use crate::aspect::State;
use std::collections::HashMap;

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
                let new_state = transition.apply(self.state.clone());

                self.state = new_state;
                
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;
    use crate::active_in::ActiveIn;
    use crate::update::Update;
    use crate::blueprint::StateMachineBlueprint;

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
            ActiveIn::aspect_string_eq(AspectId(0), "idle"),
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

        let zone = Zone::new(ZoneId(0), "low_battery", ActiveIn::aspect_lt(AspectId(1), 20));

        let transition = Transition::new(
            TransitionId(0),
            "consume",
            ActiveIn::always(),
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
            ActiveIn::always(),
            EventId::new("start"),
            Update::set_string(AspectId(0), "running"),
        );

        let zone = Zone::new(ZoneId(0), "running", ActiveIn::aspect_string_eq(AspectId(0), "running"));

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