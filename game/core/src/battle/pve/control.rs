extern crate alloc;
use alloc::{vec, vec::Vec};
use rand::RngCore;

use crate::battle::pve::{FightView, Instruction, MapBattlePVE};
use crate::battle::traits::{FightLog, IterationOutput};
use crate::contexts::CtxAdaptor;
use crate::errors::Error;
use crate::systems::{SystemController, SystemInput, SystemReturn};
use crate::wrappings::RequireTarget;

impl<'a> MapBattlePVE<'a> {
    pub(super) fn player_draw(
        &mut self,
        draw_count: u8,
        controller: &mut SystemController<'a>,
    ) -> Result<(), Error> {
        if draw_count == 0 {
            return Err(Error::BattleUnexpectedDrawCount);
        }
        for _ in 0..draw_count {
            if self.player.deck.is_empty() {
                let mut grave_cards = self.player.grave_deck.drain(..).collect::<Vec<_>>();
                if grave_cards.is_empty() {
                    return Err(Error::BattleInternalError);
                }
                self.player.deck.append(&mut grave_cards);
                self.trigger_log(FightLog::RecoverGraveDeck)?;
            }
            let card_index = controller.rng.next_u32() as usize % self.player.deck.len();
            let card = self.player.deck.remove(card_index);
            self.player.hand_deck.push(card);
            self.trigger_log(FightLog::Draw(card_index))?;
        }
        Ok(())
    }

    pub(super) fn collect_system_contexts(
        &mut self,
        view: FightView,
        target: RequireTarget,
        offset: Option<usize>,
        controller: &mut SystemController<'a>,
    ) -> Result<Vec<&mut dyn CtxAdaptor<'a>>, Error> {
        let mut system_contexts: Vec<&mut dyn CtxAdaptor<'a>> = vec![];
        match (view, target) {
            (FightView::Player, RequireTarget::Opponent)
            | (FightView::Enemy, RequireTarget::Owner) => {
                let Some(offset) = offset else {
                    return Err(Error::BattleSelectionError);
                };
                let enemy = self
                    .opponents
                    .get_mut(offset)
                    .ok_or(Error::BattleEnemyNotFound)?;
                system_contexts.push(enemy);
            }
            (FightView::Player, RequireTarget::AllOpponents) => {
                self.opponents
                    .iter_mut()
                    .for_each(|v| system_contexts.push(v));
            }
            (FightView::Player, RequireTarget::AllCharactors)
            | (FightView::Enemy, RequireTarget::AllCharactors) => {
                self.opponents
                    .iter_mut()
                    .for_each(|v| system_contexts.push(v));
                system_contexts.push(self.player);
            }
            (FightView::Player, RequireTarget::RandomOpponent) => {
                let offset = controller.rng.next_u32() as usize % self.opponents.len();
                let enemy = self.opponents.get_mut(offset).unwrap();
                system_contexts.push(enemy);
            }
            (FightView::Player, RequireTarget::Owner)
            | (FightView::Enemy, RequireTarget::Opponent)
            | (FightView::Enemy, RequireTarget::RandomOpponent)
            | (FightView::Enemy, RequireTarget::AllOpponents) => system_contexts.push(self.player),
        };
        Ok(system_contexts)
    }

    pub(super) fn operate_pending_instructions(
        &mut self,
        controller: &mut SystemController<'a>,
    ) -> Result<IterationOutput, Error> {
        let mut game_over = false;
        self.last_output = IterationOutput::Continue;
        while !self.pending_instructions.is_empty() {
            let Instruction {
                view,
                system,
                offset,
                mut system_input,
            } = self.pending_instructions.remove(0);
            self.trigger_log(FightLog::CallSystemId(system.id.into()))?;
            let mut system_contexts =
                self.collect_system_contexts(view, system.target_type, offset, controller)?;
            if game_over {
                system_input = Some(SystemInput::GameOver);
            }
            let system_return =
                controller.system_call(&system, &mut system_contexts, system_input)?;
            if !game_over {
                if self.player.hp == 0 {
                    self.last_output = IterationOutput::GameLose;
                    self.trigger_log(FightLog::GameOver)?;
                    game_over = true;
                    continue;
                }
                if self.opponents.iter().all(|v| v.hp == 0) {
                    self.last_output = IterationOutput::GameWin;
                    self.trigger_log(FightLog::GameOver)?;
                    game_over = true;
                    continue;
                }
                match system_return {
                    SystemReturn::RequireCardSelect => {
                        if view != FightView::Player {
                            return Err(Error::ResourceEffectCardSelectInEnemy);
                        }
                        self.last_output = IterationOutput::RequireCardSelect;
                        break;
                    }
                    SystemReturn::DrawCard(draw_count) => {
                        self.player_draw(draw_count, controller)?
                    }
                    SystemReturn::SystemLog(mut logs) => self.fight_logs.append(&mut logs),
                }
            }
        }
        if game_over {
            self.player.reset();
        }
        Ok(self.last_output)
    }
}
