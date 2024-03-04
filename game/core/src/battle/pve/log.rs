extern crate alloc;
use alloc::vec::Vec;

use crate::battle::pve::{FightView, Instruction, MapBattlePVE};
use crate::battle::traits::FightLog;
use crate::errors::Error;
use crate::systems::SystemInput;
use crate::wrappings::System;

impl<'a> MapBattlePVE<'a> {
    pub(super) fn trigger_log(&mut self, log: FightLog) -> Result<(), Error> {
        self.trigger_mounting_systems(
            FightView::Player,
            self.player.mounting_systems.clone(),
            None,
            log.clone(),
        )?;
        self.opponents
            .iter()
            .map(|enemy| enemy.mounting_systems.clone())
            .collect::<Vec<_>>()
            .into_iter()
            .enumerate()
            .map(|(offset, effects)| {
                self.trigger_mounting_systems(FightView::Enemy, effects, Some(offset), log.clone())
            })
            .collect::<Result<Vec<_>, _>>()?;
        self.player
            .collect_card_systems(true)
            .into_iter()
            .map(|(offset, effects)| {
                self.trigger_mounting_systems(FightView::Card(offset), effects, None, log.clone())
            })
            .collect::<Result<Vec<_>, _>>()?;
        self.fight_logs.push(log);
        Ok(())
    }

    fn trigger_mounting_systems(
        &mut self,
        view: FightView,
        effects: Vec<System>,
        target: Option<usize>,
        log: FightLog,
    ) -> Result<(), Error> {
        effects
            .into_iter()
            .map(|system| {
                let system_input = SystemInput::Trigger(log.clone());
                self.pending_instructions.push_back(Instruction {
                    target,
                    view,
                    system,
                    system_input: Some(system_input),
                });
                Ok(())
            })
            .collect::<Result<_, _>>()?;
        Ok(())
    }
}
