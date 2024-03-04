extern crate alloc;
use alloc::{vec, vec::Vec};
use rlp::{RlpDecodable, RlpEncodable};

use crate::contexts::{ContextType, CtxAdaptor};
use crate::errors::Error;
use crate::wrappings::{Card, System};

#[cfg_attr(feature = "debug", derive(Debug, PartialEq))]
#[derive(Clone, RlpEncodable, RlpDecodable)]
pub struct CardContext {
    pub card: Card,
    pub power_cost: u8,
    pub max_power_cost: u8,
    pub mounting_system: Vec<System>,
}

impl CardContext {
    pub fn new(card: Card) -> Self {
        Self {
            power_cost: card.power_cost,
            max_power_cost: card.power_cost,
            mounting_system: vec![],
            card,
        }
    }

    pub fn reset(&mut self) {
        self.power_cost = self.max_power_cost;
    }
}

impl CtxAdaptor for CardContext {
    fn context_type(&self) -> ContextType {
        ContextType::Card
    }

    fn offset(&self) -> usize {
        self.card.offset
    }

    fn card(&mut self) -> Result<&mut CardContext, Error> {
        Ok(self)
    }
}
