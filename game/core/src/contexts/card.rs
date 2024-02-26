extern crate alloc;
use alloc::vec;
use rlp::{RlpDecodable, RlpEncodable};

use crate::contexts::{ContextType, CtxAdaptor};
use crate::errors::Error;
use crate::wrappings::{Card, Value};

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, RlpEncodable, RlpDecodable)]
pub struct CardContext {
    pub card: Card,
    pub power_cost: u8,
    pub extra_args: Vec<Value>,
}

impl CardContext {
    pub fn new(card: Card) -> Self {
        Self {
            power_cost: card.power_cost,
            extra_args: vec![],
            card,
        }
    }
}

impl CtxAdaptor for CardContext {
    fn context_type(&self) -> ContextType {
        ContextType::Card
    }

    fn offset(&self) -> usize {
        self.card.unique_id as usize
    }

    fn card(&mut self) -> Result<&mut CardContext, Error> {
        Ok(self)
    }
}
