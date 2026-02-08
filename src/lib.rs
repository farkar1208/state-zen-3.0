pub mod aspect;
pub mod active_in;
pub mod zone;
pub mod transition;
pub mod update;
pub mod blueprint;

pub use aspect::{StateAspect, AspectId, StateValue, State, StateBuilder};
pub use active_in::{ActiveIn, Predicate};
pub use zone::Zone;
pub use transition::Transition;
pub use update::Update;
pub use blueprint::StateMachineBlueprint;

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::aspect::{StateAspect, AspectId, StateValue, State, StateBuilder};
    pub use crate::active_in::{ActiveIn, Predicate};
    pub use crate::zone::Zone;
    pub use crate::transition::Transition;
    pub use crate::update::Update;
    pub use crate::blueprint::StateMachineBlueprint;
}