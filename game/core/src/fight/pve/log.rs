extern crate alloc;
use alloc::vec::Vec;
use rand::RngCore;

use crate::errors::Error;
use crate::fight::pve::{FightView, Instruction, MapFightPVE};
use crate::fight::traits::FightLog;
use crate::systems::{GameSystem, SystemInput, SystemReturn};
use crate::wrappings::Effect;

impl<'a> MapFightPVE<'a> {
    pub(super) fn trigger_fight_log(
        &mut self,
        log: FightLog,
        system: &mut GameSystem<'a, impl RngCore>,
    ) -> Result<(), Error> {
        self.prepare_negative_effects(
            FightView::Player,
            &self.player.pending_effects.clone(),
            None,
            log.clone(),
            system,
        )?;
        self.opponents
            .iter()
            .map(|enemy| enemy.pending_effects.clone())
            .collect::<Vec<_>>()
            .into_iter()
            .enumerate()
            .map(|(offset, effects)| {
                self.prepare_negative_effects(
                    FightView::Enemy,
                    &effects,
                    Some(offset),
                    log.clone(),
                    system,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        self.fight_logs.push(log);
        Ok(())
    }

    fn prepare_negative_effects(
        &mut self,
        view: FightView,
        effects: &[&'a Effect],
        enemy_offset: Option<usize>,
        log: FightLog,
        system: &mut GameSystem<'a, impl RngCore>,
    ) -> Result<(), Error> {
        effects
            .iter()
            .map(|effect| {
                let Some(trigger) = &effect.on_trigger else {
                    return Err(Error::ResourceEffectNotNegative);
                };
                let mut system_contexts = self.collect_system_contexts(
                    view,
                    trigger.target_position,
                    enemy_offset,
                    system,
                )?;
                let system_input = SystemInput::FightLog(log.clone());
                let trigger_return = system.call(
                    trigger.system_id,
                    &trigger.args,
                    &mut system_contexts,
                    Some(system_input.clone()),
                )?;
                let context = match trigger_return {
                    SystemReturn::Triggered => effect
                        .on_execution
                        .as_ref()
                        .ok_or(Error::ResourceEffectSetupConflict)?,
                    SystemReturn::Discarded => effect
                        .on_discard
                        .as_ref()
                        .ok_or(Error::ResourceEffectSetupConflict)?,
                    _ => return Err(Error::BattleSystemInvalidReturn),
                };
                self.pending_instructions.push(Instruction::<'a> {
                    enemy_offset,
                    context,
                    view,
                    system_input: Some(system_input),
                });
                Ok(())
            })
            .collect::<Result<_, _>>()?;
        Ok(())
    }
}
