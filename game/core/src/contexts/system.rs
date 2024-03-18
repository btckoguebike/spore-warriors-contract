extern crate alloc;
use alloc::vec::Vec;
use rlp::{RlpDecodable, RlpEncodable};

use crate::wrappings::System;

#[cfg(feature = "json_serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "debug", derive(Debug, PartialEq))]
#[cfg_attr(feature = "json_serde", derive(Serialize, Deserialize))]
#[derive(Clone, RlpEncodable, RlpDecodable)]
pub struct SystemContext {
    pub system: System,
    pub duration_counter: u16,
    pub register_1: Option<u16>,
    pub register_2: Option<u16>,
    pub register_3: Vec<usize>,
}

impl SystemContext {
    pub fn new(system: System) -> Self {
        Self {
            system,
            duration_counter: 0,
            register_1: None,
            register_2: None,
            register_3: Default::default(),
        }
    }

    pub fn equal(&self, other: &Self) -> bool {
        if self.system.id != other.system.id || self.system.args != other.system.args {
            return false;
        }
        match (self.system.duration, other.system.duration) {
            (Some(v1), Some(v2)) => v1.trigger == v2.trigger,
            (None, Some(_)) | (Some(_), None) => false,
            (None, None) => true,
        }
    }

    pub fn durable_combine(&mut self, other: &Self) -> bool {
        let Some(duration) = &self.system.duration else {
            return false;
        };
        if self.equal(other) {
            if let Some(other_duration) = other.system.duration {
                if duration.trigger == other_duration.trigger {
                    self.duration_counter += other.duration_counter;
                    return true;
                }
            }
        }
        false
    }

    pub fn durable_update(&mut self, other: &Self) -> bool {
        let Some(duration) = &self.system.duration else {
            return false;
        };
        if self.equal(other) {
            if let Some(other_duration) = other.system.duration {
                if duration.trigger == other_duration.trigger {
                    self.duration_counter = other.duration_counter;
                    // TODO: combine registers?
                    return true;
                }
            }
        }
        false
    }

    pub fn is_durable(&self) -> bool {
        self.system.duration.is_some()
    }
}

impl From<&System> for SystemContext {
    fn from(value: &System) -> Self {
        Self::new(value.clone())
    }
}

impl From<System> for SystemContext {
    fn from(value: System) -> Self {
        Self::new(value)
    }
}
