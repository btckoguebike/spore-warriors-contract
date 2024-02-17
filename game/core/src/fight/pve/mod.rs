extern crate alloc;
use alloc::{vec, vec::Vec};
use rand::RngCore;

use crate::contexts::{EnemyContext, WarriorContext};
use crate::errors::Error;
use crate::fight::traits::{FightLog, IterationInput, IterationOutput, Selection, SimplePVE};
use crate::systems::{GameSystem, SystemInput};
use crate::wrappings::{Context, Enemy, ItemClass, Potion, RequireTarget, Warrior};

mod control;
mod iteration;
mod log;

#[derive(Clone, Copy, PartialEq)]
enum FightView {
    Player,
    Enemy,
}

#[derive(Clone)]
struct Instruction<'a> {
    effect_id: u16,
    enemy_offset: Option<usize>,
    context: &'a Context,
    view: FightView,
    system_input: Option<SystemInput>,
}

pub struct MapFightPVE<'a> {
    player: WarriorContext<'a>,
    opponents: Vec<EnemyContext<'a>>,
    round: u8,
    fight_logs: Vec<FightLog>,
    last_output: IterationOutput,
    pending_instructions: Vec<Instruction<'a>>,
}

impl<'a> SimplePVE<'a> for MapFightPVE<'a> {
    fn create(
        player: &'a Warrior,
        potion: Option<&'a Potion>,
        enemies: &'a [Enemy],
    ) -> Result<Self, Error> {
        let mut player = WarriorContext::new(player);
        if let Some(potion) = potion {
            player.hp += potion.hp as u16;
            player.power += potion.power;
            player.armor += potion.armor;
            player.shield += potion.shield;
            player.attack += potion.attack;
            player.draw_count += potion.draw_count;
            player
                .props_list
                .append(&mut potion.package_status.iter().collect());
            player.deck.append(&mut potion.deck_status.iter().collect());
        }
        let opponents = enemies
            .iter()
            .enumerate()
            .map(|(i, enemy)| EnemyContext::new(enemy, i + 1))
            .collect();
        Ok(Self {
            player,
            opponents,
            round: 0,
            fight_logs: vec![],
            last_output: IterationOutput::Continue,
            pending_instructions: vec![],
        })
    }

    fn start(
        &mut self,
        system: &mut GameSystem<'a, impl RngCore>,
    ) -> Result<(IterationOutput, Vec<FightLog>), Error> {
        if self.round != 0 {
            return Err(Error::BattleRepeatStart);
        }
        let mut equipment_effects = vec![];
        self.player.warrior.package_status.iter().for_each(|v| {
            if v.class == ItemClass::Equipment {
                v.effect_pool
                    .iter()
                    .for_each(|effect| equipment_effects.push(effect));
            }
        });
        self.trigger_fight_log(
            FightLog::CharactorSet(
                self.player.snapshot(),
                self.opponents.iter().map(|v| v.snapshot()).collect(),
            ),
            system,
        )?;
        self.round = 1;
        self.trigger_fight_log(FightLog::PlayerTurn(self.round), system)?;
        self.player_draw(self.player.draw_count, system)?;
        let output =
            self.operate_positive_effects(FightView::Player, &equipment_effects, None, system)?;
        let logs = self.fight_logs.drain(..).collect();
        Ok((output, logs))
    }

    fn run(
        &mut self,
        operations: Vec<IterationInput>,
        system: &mut GameSystem<'a, impl RngCore>,
    ) -> Result<(IterationOutput, Vec<FightLog>), Error> {
        if self.round == 0 {
            return Err(Error::BattleNotStarted);
        }
        for operation in operations {
            self.iterate(operation, system)?;
        }
        #[cfg(feature = "debug")]
        self.trigger_fight_log(
            FightLog::CharactorSet(
                self.player.snapshot(),
                self.opponents.iter().map(|v| v.snapshot()).collect(),
            ),
            system,
        )?;
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
            .effect_pool
            .iter()
            .filter_map(|effect| effect.on_execution.as_ref())
            .map(|ctx| &ctx.target_position)
            .collect::<Vec<_>>();
        let mut select_required = false;
        for target in &required_targets {
            if let RequireTarget::Opponent = target {
                if select_required {
                    return Err(Error::ResourceEffectMultiTargetInEffectPool);
                }
                select_required = true;
            }
        }
        Ok(select_required)
    }
}
