use crate::active_in::{ActiveIn, ActiveInBlueprint};
use crate::update::{Update, UpdateBlueprint};
use crate::aspect::State;
use std::fmt;

/// Represents a state transition triggered by an event
///
/// A Transition describes how the system responds to events and evolves state.
/// It only listens for events when its activeIn condition is true.
pub struct Transition {
    pub id: String,
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
            .field("event", &self.event)
            .field("has_on_tran", &self.on_tran.is_some())
            .finish()
    }
}

/// Unique identifier for an event type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EventId(pub String);

impl EventId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

/// Side effect handler for transition events
pub type TransitionHandler = Box<dyn Fn() + Send + Sync>;

impl Transition {
    /// Create a new Transition with runtime ActiveIn
    pub fn new(id: impl Into<String>, active_in: ActiveIn, event: EventId, update: Update) -> Self {
        Self {
            id: id.into(),
            active_in,
            event,
            update,
            on_tran: None,
        }
    }

    /// Create a new Transition from a blueprint ActiveInBlueprint
    pub fn from_blueprint(
        id: impl Into<String>,
        active_in: ActiveInBlueprint,
        event: EventId,
        update: Update,
    ) -> Self {
        Self {
            id: id.into(),
            active_in: ActiveIn::from_blueprint(active_in),
            event,
            update,
            on_tran: None,
        }
    }

    /// Create a new Transition from full blueprint (ActiveInBlueprint + UpdateBlueprint)
    pub fn from_blueprint_full(
        id: impl Into<String>,
        active_in: ActiveInBlueprint,
        event: EventId,
        update: UpdateBlueprint,
    ) -> Self {
        Self {
            id: id.into(),
            active_in: ActiveIn::from_blueprint(active_in),
            event,
            update: Update::from_blueprint(update),
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
    id: Option<String>,
    active_in: Option<ActiveIn>,
    event: Option<EventId>,
    update: Option<Update>,
    on_tran: Option<TransitionHandler>,
}

impl TransitionBuilder {
    pub fn new() -> Self {
        Self {
            id: None,
            active_in: None,
            event: None,
            update: None,
            on_tran: None,
        }
    }

    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
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
        let active_in = self.active_in.ok_or("active_in is required")?;
        let event = self.event.ok_or("event is required")?;
        let update = self.update.ok_or("update is required")?;

        Ok(Transition {
            id,
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
    use crate::active_in::ActiveIn;
    use crate::aspect::{AspectId, StateBuilder};
    use crate::update::Update;

    #[test]
    fn test_transition_creation() {
        let mode_id = AspectId(0);
        let active_in = ActiveIn::aspect_bool(mode_id, true);
        let event = EventId::new("start");
        let update = Update::set_bool(mode_id, false);

        let transition = Transition::new("test_transition", active_in, event, update);

        assert_eq!(transition.id, "test_transition");
        assert_eq!(transition.event, EventId::new("start"));
        assert!(transition.on_tran.is_none());
    }

    #[test]
    fn test_transition_activation() {
        let mode_id = AspectId(0);
        let active_in = ActiveIn::aspect_bool(mode_id, true);
        let event = EventId::new("start");
        let update = Update::noop();

        let transition = Transition::new("test_transition", active_in, event, update);

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
        let active_in = ActiveIn::always();
        let event = EventId::new("start");
        let update = Update::set_bool(mode_id, false);

        let transition = Transition::new("test_transition", active_in, event, update);

        let state = StateBuilder::new()
            .set_bool(mode_id, true)
            .build();

        let new_state = transition.apply(state);

        assert_eq!(new_state.get_as::<bool>(mode_id), Some(&false));
    }

    #[test]
    fn test_transition_builder() {
        let mode_id = AspectId(0);
        let active_in = ActiveIn::always();
        let event = EventId::new("start");
        let update = Update::set_bool(mode_id, false);

        let transition = TransitionBuilder::new()
            .id("test_transition")
            .active_in(active_in)
            .event(event)
            .update(update)
            .on_tran(|| {})
            .build()
            .unwrap();

        assert_eq!(transition.id, "test_transition");
        assert!(transition.on_tran.is_some());
    }
}