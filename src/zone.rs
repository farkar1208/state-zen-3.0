use crate::active_in::ActiveIn;
use std::fmt;

/// Represents a state behavior area with lifecycle semantics
///
/// A Zone defines a region in the state space with associated side effects
/// that trigger when entering or leaving that region.
pub struct Zone {
    pub id: String,
    /// Defines the state collection covered by this zone
    pub active_in: ActiveIn,
    /// Side effect triggered when entering this region
    pub on_enter: Option<ZoneHandler>,
    /// Side effect triggered when leaving this region
    pub on_exit: Option<ZoneHandler>,
}

impl fmt::Debug for Zone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Zone")
            .field("id", &self.id)
            .field("has_on_enter", &self.on_enter.is_some())
            .field("has_on_exit", &self.on_exit.is_some())
            .finish()
    }
}

/// Side effect handler for zone lifecycle events
pub type ZoneHandler = Box<dyn Fn() + Send + Sync>;

impl Zone {
    /// Create a new Zone
    pub fn new(id: impl Into<String>, active_in: ActiveIn) -> Self {
        Self {
            id: id.into(),
            active_in,
            on_enter: None,
            on_exit: None,
        }
    }

    /// Set the on_enter handler
    pub fn with_on_enter<F>(mut self, handler: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_enter = Some(Box::new(handler));
        self
    }

    /// Set the on_exit handler
    pub fn with_on_exit<F>(mut self, handler: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_exit = Some(Box::new(handler));
        self
    }

    /// Evaluate whether this zone is active in the given state
    pub fn is_active(&self, state: &crate::aspect::State) -> bool {
        self.active_in.evaluate(state)
    }

    /// Execute the on_enter handler if present
    pub fn enter(&self) {
        if let Some(handler) = &self.on_enter {
            handler();
        }
    }

    /// Execute the on_exit handler if present
    pub fn exit(&self) {
        if let Some(handler) = &self.on_exit {
            handler();
        }
    }
}

/// Builder for constructing Zone instances
pub struct ZoneBuilder {
    id: Option<String>,
    active_in: Option<ActiveIn>,
    on_enter: Option<ZoneHandler>,
    on_exit: Option<ZoneHandler>,
}

impl ZoneBuilder {
    pub fn new() -> Self {
        Self {
            id: None,
            active_in: None,
            on_enter: None,
            on_exit: None,
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

    pub fn on_enter<F>(mut self, handler: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_enter = Some(Box::new(handler));
        self
    }

    pub fn on_exit<F>(mut self, handler: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_exit = Some(Box::new(handler));
        self
    }

    pub fn build(self) -> Result<Zone, String> {
        let id = self.id.ok_or("Zone id is required")?;
        let active_in = self.active_in.ok_or("active_in is required")?;

        Ok(Zone {
            id,
            active_in,
            on_enter: self.on_enter,
            on_exit: self.on_exit,
        })
    }
}

impl Default for ZoneBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::active_in::ActiveIn;
    use crate::aspect::{AspectId, StateBuilder, StateValue};

    #[test]
    fn test_zone_creation() {
        let id = AspectId(0);
        let active_in = ActiveIn::aspect_bool(id, true);

        let zone = Zone::new("test_zone", active_in);

        assert_eq!(zone.id, "test_zone");
        assert!(zone.on_enter.is_none());
        assert!(zone.on_exit.is_none());
    }

    #[test]
    fn test_zone_with_handlers() {
        let id = AspectId(0);
        let active_in = ActiveIn::aspect_bool(id, true);

        let zone = Zone::new("test_zone", active_in)
            .with_on_enter(|| {})
            .with_on_exit(|| {});

        assert!(zone.on_enter.is_some());
        assert!(zone.on_exit.is_some());
    }

    #[test]
    fn test_zone_activation() {
        let id = AspectId(0);

        let active_in = ActiveIn::aspect_bool(id, true);
        let zone = Zone::new("test_zone", active_in);

        let state_active = StateBuilder::new()
            .set(id, StateValue::Bool(true))
            .build();

        let state_inactive = StateBuilder::new()
            .set(id, StateValue::Bool(false))
            .build();

        assert!(zone.is_active(&state_active));
        assert!(!zone.is_active(&state_inactive));
    }

    #[test]
    fn test_zone_builder() {
        let id = AspectId(0);
        let active_in = ActiveIn::aspect_bool(id, true);

        let zone = ZoneBuilder::new()
            .id("test_zone")
            .active_in(active_in)
            .on_enter(|| {})
            .on_exit(|| {})
            .build()
            .unwrap();

        assert_eq!(zone.id, "test_zone");
        assert!(zone.on_enter.is_some());
        assert!(zone.on_exit.is_some());
    }
}