pub mod aspect;
pub mod active_in;
pub mod zone;
pub mod transition;
pub mod update;
pub mod blueprint;
pub mod runtime;

// Export core types
pub use aspect::{
    AspectId, StateValue, State, StateBuilder,
    StateAspect, StateAspectLegacy, Bounds, validate_bounds, any_value,
};
pub use active_in::{ActiveIn, Predicate};
pub use zone::Zone;
pub use transition::Transition;
pub use update::Update;
pub use blueprint::{StateMachineBlueprint, AspectDescriptor, BlueprintBuilder};
pub use runtime::StateMachineRuntime;

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::aspect::{
        AspectId, StateValue, State, StateBuilder,
        StateAspect, StateAspectLegacy, Bounds, validate_bounds, any_value,
    };
    pub use crate::active_in::{ActiveIn, Predicate};
    pub use crate::zone::Zone;
    pub use crate::transition::Transition;
    pub use crate::update::Update;
    pub use crate::blueprint::{StateMachineBlueprint, AspectDescriptor, BlueprintBuilder};
    pub use crate::runtime::StateMachineRuntime;
}