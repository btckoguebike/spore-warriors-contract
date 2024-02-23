extern crate alloc;
use alloc::vec;

use crate::contexts::{BytesExtractor, BytesPusher, ContextType, CtxAdaptor};
use crate::errors::Error;
use crate::game::ReferContainer;
use crate::wrappings::{Card, Value};

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct CardContext<'a> {
    pub card: &'a Card,
    pub power_cost: u8,
    pub extra_args: Vec<Value>,
}

impl<'a> CardContext<'a> {
    pub fn new(card: &'a Card) -> Self {
        Self {
            card,
            power_cost: card.power_cost,
            extra_args: vec![],
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut pusher = BytesPusher::new();
        pusher.push_u16(self.card.unique_id);
        pusher.push_u8(self.power_cost);
        pusher.push_usize(self.extra_args.len());
        self.extra_args.iter().for_each(|v| {
            pusher.push_u16(v.0);
        });
        pusher.data()
    }

    pub(super) fn deserialize(
        refers: &'a ReferContainer<'a>,
        extractor: &mut BytesExtractor,
    ) -> Result<Self, Error> {
        let unique_id = extractor.pop_u16()?;
        let card = refers.get_card(unique_id)?;
        let mut card_context = Self::new(card);
        card_context.power_cost = extractor.pop_u8()?;
        let len = extractor.pop_usize()?;
        for _ in 0..len {
            let arg = extractor.pop_u16()?;
            card_context.extra_args.push(Value(arg));
        }
        Ok(card_context)
    }
}

impl<'a> CtxAdaptor<'a> for CardContext<'a> {
    fn context_type(&self) -> ContextType {
        ContextType::Card
    }

    fn offset(&self) -> usize {
        self.card.unique_id as usize
    }

    fn card(&mut self) -> Result<&mut CardContext<'a>, Error> {
        Ok(self)
    }
}
