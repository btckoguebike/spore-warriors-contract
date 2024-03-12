extern crate alloc;
use alloc::vec::Vec;

use crate::battle::pve::{FightView, Instruction, MapBattlePVE};
use crate::battle::traits::{FightLog, IterationInput, IterationOutput, Selection};
use crate::contexts::CtxAdaptor;
use crate::errors::Error;
use crate::systems::SystemController;
use crate::wrappings::System;

impl<'a> MapBattlePVE<'a> {
    pub(super) fn iterate(
        &mut self,
        operation: IterationInput,
        controller: &mut SystemController,
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
        systems: Vec<System>,
        target: Option<usize>,
        controller: &mut SystemController,
    ) -> Result<IterationOutput, Error> {
        systems
            .into_iter()
            .map(|system| {
                self.pending_instructions.push_back(Instruction {
                    view,
                    ctx: system.into(),
                    target,
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
        target: Option<usize>,
        controller: &mut SystemController,
    ) -> Result<IterationOutput, Error> {
        if item_index >= self.player.props_list.len() {
            return Err(Error::BattleSelectionError);
        }
        if IterationOutput::Continue != self.last_output {
            return Err(Error::BattleUnexpectedOutput);
        }
        let props_item = self.player.props_list.remove(item_index);
        self.trigger_log(FightLog::ItemUse(item_index))?;
        self.trigger_iteration_systems(
            FightView::Player,
            props_item.system_pool.clone(),
            target,
            controller,
        )
    }

    fn iterate_hand_card_use(
        &mut self,
        card_index: usize,
        target: Option<usize>,
        controller: &mut SystemController,
    ) -> Result<IterationOutput, Error> {
        if card_index >= self.player_deck.hand_deck.len() {
            return Err(Error::BattleSelectionError);
        }
        if IterationOutput::Continue != self.last_output {
            return Err(Error::BattleUnexpectedOutput);
        };
        let context = self.player_deck.hand_deck.remove(card_index);
        if self.player.power < context.power_cost {
            return Err(Error::BattlePowerInsufficient);
        }
        let systems = context.card.system_pool.clone();
        self.trigger_log(FightLog::PowerCost(context.power_cost))?;
        self.trigger_log(FightLog::HandCardUse(context.offset()))?;
        self.player_deck.grave_deck.push(context);
        self.trigger_iteration_systems(FightView::Player, systems, target, controller)
    }

    fn iterate_special_card_use(
        &mut self,
        target: Option<usize>,
        controller: &mut SystemController,
    ) -> Result<IterationOutput, Error> {
        if IterationOutput::Continue != self.last_output {
            return Err(Error::BattleUnexpectedOutput);
        };
        let cost = self.player_deck.special_card.power_cost;
        if self.player.power < cost {
            return Err(Error::BattlePowerInsufficient);
        }
        self.player.power -= cost;
        self.trigger_log(FightLog::PowerCost(cost))?;
        self.trigger_log(FightLog::SpecialCardUse)?;
        self.trigger_iteration_systems(
            FightView::Player,
            self.player_deck.special_card.card.system_pool.clone(),
            target,
            controller,
        )
    }

    fn iterate_pending_card_select(
        &mut self,
        card_offsets: Vec<usize>,
        controller: &mut SystemController,
    ) -> Result<IterationOutput, Error> {
        if card_offsets
            .iter()
            .any(|v| *v >= self.player_deck.selection_pool.len())
        {
            return Err(Error::BattleSelectionError);
        }
        let IterationOutput::RequireCardSelect(count, draw) = self.last_output else {
            return Err(Error::BattleUnexpectedOutput);
        };
        if card_offsets.len() > count as usize {
            return Err(Error::BattleExceedCardSelection);
        }
        if draw {
            self.player_select_draw(card_offsets)?;
        } else {
            self.player_select_discard(card_offsets, true)?;
        }
        self.operate_pending_instructions(controller)
    }

    fn iterate_enemy_turn(
        &mut self,
        controller: &mut SystemController,
    ) -> Result<IterationOutput, Error> {
        if !self.pending_instructions.is_empty() {
            return Err(Error::BattleInstructionNotEmpty);
        }
        let mut remained_hand_cards = self.player_deck.hand_deck.drain(..).collect::<Vec<_>>();
        self.player_deck.grave_deck.append(&mut remained_hand_cards);
        self.trigger_log(FightLog::DiscardAllHandDeck)?;
        self.trigger_log(FightLog::EnemyTurn(self.round))?;

        let actions = self
            .opponents
            .iter_mut()
            .map(|enemy| enemy.pop_action(&mut controller.rng))
            .collect::<Result<Vec<_>, _>>()?;
        for (offset, effects) in actions.into_iter().enumerate() {
            let output = self.trigger_iteration_systems(
                FightView::Enemy,
                effects,
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
        self.player.round_reset();
        self.trigger_log(FightLog::RecoverPower)?;
        self.trigger_log(FightLog::RecoverCardCost)?;
        self.trigger_log(FightLog::PlayerTurn(self.round))?;
        self.player_draw(self.player.draw_count, controller)?;

        #[cfg(feature = "debug")]
        self.trigger_log(FightLog::Snapshot(
            self.player.clone(),
            self.opponents.iter().map(|v| v.clone()).collect(),
        ))?;

        let output = self.operate_pending_instructions(controller)?;
        if IterationOutput::Continue == output {
            Ok(IterationOutput::PlayerTurn)
        } else {
            Ok(output)
        }
    }
}
