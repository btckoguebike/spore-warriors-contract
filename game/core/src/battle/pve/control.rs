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
        controller: &mut SystemController,
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
        target_offset: Option<usize>,
        controller: &mut SystemController,
    ) -> Result<Vec<&mut dyn CtxAdaptor>, Error> {
        let mut system_contexts: Vec<&mut dyn CtxAdaptor> = vec![];
        match (view, target) {
            (FightView::Card(offset), _) => {
                let card = self
                    .player
                    .refer_card(offset)
                    .ok_or(Error::BattleInternalError)?;
                system_contexts.push(card);
            }
            (FightView::Player, RequireTarget::Opponent)
            | (FightView::Enemy, RequireTarget::Owner) => {
                let Some(offset) = target_offset else {
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
        controller: &mut SystemController,
    ) -> Result<IterationOutput, Error> {
        let mut game_over = false;
        self.last_output = IterationOutput::Continue;
        while let Some(Instruction {
            view,
            system,
            target,
            mut system_input,
        }) = self.pending_instructions.pop_front()
        {
            self.trigger_log(FightLog::CallSystemId(system.id.into()))?;
            let mut system_contexts =
                self.collect_system_contexts(view, system.target_type, target, controller)?;
            if game_over {
                system_input = Some(SystemInput::GameOver);
            }
            let system_return =
                controller.system_call(&system, &mut system_contexts, system_input.clone())?;
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
                        if let FightView::Player = view {
                            return Err(Error::ResourceEffectCardSelectInEnemy);
                        }
                        self.last_output = IterationOutput::RequireCardSelect;
                        break;
                    }
                    SystemReturn::DrawCard(draw_count) => {
                        self.player_draw(draw_count, controller)?
                    }
                    SystemReturn::SystemLog(mut logs) => self.fight_logs.append(&mut logs),
                    SystemReturn::PendingSystems(pending, mut logs) => {
                        let mut instructions = pending
                            .into_iter()
                            .map(|system| Instruction {
                                view,
                                system,
                                target,
                                system_input: system_input.clone(),
                            })
                            .collect::<Vec<_>>();
                        instructions.reverse();
                        instructions.into_iter().for_each(|v| {
                            self.pending_instructions.push_front(v);
                        });
                        self.fight_logs.append(&mut logs);
                    }
                }
            }
        }
        if game_over {
            self.player.reset();
        }
        Ok(self.last_output)
    }
}
