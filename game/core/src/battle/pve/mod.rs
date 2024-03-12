extern crate alloc;
use alloc::collections::VecDeque;
use alloc::{vec, vec::Vec};

use crate::battle::traits::{FightLog, IterationInput, IterationOutput, Selection, SimplePVE};
use crate::contexts::{EnemyContext, SystemContext, WarriorContext};
use crate::errors::Error;
use crate::systems::{SystemController, SystemInput};
use crate::wrappings::{Enemy, ItemClass, RequireTarget};

mod control;
mod iteration;
mod log;

#[derive(Clone, Copy, PartialEq)]
enum FightView {
    Player,
    Enemy,
    Card(usize),
}

struct Instruction {
    target: Option<usize>,
    ctx: SystemContext,
    view: FightView,
    system_input: Option<SystemInput>,
}

pub struct MapBattlePVE<'a> {
    player: &'a mut WarriorContext,
    opponents: Vec<EnemyContext>,
    round: u8,
    fight_logs: Vec<FightLog>,
    last_output: IterationOutput,
    pending_instructions: VecDeque<Instruction>,
}

impl<'a> SimplePVE<'a> for MapBattlePVE<'a> {
    fn create(player: &'a mut WarriorContext, enemies: Vec<Enemy>) -> Result<Self, Error> {
        let opponents = enemies
            .into_iter()
            .enumerate()
            .map(|(i, enemy)| EnemyContext::new(enemy, i + 1))
            .collect();
        Ok(Self {
            player,
            opponents,
            round: 0,
            fight_logs: vec![],
            last_output: IterationOutput::Continue,
            pending_instructions: VecDeque::new(),
        })
    }

    fn start(
        &mut self,
        controller: &mut SystemController,
    ) -> Result<(IterationOutput, Vec<FightLog>), Error> {
        if self.round != 0 {
            return Err(Error::BattleRepeatStart);
        }
        let mut equipment_effects = vec![];
        self.player.warrior.package_status.iter().for_each(|v| {
            if v.class == ItemClass::Equipment {
                v.system_pool
                    .iter()
                    .for_each(|effect| equipment_effects.push(effect.clone()));
            }
        });
        self.trigger_log(FightLog::GameStart)?;
        self.trigger_log(FightLog::Snapshot(
            self.player.clone(),
            self.opponents.iter().map(|v| v.clone()).collect(),
        ))?;
        self.player
            .collect_card_systems()
            .into_iter()
            .map(|(offset, effects)| {
                self.trigger_iteration_systems(FightView::Card(offset), effects, None, controller)
            })
            .collect::<Result<Vec<_>, _>>()?;
        self.round = 1;
        self.trigger_log(FightLog::PlayerTurn(self.round))?;
        self.player_draw(self.player.draw_count, controller)?;
        let output =
            self.trigger_iteration_systems(FightView::Player, equipment_effects, None, controller)?;
        let logs = self.fight_logs.drain(..).collect();
        Ok((output, logs))
    }

    fn run(
        &mut self,
        operations: Vec<IterationInput>,
        controller: &mut SystemController,
    ) -> Result<(IterationOutput, Vec<FightLog>), Error> {
        if self.round == 0 {
            return Err(Error::BattleNotStarted);
        }
        for operation in operations {
            let output = self.iterate(operation, controller)?;
            if output == IterationOutput::GameWin || output == IterationOutput::GameLose {
                let logs = self.fight_logs.drain(..).collect();
                return Ok((self.last_output, logs));
            }
        }
        let logs = self.fight_logs.drain(..).collect();
        Ok((self.last_output, logs))
    }

    fn peak_target(&self, hand_card_selection: Selection) -> Result<bool, Error> {
        let Selection::SingleCard(index) = hand_card_selection else {
            return Err(Error::BattleOperationMismatch);
        };
        let required_targets = self
            .player
            .hand_deck
            .get(index)
            .ok_or(Error::BattleSelectionError)?
            .card
            .system_pool
            .iter()
            .map(|system| system.target_type)
            .collect::<Vec<_>>();
        let mut select_required = false;
        for target in &required_targets {
            if let RequireTarget::Opponent = target {
                if select_required {
                    return Err(Error::ResourceEffectMultiTargetInSystemPool);
                }
                select_required = true;
            }
        }
        Ok(select_required)
    }
}
