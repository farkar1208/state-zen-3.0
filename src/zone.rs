use crate::active_in::{ActiveIn, ActiveInBlueprint};
use std::fmt;

// ============================================================================
// ID TYPES
// ============================================================================

/// Unique identifier for a Zone
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ZoneId(pub usize);

// ============================================================================
// BLUEPRINT LAYER - ZoneBlueprint (without side effects)
// ============================================================================

/// Blueprint for a Zone (declaration layer, no side effect handlers)
///
/// A ZoneBlueprint defines a region in the state space.
/// It does NOT include side effect handlers (on_enter, on_exit).
#[derive(Debug, Clone)]
pub struct ZoneBlueprint {
    pub id: ZoneId,
    pub name: String,
    /// Defines the state collection covered by this zone
    pub active_in: ActiveInBlueprint,
}

impl ZoneBlueprint {
    /// Create a new ZoneBlueprint
    pub fn new(id: ZoneId, name: impl Into<String>, active_in: ActiveInBlueprint) -> Self {
        Self {
            id,
            name: name.into(),
            active_in,
        }
    }
}

// ============================================================================
// RUNTIME LAYER - Zone (with side effect handlers)
// ============================================================================

/// Represents a state behavior area with lifecycle semantics
///
/// A Zone defines a region in the state space with associated side effects
/// that trigger when entering or leaving that region.
pub struct Zone {
    pub id: ZoneId,
    pub name: String,
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
            .field("name", &self.name)
            .field("has_on_enter", &self.on_enter.is_some())
            .field("has_on_exit", &self.on_exit.is_some())
            .finish()
    }
}

impl PartialEq for Zone {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Zone {}

/// Side effect handler for zone lifecycle events
pub type ZoneHandler = Box<dyn Fn() + Send + Sync>;

impl Zone {
    /// Create a new Zone with runtime ActiveIn
    pub fn new(id: ZoneId, name: impl Into<String>, active_in: ActiveIn) -> Self {
        Self {
            id,
            name: name.into(),
            active_in,
            on_enter: None,
            on_exit: None,
        }
    }

    /// Create a new Zone from a blueprint
    pub fn from_blueprint(blueprint: ZoneBlueprint) -> Self {
        Self {
            id: blueprint.id,
            name: blueprint.name,
            active_in: ActiveIn::from_blueprint(blueprint.active_in),
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
    id: Option<ZoneId>,
    name: Option<String>,
    active_in: Option<ActiveIn>,
    on_enter: Option<ZoneHandler>,
    on_exit: Option<ZoneHandler>,
}

impl ZoneBuilder {
    pub fn new() -> Self {
        Self {
            id: None,
            name: None,
            active_in: None,
            on_enter: None,
            on_exit: None,
        }
    }

    pub fn id(mut self, id: ZoneId) -> Self {
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
        let name = self.name.ok_or("Zone name is required")?;
        let active_in = self.active_in.ok_or("active_in is required")?;

        Ok(Zone {
            id,
            name,
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
    use crate::aspect::AspectId;
    use crate::aspect::StateBuilder;

    #[test]
    fn test_zone_blueprint() {
        let aspect_id = AspectId(0);
        let zone_id = ZoneId(0);
        let active_in = ActiveInBlueprint::aspect_bool(aspect_id, true);

        let blueprint = ZoneBlueprint::new(zone_id, "test_zone", active_in);

        assert_eq!(blueprint.id, ZoneId(0));
        assert_eq!(blueprint.name, "test_zone");
    }

    #[test]
    fn test_zone_from_blueprint() {
        let aspect_id = AspectId(0);
        let zone_id = ZoneId(0);
        let active_in = ActiveInBlueprint::aspect_bool(aspect_id, true);

        let blueprint = ZoneBlueprint::new(zone_id, "test_zone", active_in);
        let zone = Zone::from_blueprint(blueprint);

        assert_eq!(zone.id, ZoneId(0));
        assert_eq!(zone.name, "test_zone");
        assert!(zone.on_enter.is_none());
        assert!(zone.on_exit.is_none());
    }

    #[test]
    fn test_zone_creation() {
        let aspect_id = AspectId(0);
        let zone_id = ZoneId(0);
        let active_in = ActiveIn::aspect_bool(aspect_id, true);

        let zone = Zone::new(zone_id, "test_zone", active_in);

        assert_eq!(zone.id, ZoneId(0));
        assert_eq!(zone.name, "test_zone");
        assert!(zone.on_enter.is_none());
        assert!(zone.on_exit.is_none());
    }

    #[test]
    fn test_zone_equality() {
        let zone_id = ZoneId(0);
        let active_in = ActiveIn::always();

        let zone1 = Zone::new(zone_id, "zone1", active_in.clone());
        let zone2 = Zone::new(zone_id, "zone2", active_in.clone());
        let zone3 = Zone::new(ZoneId(1), "zone3", active_in);

        assert_eq!(zone1, zone2);  // Same ID, different names
        assert_ne!(zone1, zone3);  // Different IDs
    }

    #[test]
    fn test_zone_with_handlers() {
        let aspect_id = AspectId(0);
        let zone_id = ZoneId(0);
        let active_in = ActiveIn::aspect_bool(aspect_id, true);

        let zone = Zone::new(zone_id, "test_zone", active_in)
            .with_on_enter(|| {})
            .with_on_exit(|| {});

        assert!(zone.on_enter.is_some());
        assert!(zone.on_exit.is_some());
    }

    #[test]
    fn test_zone_activation() {
        let aspect_id = AspectId(0);
        let zone_id = ZoneId(0);

        let active_in = ActiveIn::aspect_bool(aspect_id, true);
        let zone = Zone::new(zone_id, "test_zone", active_in);

        let state_active = StateBuilder::new()
            .set_bool(aspect_id, true)
            .build();

        let state_inactive = StateBuilder::new()
            .set_bool(aspect_id, false)
            .build();

        assert!(zone.is_active(&state_active));
        assert!(!zone.is_active(&state_inactive));
    }

    #[test]
    fn test_zone_builder() {
        let aspect_id = AspectId(0);
        let zone_id = ZoneId(0);
        let active_in = ActiveIn::aspect_bool(aspect_id, true);

        let zone = ZoneBuilder::new()
            .id(zone_id)
            .name("test_zone")
            .active_in(active_in)
            .on_enter(|| {})
            .on_exit(|| {})
            .build()
            .unwrap();

        assert_eq!(zone.id, ZoneId(0));
        assert_eq!(zone.name, "test_zone");
        assert!(zone.on_enter.is_some());
        assert!(zone.on_exit.is_some());
    }
}