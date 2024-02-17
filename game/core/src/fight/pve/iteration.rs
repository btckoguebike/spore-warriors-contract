extern crate alloc;
use alloc::vec::Vec;
use rand::RngCore;

use crate::errors::Error;
use crate::fight::pve::{FightView, MapFightPVE};
use crate::fight::traits::{FightLog, IterationInput, IterationOutput, Selection};
use crate::systems::{GameSystem, SystemInput};

impl<'a> MapFightPVE<'a> {
    pub(super) fn iterate(
        &mut self,
        operation: IterationInput,
        system: &mut GameSystem<'a, impl RngCore>,
    ) -> Result<IterationOutput, Error> {
        match operation {
            IterationInput::ItemUse(Selection::Item(item_index), enemy_offset) => {
                self.iterate_item_use(item_index, enemy_offset, system)
            }
            IterationInput::HandCardUse(Selection::SingleCard(card_index), enemy_offset) => {
                self.iterate_hand_card_use(card_index, enemy_offset, system)
            }
            IterationInput::PendingCardSelect(Selection::MultiCards(card_indexes)) => {
                self.iterate_pending_card_select(card_indexes, system)
            }
            IterationInput::SpecialCardUse(enemy_offset) => {
                self.iterate_special_card_use(enemy_offset, system)
            }
            IterationInput::EnemyTurn => self.iterate_enemy_turn(system),
            _ => Err(Error::BattleSelectionMismatch),
        }
    }

    fn iterate_item_use(
        &mut self,
        item_index: usize,
        enemy_offset: Option<usize>,
        system: &mut GameSystem<'a, impl RngCore>,
    ) -> Result<IterationOutput, Error> {
        if item_index >= self.player.props_list.len() {
            return Err(Error::BattleSelectionError);
        }
        if IterationOutput::Continue != self.last_output {
            return Err(Error::BattleUnexpectedOutput);
        }
        let props_item = self.player.props_list.remove(item_index);
        self.trigger_fight_log(FightLog::ItemUse(item_index), system)?;
        let effects = props_item.effect_pool.iter().collect::<Vec<_>>();
        self.operate_positive_effects(FightView::Player, &effects, enemy_offset, system)
    }

    fn iterate_hand_card_use(
        &mut self,
        card_index: usize,
        enemy_offset: Option<usize>,
        system: &mut GameSystem<'a, impl RngCore>,
    ) -> Result<IterationOutput, Error> {
        if card_index >= self.player.hand_deck.len() {
            return Err(Error::BattleSelectionError);
        }
        if IterationOutput::Continue != self.last_output {
            return Err(Error::BattleUnexpectedOutput);
        };
        let card = self.player.hand_deck.remove(card_index);
        if self.player.power < card.power_cost {
            return Err(Error::BattlePowerInsufficient);
        }
        self.player.grave_deck.push(card);
        self.trigger_fight_log(FightLog::PowerCost(card.power_cost), system)?;
        self.trigger_fight_log(FightLog::HandCardUse(card_index), system)?;
        let effects = card.effect_pool.iter().collect::<Vec<_>>();
        self.operate_positive_effects(FightView::Player, &effects, enemy_offset, system)
    }

    fn iterate_special_card_use(
        &mut self,
        enemy_offset: Option<usize>,
        system: &mut GameSystem<'a, impl RngCore>,
    ) -> Result<IterationOutput, Error> {
        if IterationOutput::Continue != self.last_output {
            return Err(Error::BattleUnexpectedOutput);
        };
        let cost = self.player.special_card.power_cost;
        if self.player.power < cost {
            return Err(Error::BattlePowerInsufficient);
        }
        self.player.power -= cost;
        self.trigger_fight_log(FightLog::PowerCost(cost), system)?;
        self.trigger_fight_log(FightLog::SpecialCardUse, system)?;
        let effects = self
            .player
            .special_card
            .effect_pool
            .iter()
            .collect::<Vec<_>>();
        self.operate_positive_effects(FightView::Player, &effects, enemy_offset, system)
    }

    fn iterate_pending_card_select(
        &mut self,
        card_indexes: Vec<usize>,
        system: &mut GameSystem<'a, impl RngCore>,
    ) -> Result<IterationOutput, Error> {
        if card_indexes
            .iter()
            .any(|v| *v >= self.player.pending_deck.len())
        {
            return Err(Error::BattleSelectionError);
        }
        if IterationOutput::RequireCardSelect != self.last_output {
            return Err(Error::BattleUnexpectedOutput);
        }
        if let Some(value) = self.pending_instructions.first_mut() {
            value.system_input = Some(SystemInput::CardSelect(card_indexes));
        } else {
            return Err(Error::BattleInstructionEmpty);
        }
        self.operate_pending_instructions(system)
    }

    fn iterate_enemy_turn(
        &mut self,
        system: &mut GameSystem<'a, impl RngCore>,
    ) -> Result<IterationOutput, Error> {
        if !self.pending_instructions.is_empty() {
            return Err(Error::BattleInstructionNotEmpty);
        }
        let mut remained_hand_cards = self.player.hand_deck.drain(..).collect::<Vec<_>>();
        self.player.grave_deck.append(&mut remained_hand_cards);
        self.trigger_fight_log(FightLog::DiscardHandDeck, system)?;
        self.trigger_fight_log(FightLog::EnemyTurn(self.round), system)?;

        let actions = self
            .opponents
            .iter_mut()
            .map(|enemy| enemy.pop_action(system.rng()))
            .collect::<Result<Vec<_>, _>>()?;
        for (offset, effects) in actions.into_iter().enumerate() {
            let output =
                self.operate_positive_effects(FightView::Enemy, &effects, Some(offset), system)?;
            if IterationOutput::GameWin == output || IterationOutput::GameLose == output {
                return Ok(output);
            }
        }
        if !self.pending_instructions.is_empty() {
            return Err(Error::BattleInstructionNotEmpty);
        }
        self.round += 1;
        self.player.power = self.player.warrior.power;
        self.trigger_fight_log(FightLog::PlayerTurn(self.round), system)?;
        self.trigger_fight_log(FightLog::RecoverPower, system)?;
        self.player_draw(self.player.draw_count, system)?;

        let output = self.operate_pending_instructions(system)?;
        if IterationOutput::Continue == output {
            Ok(IterationOutput::PlayerTurn)
        } else {
            Ok(output)
        }
    }
}
