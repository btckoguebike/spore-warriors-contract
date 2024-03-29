extern crate alloc;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use spore_warriors_generated as generated;

use crate::battle::traits::FightLog;
use crate::contexts::{CtxAdaptor, SystemContext, WarriorDeckContext};
use crate::errors::Error;
use crate::game::SporeRng;
use crate::wrappings::SystemId;

#[cfg(feature = "json_serde")]
use serde::Serialize;

const MAX_WEAK_COUNT: u8 = 10;

mod applications;
mod instant;
mod triggered;

pub enum Command {
    AddLogs(Vec<FightLog>),
    DrawCards(u8),
    DiscardHandCards(u8, bool),
}

type DeckOperator = Box<dyn FnMut(&mut WarriorDeckContext) -> Vec<FightLog>>;

pub enum SystemReturn {
    RequireCardSelect(u8, bool, Option<DeckOperator>),
    RequireDeckChange(DeckOperator),
    Continue(Vec<Command>),
}

#[cfg_attr(feature = "json_serde", derive(Serialize))]
#[derive(Clone)]
pub enum SystemInput {
    Selection(Vec<usize>),
    Trigger(FightLog),
}

pub type SystemCallback = fn(
    &generated::ResourcePool,
    &mut SporeRng,
    SystemContext,
    usize,
    Vec<usize>,
    &mut [&mut dyn CtxAdaptor],
    Option<SystemInput>,
) -> Result<SystemReturn, Error>;

pub struct SystemController {
    pub resource_pool: generated::ResourcePool,
    pub rng: SporeRng,
    controller: BTreeMap<SystemId, SystemCallback>,
}

macro_rules! collect_systems {
    ($collector:ident $(, ($id:ident, $ns:ident::$sys:ident))+) => {
        $(
            $collector.insert(SystemId::$id, $ns::$sys as SystemCallback);
        )+
    };
}

#[macro_export]
macro_rules! apply_system {
    ($logs:ident, $iter:ident, $input:ident, $ctxs:ident, $field:ident, $ft:ty, $meth:ident, $log:ident) => {
        if let Some(SystemInput::Trigger(FightLog::GameOver)) = $input {
            return Ok(SystemReturn::Continue(vec![]));
        }
        let Some(Value(value)) = $iter.next() else {
            return Err(Error::BattleUnexpectedSystemArgs);
        };
        let value = *value as $ft;
        for object in $ctxs {
            match object.context_type() {
                ContextType::Warrior => {
                    let warrior = object.warrior()?;
                    warrior.$field = warrior.$field.$meth(value);
                }
                ContextType::Enemy => {
                    let enemy = object.enemy()?;
                    enemy.$field = enemy.$field.$meth(value);
                }
                ContextType::Card => continue,
            };
            $logs.push(FightLog::$log(object.offset(), value));
        }
    };
    ($logs:ident, $iter:ident, $input:ident, $ctxs:ident, $ft:ty, $app:tt) => {
        if let Some(SystemInput::Trigger(FightLog::GameOver)) = $input {
            let commands = if $logs.is_empty() {
                vec![]
            } else {
                vec![Command::AddLogs($logs)]
            };
            return Ok(SystemReturn::Continue(commands));
        }
        let Some(Value(value)) = $iter.next() else {
            return Err(Error::BattleUnexpectedSystemArgs);
        };
        for object in $ctxs {
            $app(&mut $logs, *value as $ft, object)?;
        }
    };
}

#[macro_export]
macro_rules! filter_objects {
    ($objs:ident, $filters:tt) => {
        $objs
            .into_iter()
            .filter(|v| $filters.contains(&v.offset()))
            .collect::<Vec<_>>()
    };
}

impl SystemController {
    pub fn new(resource_pool: generated::ResourcePool, rng: SporeRng) -> Self {
        let mut controller = BTreeMap::new();
        collect_systems!(
            controller,
            (InstantDamage, instant::attack),
            (InstantMultipleDamage, instant::multiple_attack),
            (InstantArmorUp, instant::armor_up),
            (InstantArmorDown, instant::armor_down),
            (InstantShieldUp, instant::shield_up),
            (InstantShieldDown, instant::shield_down),
            (InstantHealing, instant::healing),
            (InstantDrawCountUp, instant::draw_count_up),
            (InstantDrawCountDown, instant::draw_count_down),
            (InstantAttackUp, instant::attack_up),
            (InstantAttackDown, instant::attack_down),
            (InstantAttackWeakUp, instant::attack_weak_up),
            (InstantAttackWeakDown, instant::attack_weak_down),
            (InstantDefenseUp, instant::defense_up),
            (InstantDefenseDown, instant::defense_down),
            (InstantDefenseWeakUp, instant::defense_weak_up),
            (InstantDefenseWeakDown, instant::defense_weak_down),
            (InstantMaxHpUp, instant::max_hp_up),
            (InstantMaxHpDown, instant::max_hp_down),
            (InstantDrawCards, instant::draw_cards),
            (InstantDrawSelectCards, instant::draw_select_cards),
            (InstantDiscardCards, instant::discard_cards),
            (InstantDiscardSelectCards, instant::discard_select_cards),
            (InstantPowerCostDown, instant::card_power_cost_down),
            (InstantChangePowerCost, instant::change_random_card_cost),
            (TriggerRecoverHp, triggered::recover_hp),
            (TriggerRecoverHpAttack, triggered::recover_hp_to_attack),
            (TriggerAttackUp, triggered::attack_up),
            (TriggerAttackDown, triggered::attack_down)
        );
        Self {
            resource_pool,
            rng,
            controller,
        }
    }

    pub fn system_call(
        &mut self,
        ctx: SystemContext,
        caster: usize,
        targets: Vec<usize>,
        objects: &mut [&mut dyn CtxAdaptor],
        system_input: Option<SystemInput>,
    ) -> Result<SystemReturn, Error> {
        let system_trigger = self
            .controller
            .get(&ctx.system.system_id)
            .ok_or(Error::SystemTriggerMissing)?;
        system_trigger(
            &self.resource_pool,
            &mut self.rng,
            ctx,
            caster,
            targets,
            objects,
            system_input,
        )
    }
}
