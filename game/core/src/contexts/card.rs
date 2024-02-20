extern crate alloc;
use alloc::vec;

use crate::contexts::{ContextType, CtxAdaptor};
use crate::errors::Error;
use crate::wrappings::{Card, Value};

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct CardContext<'a> {
    pub card: &'a Card,
    pub offset: usize,
    pub power_cost: u8,
    pub extra_args: Vec<Value>,
}

impl<'a> CardContext<'a> {
    pub fn new(card: &'a Card, offset: usize) -> Self {
        Self {
            card,
            offset,
            power_cost: card.power_cost,
            extra_args: vec![],
        }
    }
}

impl<'a> CtxAdaptor<'a> for CardContext<'a> {
    fn context_type(&self) -> ContextType {
        ContextType::Card
    }

    fn offset(&self) -> usize {
        self.offset
    }

    fn card(&mut self) -> Result<&mut CardContext<'a>, Error> {
        Ok(self)
    }
}
