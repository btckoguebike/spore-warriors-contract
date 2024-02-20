extern crate alloc;
use alloc::vec::Vec;
use rand::RngCore;

use crate::errors::Error;
use crate::fight::pve::{FightView, MapFightPVE};
use crate::fight::traits::{FightLog, IterationInput, IterationOutput, Selection};
use crate::systems::{SystemController, SystemInput};
use crate::wrappings::System;

use super::Instruction;

impl<'a> MapFightPVE<'a> {
    pub(super) fn iterate(
        &mut self,
        operation: IterationInput,
        controller: &mut SystemController<'a, impl RngCore>,
    ) -> Result<IterationOutput, Error> {
        match operation {
            IterationInput::ItemUse(Selection::Item(item_index), offset) => {
                self.iterate_item_use(item_index, offset, controller)
            }
            IterationInput::HandCardUse(Selection::SingleCard(card_index), enemy_offset) => {
                self.iterate_hand_card_use(card_index, enemy_offset, controller)
            }
            IterationInput::PendingCardSelect(Selection::MultiCards(card_indexes)) => {
                self.iterate_pending_card_select(card_indexes, controller)
            }
            IterationInput::SpecialCardUse(offset) => {
                self.iterate_special_card_use(offset, controller)
            }
            IterationInput::EnemyTurn => self.iterate_enemy_turn(controller),
            _ => Err(Error::BattleSelectionMismatch),
        }
    }

    pub(super) fn trigger_iteration_systems(
        &mut self,
        view: FightView,
        effects: &[&'a System],
        offset: Option<usize>,
        controller: &mut SystemController<'a, impl RngCore>,
    ) -> Result<IterationOutput, Error> {
        effects
            .iter()
            .map(|system| {
                self.pending_instructions.push(Instruction::<'a> {
                    view,
                    offset,
                    system,
                    system_input: None,
                });
                Ok(())
            })
            .collect::<Result<_, _>>()?;
        self.operate_pending_instructions(controller)
    }

    fn iterate_item_use(
        &mut self,
        item_index: usize,
        offset: Option<usize>,
        controller: &mut SystemController<'a, impl RngCore>,
    ) -> Result<IterationOutput, Error> {
        if item_index >= self.player.props_list.len() {
            return Err(Error::BattleSelectionError);
        }
        if IterationOutput::Continue != self.last_output {
            return Err(Error::BattleUnexpectedOutput);
        }
        let props_item = self.player.props_list.remove(item_index);
        self.trigger_log(FightLog::ItemUse(item_index))?;
        let effects = props_item.system_pool.iter().collect::<Vec<_>>();
        self.trigger_iteration_systems(FightView::Player, &effects, offset, controller)
    }

    fn iterate_hand_card_use(
        &mut self,
        card_index: usize,
        offset: Option<usize>,
        controller: &mut SystemController<'a, impl RngCore>,
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
        self.trigger_log(FightLog::PowerCost(card.power_cost))?;
        self.trigger_log(FightLog::HandCardUse(card_index))?;
        let effects = card.card.system_pool.iter().collect::<Vec<_>>();
        self.player.grave_deck.push(card);
        self.trigger_iteration_systems(FightView::Player, &effects, offset, controller)
    }

    fn iterate_special_card_use(
        &mut self,
        offset: Option<usize>,
        controller: &mut SystemController<'a, impl RngCore>,
    ) -> Result<IterationOutput, Error> {
        if IterationOutput::Continue != self.last_output {
            return Err(Error::BattleUnexpectedOutput);
        };
        let cost = self.player.special_card.power_cost;
        if self.player.power < cost {
            return Err(Error::BattlePowerInsufficient);
        }
        self.player.power -= cost;
        self.trigger_log(FightLog::PowerCost(cost))?;
        self.trigger_log(FightLog::SpecialCardUse)?;
        let effects = self
            .player
            .special_card
            .card
            .system_pool
            .iter()
            .collect::<Vec<_>>();
        self.trigger_iteration_systems(FightView::Player, &effects, offset, controller)
    }

    fn iterate_pending_card_select(
        &mut self,
        card_indexes: Vec<usize>,
        controller: &mut SystemController<'a, impl RngCore>,
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
            value.system_input = Some(SystemInput::Selection(card_indexes));
        } else {
            return Err(Error::BattleInstructionEmpty);
        }
        self.operate_pending_instructions(controller)
    }

    fn iterate_enemy_turn(
        &mut self,
        controller: &mut SystemController<'a, impl RngCore>,
    ) -> Result<IterationOutput, Error> {
        if !self.pending_instructions.is_empty() {
            return Err(Error::BattleInstructionNotEmpty);
        }
        let mut remained_hand_cards = self.player.hand_deck.drain(..).collect::<Vec<_>>();
        self.player.grave_deck.append(&mut remained_hand_cards);
        self.trigger_log(FightLog::DiscardHandDeck)?;
        self.trigger_log(FightLog::EnemyTurn(self.round))?;

        let actions = self
            .opponents
            .iter_mut()
            .map(|enemy| enemy.pop_action(controller.rng()))
            .collect::<Result<Vec<_>, _>>()?;
        for (offset, effects) in actions.into_iter().enumerate() {
            let output = self.trigger_iteration_systems(
                FightView::Enemy,
                &effects,
                Some(offset),
                controller,
            )?;
            if IterationOutput::GameWin == output || IterationOutput::GameLose == output {
                return Ok(output);
            }
        }
        if !self.pending_instructions.is_empty() {
            return Err(Error::BattleInstructionNotEmpty);
        }
        self.round += 1;
        self.player.power = self.player.warrior.power;
        self.trigger_log(FightLog::PlayerTurn(self.round))?;
        self.trigger_log(FightLog::RecoverPower)?;
        self.player_draw(self.player.draw_count, controller)?;

        let output = self.operate_pending_instructions(controller)?;
        if IterationOutput::Continue == output {
            Ok(IterationOutput::PlayerTurn)
        } else {
            Ok(output)
        }
    }
}
