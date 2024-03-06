extern crate alloc;
use alloc::{vec, vec::Vec};
use rand::RngCore;

use crate::battle::pve::{FightView, Instruction, MapBattlePVE};
use crate::battle::traits::{FightLog, IterationOutput};
use crate::contexts::CtxAdaptor;
use crate::errors::Error;
use crate::systems::{Command, SystemController, SystemInput, SystemReturn};
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

    pub(super) fn collect_system_caster_offset(
        &self,
        view: FightView,
        target: RequireTarget,
        target_offset: Option<usize>,
    ) -> Result<Option<usize>, Error> {
        match (view, target) {
            (FightView::Card(_), _) => Ok(None),
            (FightView::Player, _) => Ok(Some(self.player.offset())),
            (FightView::Enemy, _) => {
                let Some(offset) = target_offset else {
                    return Err(Error::BattleSelectionError);
                };
                Ok(Some(offset))
            }
        }
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
            ctx,
            target,
            mut system_input,
        }) = self.pending_instructions.pop_front()
        {
            let target_type = ctx.system.target_type;
            if let Some(caster) = self.collect_system_caster_offset(view, target_type, target)? {
                self.trigger_log(FightLog::CallSystem(caster, ctx.clone()))?;
            }
            let mut system_contexts =
                self.collect_system_contexts(view, target_type, target, controller)?;
            if game_over {
                system_input = Some(SystemInput::Trigger(FightLog::GameOver));
            }
            let system_return =
                controller.system_call(ctx, &mut system_contexts, system_input.clone())?;
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
                self.operate_system_return(system_return, view, target, system_input, controller)?;
                if self.last_output == IterationOutput::RequireCardSelect {
                    break;
                }
            }
        }
        if game_over {
            self.player.reset();
        }
        Ok(self.last_output)
    }

    fn operate_system_return(
        &mut self,
        system_return: SystemReturn,
        view: FightView,
        target: Option<usize>,
        system_input: Option<SystemInput>,
        controller: &mut SystemController,
    ) -> Result<(), Error> {
        let return_cmds;
        match system_return {
            SystemReturn::Continue(cmds) => return_cmds = cmds,
            SystemReturn::RequireCardSelect(cmds) => {
                if let FightView::Player = view {
                    return Err(Error::ResourceEffectCardSelectInEnemy);
                }
                self.last_output = IterationOutput::RequireCardSelect;
                return_cmds = cmds;
            }
            SystemReturn::PendingSystems(pending, cmds) => {
                let mut instructions = pending
                    .into_iter()
                    .map(|instant_system| Instruction {
                        view,
                        ctx: instant_system.into(),
                        target,
                        system_input: system_input.clone(),
                    })
                    .collect::<Vec<_>>();
                instructions.reverse();
                instructions.into_iter().for_each(|v| {
                    self.pending_instructions.push_front(v);
                });
                return_cmds = cmds;
            }
        };
        for cmd in return_cmds {
            match cmd {
                Command::AddLogs(mut logs) => self.fight_logs.append(&mut logs),
                Command::DrawCards(count) => self.player_draw(count, controller)?,
            }
        }
        Ok(())
    }
}
