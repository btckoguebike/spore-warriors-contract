extern crate alloc;
use alloc::vec::Vec;

use crate::battle::pve::{FightView, Instruction, MapBattlePVE};
use crate::battle::traits::FightLog;
use crate::contexts::{CtxAdaptor, SystemContext};
use crate::errors::Error;
use crate::systems::SystemInput;

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
            .map(|enemy| (enemy.offset(), enemy.mounting_systems.clone()))
            .collect::<Vec<_>>()
            .into_iter()
            .map(|(offset, contexts)| {
                self.trigger_mounting_systems(FightView::Enemy, contexts, Some(offset), log.clone())
            })
            .collect::<Result<Vec<_>, _>>()?;
        self.player
            .collect_card_mountings()
            .into_iter()
            .map(|(card_offset, system_offsets)| {
                self.trigger_mounting_systems(
                    FightView::Card(card_offset),
                    system_offsets,
                    None,
                    log.clone(),
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        self.fight_logs.push(log);
        Ok(())
    }

    fn trigger_mounting_systems(
        &mut self,
        view: FightView,
        contexts: Vec<SystemContext>,
        target: Option<usize>,
        log: FightLog,
    ) -> Result<(), Error> {
        contexts
            .into_iter()
            .map(|ctx| {
                let system_input = SystemInput::Trigger(log.clone());
                self.pending_instructions.push_back(Instruction {
                    target,
                    ctx,
                    view,
                    system_input: Some(system_input),
                });
                Ok(())
            })
            .collect::<Result<_, _>>()?;
        Ok(())
    }
}
