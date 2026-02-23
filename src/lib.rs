pub mod core;
pub mod aspect;
pub mod state;
pub mod active_in;
pub mod zone;
pub mod transition;
pub mod update;
pub mod statemachine;

// Export core types
pub use core::{ClonableAny, AspectId, EventId};
pub use aspect::{AspectBlueprint, AspectBoundsBlueprint};
pub use state::{State, StateBuilder};
pub use active_in::{ActiveIn, ActiveInBlueprint, ActiveInFactory, Predicate};
pub use zone::{Zone, ZoneBlueprint, ZoneId};
pub use transition::{Transition, TransitionBlueprint, TransitionId};
pub use update::{Update, UpdateBlueprint};
// Re-export from statemachine module for backward compatibility
pub use statemachine::{StateMachineBlueprint, AspectDescriptor, StateMachineRuntime, ValidationError};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::core::{ClonableAny, AspectId, EventId};
    pub use crate::aspect::{AspectBlueprint, AspectBoundsBlueprint};
    pub use crate::state::{State, StateBuilder};
    pub use crate::active_in::{ActiveIn, ActiveInBlueprint, ActiveInFactory, Predicate};
    pub use crate::zone::{Zone, ZoneBlueprint, ZoneId};
    pub use crate::transition::{Transition, TransitionBlueprint, TransitionId};
    pub use crate::update::{Update, UpdateBlueprint};
    pub use crate::statemachine::{StateMachineBlueprint, AspectDescriptor, StateMachineRuntime, ValidationError};
}