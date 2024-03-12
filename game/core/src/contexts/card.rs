extern crate alloc;
use alloc::{vec, vec::Vec};
use rlp::{RlpDecodable, RlpEncodable};

use crate::contexts::{
    add_mounting_system_internal, remove_mounting_system_internal, ContextType, CtxAdaptor,
    SystemContext,
};
use crate::errors::Error;
use crate::wrappings::Card;

use super::update_mounting_system_internal;

#[cfg_attr(feature = "debug", derive(Debug, PartialEq))]
#[derive(Clone, RlpEncodable, RlpDecodable)]
pub struct CardContext {
    pub card: Card,
    pub power_cost: u8,
    pub battle_max_power_cost: u8,
    pub game_max_power_cost: u8,
    pub mounting_systems: Vec<SystemContext>,
}

impl CardContext {
    pub fn new(card: Card) -> Self {
        Self {
            power_cost: card.power_cost,
            battle_max_power_cost: card.power_cost,
            game_max_power_cost: card.power_cost,
            mounting_systems: vec![],
            card,
        }
    }

    pub fn round_reset(&mut self) {
        self.power_cost = self.battle_max_power_cost;
    }

    pub fn reset(&mut self) {
        self.power_cost = self.game_max_power_cost;
    }
}

impl CtxAdaptor for CardContext {
    fn context_type(&self) -> ContextType {
        ContextType::Card
    }

    fn offset(&self) -> usize {
        self.card.offset
    }

    fn add_mounting_system(&mut self, ctx: &SystemContext) -> bool {
        add_mounting_system_internal(ctx, &mut self.mounting_systems)
    }

    fn update_mounting_system(&mut self, ctx: &SystemContext) -> bool {
        update_mounting_system_internal(ctx, &mut self.mounting_systems)
    }

    fn remove_mounting_system(&mut self, ctx: &SystemContext) -> bool {
        remove_mounting_system_internal(ctx, &mut self.mounting_systems)
    }

    fn card(&mut self) -> Result<&mut CardContext, Error> {
        Ok(self)
    }
}
