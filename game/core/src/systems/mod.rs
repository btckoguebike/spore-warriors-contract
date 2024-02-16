extern crate alloc;
use alloc::collections::BTreeMap;
use rand::RngCore;
use spore_warriors_generated as generated;

use crate::contexts::CtxAdaptor;
use crate::errors::Error;
use crate::fight::traits::FightLog;
use crate::wrappings::Value;

#[derive(PartialEq, PartialOrd, Eq, Ord)]
pub enum SystemId {
    NormalDamage,
}

#[derive(PartialEq)]
pub enum SystemReturn {
    Null,
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
    &[&mut dyn CtxAdaptor],
    Option<SystemInput>,
) -> SystemReturn;

pub struct GameSystem<'a, T: RngCore> {
    resource_pool: &'a generated::ResourcePool,
    rng: &'a mut T,
    system_callbacks: BTreeMap<SystemId, SystemCallback>,
}

impl<'a, T: RngCore> GameSystem<'a, T> {
    pub fn new(resource_pool: &'a generated::ResourcePool, rng: &'a mut T) -> Self {
        let mut system_callbacks = BTreeMap::new();
        system_callbacks.insert(SystemId::NormalDamage, normal_attack as SystemCallback);
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
        contexts: &[&mut dyn CtxAdaptor],
        system_input: Option<SystemInput>,
    ) -> Result<SystemReturn, Error> {
        let system = self
            .system_callbacks
            .get(&system_id)
            .ok_or(Error::SystemMissing)?;
        Ok(system(
            self.resource_pool,
            self.rng,
            args,
            contexts,
            system_input,
        ))
    }
}

fn normal_attack(
    _resource_pool: &generated::ResourcePool,
    _rng: &mut dyn RngCore,
    _args: &[Value],
    _contexts: &[&mut dyn CtxAdaptor],
    _extra: Option<SystemInput>,
) -> SystemReturn {
    SystemReturn::Null
}
