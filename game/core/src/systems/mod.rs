extern crate alloc;
use alloc::collections::BTreeMap;
use spore_warriors_generated as generated;

use crate::battle::traits::FightLog;
use crate::contexts::{CtxAdaptor, SystemContext};
use crate::errors::Error;
use crate::game::SporeRng;
use crate::wrappings::{System, SystemId};

const MAX_WEAK_COUNT: u8 = 10;

mod applications;
mod instant;
mod triggered;

pub enum Command {
    AddLogs(Vec<FightLog>),
    DrawCards(u8),
}

pub enum SystemReturn {
    RequireCardSelect(Vec<Command>),
    Continue(Vec<Command>),
    PendingSystems(Vec<System>, Vec<Command>),
}

#[derive(Clone)]
pub enum SystemInput {
    Selection(Vec<usize>),
    Trigger(FightLog),
}

pub type SystemCallback = fn(
    &generated::ResourcePool,
    &mut SporeRng,
    SystemContext,
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
    ($iter:ident, $input:ident, $ctxs:ident, $field:ident, $ft:ty, $meth:ident, $log:ident) => {{
        let mut logs = vec![];
        if let Some(SystemInput::Trigger(FightLog::GameOver)) = $input {
            return Ok(SystemReturn::Continue(vec![]));
        }
        let Some(Value(value)) = $iter.next() else {
            return Err(Error::BattleUnexpectedSystemArgs);
        };
        let value = *value as $ft;
        for object in $ctxs.iter_mut() {
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
            logs.push(FightLog::$log(object.offset(), value));
        }
        logs
    }};
    ($logs:ident, $iter:ident, $input:ident, $ctxs:ident, $ft:ty, $app:tt) => {{
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
        for object in $ctxs.iter_mut() {
            $app(&mut $logs, *value as $ft, object)?;
        }
    }};
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
            (InstantAttackPowerUp, instant::attack_power_up),
            (InstantAttackPowerWeak, instant::attack_power_weak),
            (InstantDefensePowerUp, instant::defense_power_up),
            (InstantDefensePowerWeak, instant::defense_power_weak),
            (InstantDrawCards, instant::draw_cards),
            (TriggerRecoverHp, triggered::recover_hp)
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
            objects,
            system_input,
        )
    }
}
