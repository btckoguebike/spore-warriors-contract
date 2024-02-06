#![no_std]
#![allow(unused)]

mod asset;
mod charactors;
mod game;
mod resources;
mod scene;
mod types;

pub use asset::*;
pub use charactors::*;
pub use game::*;
pub use resources::*;
pub use scene::*;
pub use types::*;

mod casting {
    extern crate alloc;
    use super::*;
    use alloc::vec::Vec;
    use molecule::prelude::{Byte, Entity};

    impl From<Number> for u16 {
        fn from(value: Number) -> Self {
            Self::from_le_bytes(value.as_slice().try_into().unwrap())
        }
    }

    impl From<ResourceId> for u16 {
        fn from(value: ResourceId) -> Self {
            Self::from_le_bytes(value.as_slice().try_into().unwrap())
        }
    }

    impl From<ResourceIdVec> for Vec<u16> {
        fn from(value: ResourceIdVec) -> Self {
            value.into_iter().map(Into::into).collect()
        }
    }

    impl From<SystemId> for u16 {
        fn from(value: SystemId) -> Self {
            Self::from_le_bytes(value.as_slice().try_into().unwrap())
        }
    }

    impl From<Seed> for u64 {
        fn from(value: Seed) -> Self {
            Self::from_le_bytes(value.as_slice().try_into().unwrap())
        }
    }

    impl From<Uint64> for u64 {
        fn from(value: Uint64) -> Self {
            Self::from_le_bytes(value.as_slice().try_into().unwrap())
        }
    }
}

pub use casting::*;
