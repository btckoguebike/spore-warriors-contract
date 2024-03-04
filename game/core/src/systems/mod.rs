extern crate alloc;
use alloc::collections::BTreeMap;
use spore_warriors_generated as generated;

use crate::battle::traits::FightLog;
use crate::contexts::CtxAdaptor;
use crate::errors::Error;
use crate::game::SporeRng;
use crate::wrappings::{System, SystemId, Value};

mod simple;

pub enum SystemReturn {
    RequireCardSelect,
    DrawCard(u8),
    SystemLog(Vec<FightLog>),
    PendingSystems(Vec<System>, Vec<FightLog>),
}

#[derive(Clone)]
pub enum SystemInput {
    Selection(Vec<usize>),
    Trigger(FightLog),
    GameOver,
}

pub type SystemCallback = fn(
    &generated::ResourcePool,
    &mut SporeRng,
    &[Value],
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
            (Damage, simple::attack),
            (MultipleDamage, simple::multiple_attack),
            (Healing, simple::healing),
            (AttackPowerUp, simple::attack_power_up),
            (DefensePowerUp, simple::defense_power_up)
        );
        Self {
            resource_pool,
            rng,
            controller,
        }
    }

    pub fn system_call(
        &mut self,
        system: &System,
        contexts: &mut [&mut dyn CtxAdaptor],
        system_input: Option<SystemInput>,
    ) -> Result<SystemReturn, Error> {
        let system_trigger = self
            .controller
            .get(&system.system_id)
            .ok_or(Error::SystemTriggerMissing)?;
        system_trigger(
            &self.resource_pool,
            &mut self.rng,
            &system.args,
            contexts,
            system_input,
        )
    }
}
