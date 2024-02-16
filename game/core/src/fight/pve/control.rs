extern crate alloc;
use alloc::{vec, vec::Vec};
use rand::RngCore;

use crate::contexts::CtxAdaptor;
use crate::errors::Error;
use crate::fight::pve::{FightView, Instruction, MapFightPVE};
use crate::fight::traits::{FightLog, IterationOutput};
use crate::systems::{GameSystem, SystemId, SystemReturn};
use crate::wrappings::{Effect, RequireTarget};

impl<'a> MapFightPVE<'a> {
    pub(super) fn player_draw(
        &mut self,
        draw_count: u8,
        system: &mut GameSystem<'a, impl RngCore>,
    ) -> Result<(), Error> {
        for _ in 0..draw_count {
            if self.player.deck.is_empty() {
                let mut grave_cards = self.player.grave_deck.drain(..).collect::<Vec<_>>();
                if grave_cards.is_empty() {
                    return Err(Error::SystemError);
                }
                self.player.deck.append(&mut grave_cards);
            }
            let card_index = system.rng().next_u32() as usize % self.player.deck.len();
            let card = self.player.deck.remove(card_index);
            self.player.hand_deck.push(card);
        }
        self.trigger_fight_log(FightLog::Draw(draw_count), system)
    }

    pub(super) fn collect_pending_effects(
        &mut self,
        view: FightView,
        target: RequireTarget,
        enemy_offset: Option<usize>,
        system: &mut GameSystem<'a, impl RngCore>,
    ) -> Result<Vec<&mut Vec<&'a Effect>>, Error> {
        let pending_queue = match (view, target) {
            (FightView::Player, RequireTarget::Owner)
            | (FightView::Enemy, RequireTarget::Opponent)
            | (FightView::Enemy, RequireTarget::AllOpponents)
            | (FightView::Enemy, RequireTarget::RandomOpponent) => {
                vec![&mut self.player.pending_effects]
            }
            (FightView::Player, RequireTarget::Opponent)
            | (FightView::Enemy, RequireTarget::Owner) => {
                let Some(offset) = enemy_offset else {
                    return Err(Error::BattleUserSelectionMissing);
                };
                let enemy = self
                    .opponents
                    .get_mut(offset)
                    .ok_or(Error::BattleSelectionError)?;
                vec![&mut enemy.pending_effects]
            }
            (FightView::Player, RequireTarget::AllOpponents) => self
                .opponents
                .iter_mut()
                .map(|enemy| &mut enemy.pending_effects)
                .collect(),
            (FightView::Player, RequireTarget::AllCharactors)
            | (FightView::Enemy, RequireTarget::AllCharactors) => {
                let mut queue = self
                    .opponents
                    .iter_mut()
                    .map(|enemy| &mut enemy.pending_effects)
                    .collect::<Vec<_>>();
                queue.append(&mut vec![&mut self.player.pending_effects]);
                queue
            }
            (FightView::Player, RequireTarget::RandomOpponent) => {
                let offset = system.rng().next_u32() as usize % self.opponents.len();
                let enemy = self.opponents.get_mut(offset).unwrap();
                vec![&mut enemy.pending_effects]
            }
        };
        Ok(pending_queue)
    }

    pub(super) fn collect_system_contexts(
        &mut self,
        view: FightView,
        target: RequireTarget,
        enemy_offset: Option<usize>,
        system: &mut GameSystem<'a, impl RngCore>,
    ) -> Result<Vec<&mut dyn CtxAdaptor>, Error> {
        let mut system_contexts: Vec<&mut dyn CtxAdaptor> = vec![];
        match (view, target) {
            (FightView::Player, RequireTarget::Opponent)
            | (FightView::Enemy, RequireTarget::Owner) => {
                let Some(offset) = enemy_offset else {
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
                system_contexts.push(&mut self.player);
            }
            (FightView::Player, RequireTarget::RandomOpponent) => {
                let offset = system.rng().next_u32() as usize % self.opponents.len();
                let enemy = self.opponents.get_mut(offset).unwrap();
                system_contexts.push(enemy);
            }
            (FightView::Player, RequireTarget::Owner)
            | (FightView::Enemy, RequireTarget::Opponent)
            | (FightView::Enemy, RequireTarget::RandomOpponent)
            | (FightView::Enemy, RequireTarget::AllOpponents) => {
                system_contexts.push(&mut self.player)
            }
        };
        Ok(system_contexts)
    }

    pub(super) fn operate_positive_effects(
        &mut self,
        view: FightView,
        effects: &[&'a Effect],
        enemy_offset: Option<usize>,
        system: &mut GameSystem<'a, impl RngCore>,
    ) -> Result<IterationOutput, Error> {
        effects
            .iter()
            .map(|effect| {
                if let Some(ref trigger) = effect.on_trigger {
                    let mut pending_effects = self.collect_pending_effects(
                        view,
                        trigger.target_position,
                        enemy_offset,
                        system,
                    )?;
                    pending_effects
                        .iter_mut()
                        .for_each(|queue| queue.push(effect));
                } else {
                    self.pending_instructions.push(Instruction::<'a> {
                        view,
                        enemy_offset,
                        context: effect.on_execution.as_ref().unwrap(),
                        system_input: None,
                    });
                };
                Ok(())
            })
            .collect::<Result<_, _>>()?;
        self.operate_pending_instructions(system)
    }

    pub(super) fn operate_pending_instructions(
        &mut self,
        system: &mut GameSystem<'a, impl RngCore>,
    ) -> Result<IterationOutput, Error> {
        self.last_output = IterationOutput::Continue;
        while let Some(value) = self.pending_instructions.first().cloned() {
            let system_contexts = self.collect_system_contexts(
                value.view,
                value.context.target_position,
                value.enemy_offset,
                system,
            )?;
            let system_return = system.call(
                SystemId::NormalDamage, // TODO: replace with true system ids
                &value.context.args,
                &system_contexts,
                value.system_input,
            )?;
            if self.player.hp == 0 {
                self.last_output = IterationOutput::GameLose;
                break;
            }
            if self.opponents.iter().all(|v| v.hp == 0) {
                self.last_output = IterationOutput::GameWin;
                break;
            }
            match system_return {
                SystemReturn::NeedCardSelect => {
                    if value.view != FightView::Player {
                        return Err(Error::ResourceEffectCardSelectInEnemy);
                    }
                    self.last_output = IterationOutput::RequireCardSelect;
                    break;
                }
                SystemReturn::DrawCard(draw_count) => self.player_draw(draw_count, system)?,
                SystemReturn::FightLog(mut logs) => self.fight_logs.append(&mut logs),
                SystemReturn::Null => {}
                _ => return Err(Error::BattleUnexpectedSystemReturn),
            }
            self.pending_instructions.remove(0);
        }
        Ok(self.last_output)
    }
}
