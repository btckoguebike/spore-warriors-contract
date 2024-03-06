extern crate alloc;
use alloc::collections::BTreeMap;
use spore_warriors_generated as generated;

use crate::battle::traits::FightLog;
use crate::contexts::{CtxAdaptor, SystemContext};
use crate::errors::Error;
use crate::game::SporeRng;
use crate::wrappings::{System, SystemId};

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
