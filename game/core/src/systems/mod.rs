extern crate alloc;
use alloc::collections::BTreeMap;
use rand::RngCore;
use spore_warriors_generated as generated;

use crate::contexts::CtxAdaptor;
use crate::errors::Error;
use crate::fight::traits::FightLog;
use crate::wrappings::{SystemId, Value};

pub enum SystemReturn {
    Discarded,
    Triggered,
    NeedCardSelect,
    DrawCard(u8),
    FightLog(Vec<FightLog>),
}

#[derive(Clone)]
pub enum SystemInput {
    CardSelect(Vec<usize>),
    FightLog(FightLog),
}

type SystemCallback = fn(
    &generated::ResourcePool,
    &mut dyn RngCore,
    &[Value],
    &mut [&mut dyn CtxAdaptor],
    Option<SystemInput>,
) -> Result<SystemReturn, Error>;

pub struct GameSystem<'a, T: RngCore> {
    resource_pool: &'a generated::ResourcePool,
    rng: &'a mut T,
    system_callbacks: BTreeMap<SystemId, SystemCallback>,
}

impl<'a, T: RngCore> GameSystem<'a, T> {
    pub fn new(resource_pool: &'a generated::ResourcePool, rng: &'a mut T) -> Self {
        let mut system_callbacks = BTreeMap::new();
        system_callbacks.insert(SystemId::Damage, attack as SystemCallback);
        system_callbacks.insert(SystemId::MultipleDamage, multiple_attack as SystemCallback);
        Self {
            resource_pool,
            rng,
            system_callbacks,
        }
    }

    pub fn resource_pool(&self) -> &'a generated::ResourcePool {
        self.resource_pool
    }

    pub fn rng(&mut self) -> &mut T {
        self.rng
    }

    pub fn call(
        &mut self,
        system_id: SystemId,
        args: &[Value],
        contexts: &mut [&mut dyn CtxAdaptor],
        system_input: Option<SystemInput>,
    ) -> Result<SystemReturn, Error> {
        let system = self
            .system_callbacks
            .get(&system_id)
            .ok_or(Error::SystemMissing)?;
        system(self.resource_pool, self.rng, args, contexts, system_input)
    }
}

fn attack(
    _resource_pool: &generated::ResourcePool,
    _rng: &mut dyn RngCore,
    args: &[Value],
    contexts: &mut [&mut dyn CtxAdaptor],
    _extra: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let Some(Value::Number(damage)) = args.get(0) else {
        return Err(Error::BattleUnexpectedSystemArgs);
    };
    let value = *damage as i16;
    let mut logs = vec![];
    for object in contexts {
        object.change_hp(-value);
        logs.push(FightLog::SystemDamage(object.offset(), *damage));
    }
    Ok(SystemReturn::FightLog(logs))
}

fn multiple_attack(
    _resource_pool: &generated::ResourcePool,
    _rng: &mut dyn RngCore,
    args: &[Value],
    contexts: &mut [&mut dyn CtxAdaptor],
    _extra: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let (Some(Value::Number(damage)), Some(Value::Number(count))) = (args.get(0), args.get(1))
    else {
        return Err(Error::BattleUnexpectedSystemArgs);
    };
    let value = *damage as i16;
    let mut logs = vec![];
    for object in contexts {
        (0..*count).for_each(|_| {
            object.change_hp(-value);
            logs.push(FightLog::SystemDamage(object.offset(), *damage));
        });
    }
    Ok(SystemReturn::FightLog(logs))
}
