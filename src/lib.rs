pub mod aspect;
pub mod active_in;
pub mod zone;
pub mod transition;
pub mod update;
pub mod blueprint;
pub mod runtime;

// Export core types
pub use aspect::{
    AspectId, State, StateBuilder,
    Aspect, AspectBlueprint, AspectBoundsBlueprint, AspectBlueprintBuilder,
    Bounds, validate_bounds, any_value,
};
pub use active_in::{ActiveIn, ActiveInBlueprint, ActiveInBlueprintBuilder, Predicate};
pub use zone::{Zone, ZoneBlueprint, ZoneId};
pub use transition::{Transition, TransitionBlueprint, TransitionId, EventId};
pub use update::{Update, UpdateBlueprint, UpdateBlueprintBuilder};
pub use blueprint::{StateMachineBlueprint, AspectDescriptor, BlueprintBuilder};
pub use runtime::StateMachineRuntime;

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::aspect::{
        AspectId, State, StateBuilder,
        Aspect, AspectBlueprint, AspectBoundsBlueprint, AspectBlueprintBuilder,
        Bounds, validate_bounds, any_value,
    };
    pub use crate::active_in::{ActiveIn, ActiveInBlueprint, ActiveInBlueprintBuilder, Predicate};
    pub use crate::zone::{Zone, ZoneBlueprint, ZoneId};
    pub use crate::transition::{Transition, TransitionBlueprint, TransitionId, EventId};
    pub use crate::update::{Update, UpdateBlueprint, UpdateBlueprintBuilder};
    pub use crate::blueprint::{StateMachineBlueprint, AspectDescriptor, BlueprintBuilder};
    pub use crate::runtime::StateMachineRuntime;
}