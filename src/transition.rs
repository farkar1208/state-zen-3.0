use crate::active_in::{ActiveIn, ActiveInBlueprint};
use crate::update::{Update, UpdateBlueprint};
use crate::aspect::State;
use std::fmt;

// ============================================================================
// ID TYPES
// ============================================================================

/// Unique identifier for a Transition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransitionId(pub usize);

/// Unique identifier for an event type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EventId(pub String);

impl EventId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

// ============================================================================
// BLUEPRINT LAYER - TransitionBlueprint (without side effects)
// ============================================================================

/// Blueprint for a Transition (declaration layer, no side effect handlers)
///
/// A TransitionBlueprint describes how the system responds to events and evolves state.
/// It does NOT include side effect handlers (on_tran).
#[derive(Debug, Clone)]
pub struct TransitionBlueprint {
    pub id: TransitionId,
    pub name: String,
    /// When this transition should listen for events
    pub active_in: ActiveInBlueprint,
    /// The event type to listen for
    pub event: EventId,
    /// How to compute the new state (pure function)
    pub update: UpdateBlueprint,
}

impl TransitionBlueprint {
    /// Create a new TransitionBlueprint
    pub fn new(
        id: TransitionId,
        name: impl Into<String>,
        active_in: ActiveInBlueprint,
        event: EventId,
        update: UpdateBlueprint,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            active_in,
            event,
            update,
        }
    }
}

// ============================================================================
// RUNTIME LAYER - Transition (with side effect handlers)
// ============================================================================

/// Represents a state transition triggered by an event
///
/// A Transition describes how the system responds to events and evolves state.
/// It only listens for events when its activeIn condition is true.
pub struct Transition {
    pub id: TransitionId,
    pub name: String,
    /// When this transition should listen for events
    pub active_in: ActiveIn,
    /// The event type to listen for
    pub event: EventId,
    /// How to compute the new state (pure function)
    pub update: Update,
    /// Side effect triggered when transition occurs
    pub on_tran: Option<TransitionHandler>,
}

impl fmt::Debug for Transition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Transition")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("event", &self.event)
            .field("has_on_tran", &self.on_tran.is_some())
            .finish()
    }
}

impl PartialEq for Transition {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Transition {}

/// Side effect handler for transition events
pub type TransitionHandler = Box<dyn Fn() + Send + Sync>;

impl Transition {
    /// Create a new Transition with runtime ActiveIn
    pub fn new(
        id: TransitionId,
        name: impl Into<String>,
        active_in: ActiveIn,
        event: EventId,
        update: Update,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            active_in,
            event,
            update,
            on_tran: None,
        }
    }

    /// Create a new Transition from a blueprint
    pub fn from_blueprint(blueprint: TransitionBlueprint) -> Self {
        Self {
            id: blueprint.id,
            name: blueprint.name,
            active_in: ActiveIn::from_blueprint(blueprint.active_in),
            event: blueprint.event,
            update: Update::from_blueprint(blueprint.update),
            on_tran: None,
        }
    }

    /// Set the on_tran handler
    pub fn with_on_tran<F>(mut self, handler: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_tran = Some(Box::new(handler));
        self
    }

    /// Check if this transition should be active (listen for events) in the given state
    pub fn is_active(&self, state: &State) -> bool {
        self.active_in.evaluate(state)
    }

    /// Apply the update to the state, returning a new state
    pub fn apply(&self, state: State) -> State {
        self.update.apply(state)
    }

    /// Execute the on_tran handler if present
    pub fn trigger(&self) {
        if let Some(handler) = &self.on_tran {
            handler();
        }
    }
}

/// Builder for constructing Transition instances
pub struct TransitionBuilder {
    id: Option<TransitionId>,
    name: Option<String>,
    active_in: Option<ActiveIn>,
    event: Option<EventId>,
    update: Option<Update>,
    on_tran: Option<TransitionHandler>,
}

impl TransitionBuilder {
    pub fn new() -> Self {
        Self {
            id: None,
            name: None,
            active_in: None,
            event: None,
            update: None,
            on_tran: None,
        }
    }

    pub fn id(mut self, id: TransitionId) -> Self {
        self.id = Some(id);
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn active_in(mut self, active_in: ActiveIn) -> Self {
        self.active_in = Some(active_in);
        self
    }

    pub fn event(mut self, event: EventId) -> Self {
        self.event = Some(event);
        self
    }

    pub fn event_str(mut self, event: impl Into<String>) -> Self {
        self.event = Some(EventId::new(event));
        self
    }

    pub fn update(mut self, update: Update) -> Self {
        self.update = Some(update);
        self
    }

    pub fn on_tran<F>(mut self, handler: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_tran = Some(Box::new(handler));
        self
    }

    pub fn build(self) -> Result<Transition, String> {
        let id = self.id.ok_or("Transition id is required")?;
        let name = self.name.ok_or("Transition name is required")?;
        let active_in = self.active_in.ok_or("active_in is required")?;
        let event = self.event.ok_or("event is required")?;
        let update = self.update.ok_or("update is required")?;

        Ok(Transition {
            id,
            name,
            active_in,
            event,
            update,
            on_tran: self.on_tran,
        })
    }
}

impl Default for TransitionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::active_in::ActiveInFactory;
    use crate::aspect::{AspectId, StateBuilder};
    use crate::update::Update;

    #[test]
    fn test_transition_blueprint() {
        let mode_id = AspectId(0);
        let transition_id = TransitionId(0);
        let active_in = ActiveInBlueprint::aspect_bool(mode_id, true);
        let event = EventId::new("start");
        let update = UpdateBlueprint::set_bool(mode_id, false);

        let blueprint = TransitionBlueprint::new(transition_id, "test_transition", active_in, event, update);

        assert_eq!(blueprint.id, TransitionId(0));
        assert_eq!(blueprint.name, "test_transition");
        assert_eq!(blueprint.event, EventId::new("start"));
    }

    #[test]
    fn test_transition_from_blueprint() {
        let mode_id = AspectId(0);
        let transition_id = TransitionId(0);
        let active_in = ActiveInBlueprint::aspect_bool(mode_id, true);
        let event = EventId::new("start");
        let update = UpdateBlueprint::set_bool(mode_id, false);

        let blueprint = TransitionBlueprint::new(transition_id, "test_transition", active_in, event, update);
        let transition = Transition::from_blueprint(blueprint);

        assert_eq!(transition.id, TransitionId(0));
        assert_eq!(transition.name, "test_transition");
        assert_eq!(transition.event, EventId::new("start"));
        assert!(transition.on_tran.is_none());
    }

    #[test]
    fn test_transition_creation() {
        let mode_id = AspectId(0);
        let transition_id = TransitionId(0);
        let active_in = ActiveInFactory::aspect_bool(mode_id, true);
        let event = EventId::new("start");
        let update = Update::set_bool(mode_id, false);

        let transition = Transition::new(transition_id, "test_transition", active_in, event, update);

        assert_eq!(transition.id, TransitionId(0));
        assert_eq!(transition.name, "test_transition");
        assert_eq!(transition.event, EventId::new("start"));
        assert!(transition.on_tran.is_none());
    }

    #[test]
    fn test_transition_equality() {
        let transition_id = TransitionId(0);
        let active_in = ActiveInFactory::always();
        let event = EventId::new("start");
        let update = Update::noop();

        let transition1 = Transition::new(transition_id, "transition1", active_in.clone(), event.clone(), update.clone());
        let transition2 = Transition::new(transition_id, "transition2", active_in.clone(), event.clone(), update.clone());
        let transition3 = Transition::new(TransitionId(1), "transition3", active_in, event, update);

        assert_eq!(transition1, transition2);  // Same ID, different names
        assert_ne!(transition1, transition3);  // Different IDs
    }

    #[test]
    fn test_transition_activation() {
        let mode_id = AspectId(0);
        let transition_id = TransitionId(0);
        let active_in = ActiveInFactory::aspect_bool(mode_id, true);
        let event = EventId::new("start");
        let update = Update::noop();

        let transition = Transition::new(transition_id, "test_transition", active_in, event, update);

        let state_active = StateBuilder::new()
            .set_bool(mode_id, true)
            .build();

        let state_inactive = StateBuilder::new()
            .set_bool(mode_id, false)
            .build();

        assert!(transition.is_active(&state_active));
        assert!(!transition.is_active(&state_inactive));
    }

    #[test]
    fn test_transition_apply() {
        let mode_id = AspectId(0);
        let transition_id = TransitionId(0);
        let active_in = ActiveInFactory::always();
        let event = EventId::new("start");
        let update = Update::set_bool(mode_id, false);

        let transition = Transition::new(transition_id, "test_transition", active_in, event, update);

        let state = StateBuilder::new()
            .set_bool(mode_id, true)
            .build();

        let new_state = transition.apply(state);

        assert_eq!(new_state.get_as::<bool>(mode_id), Some(&false));
    }

    #[test]
    fn test_transition_builder() {
        let mode_id = AspectId(0);
        let transition_id = TransitionId(0);
        let active_in = ActiveInFactory::always();
        let event = EventId::new("start");
        let update = Update::set_bool(mode_id, false);

        let transition = TransitionBuilder::new()
            .id(transition_id)
            .name("test_transition")
            .active_in(active_in)
            .event(event)
            .update(update)
            .on_tran(|| {})
            .build()
            .unwrap();

        assert_eq!(transition.id, TransitionId(0));
        assert_eq!(transition.name, "test_transition");
        assert!(transition.on_tran.is_some());
    }
}