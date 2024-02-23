extern crate alloc;
use alloc::collections::BTreeMap;
use rand::RngCore;
use spore_warriors_generated as generated;

use crate::battle::traits::FightLog;
use crate::contexts::{ContextType, CtxAdaptor};
use crate::errors::Error;
use crate::wrappings::{System, SystemId, Value};

pub enum SystemReturn {
    RequireCardSelect,
    DrawCard(u8),
    SystemLog(Vec<FightLog>),
}

#[derive(Clone)]
pub enum SystemInput {
    Selection(Vec<usize>),
    Trigger(FightLog),
    GameOver,
}

type SystemCallback = fn(
    &generated::ResourcePool,
    &mut dyn RngCore,
    &[Value],
    &mut [&mut dyn CtxAdaptor],
    Option<SystemInput>,
) -> Result<SystemReturn, Error>;

pub struct SystemController<'a, T: RngCore> {
    pub resource_pool: &'a generated::ResourcePool,
    pub rng: &'a mut T,
    system_callbacks: BTreeMap<SystemId, SystemCallback>,
}

impl<'a, T: RngCore> SystemController<'a, T> {
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

    pub fn call(
        &mut self,
        system: &System,
        contexts: &mut [&mut dyn CtxAdaptor],
        system_input: Option<SystemInput>,
    ) -> Result<SystemReturn, Error> {
        let trigger = self
            .system_callbacks
            .get(&system.id)
            .ok_or(Error::SystemMissing)?;
        trigger(
            self.resource_pool,
            self.rng,
            &system.args,
            contexts,
            system_input,
        )
    }
}

fn attack(
    _: &generated::ResourcePool,
    _: &mut dyn RngCore,
    args: &[Value],
    contexts: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    if let Some(SystemInput::GameOver) = input {
        return Ok(SystemReturn::SystemLog(vec![]));
    }
    let Some(Value(damage)) = args.get(0) else {
        return Err(Error::BattleUnexpectedSystemArgs);
    };
    let value = *damage;
    let mut logs = vec![];
    for object in contexts {
        match object.context_type() {
            ContextType::Warrior => {
                let warrior = object.warrior()?;
                warrior.hp -= value;
            }
            ContextType::Enemy => {
                let enemy = object.enemy()?;
                enemy.hp -= value;
            }
            ContextType::Card => unreachable!(),
        };
        logs.push(FightLog::SystemDamage(object.offset(), *damage));
    }
    Ok(SystemReturn::SystemLog(logs))
}

fn multiple_attack(
    _: &generated::ResourcePool,
    _: &mut dyn RngCore,
    args: &[Value],
    contexts: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    if let Some(SystemInput::GameOver) = input {
        return Ok(SystemReturn::SystemLog(vec![]));
    }
    let (Some(Value(damage)), Some(Value(count))) = (args.get(0), args.get(1)) else {
        return Err(Error::BattleUnexpectedSystemArgs);
    };
    let value = *damage;
    let mut logs = vec![];
    for object in contexts {
        (0..*count)
            .map(|_| {
                match object.context_type() {
                    ContextType::Warrior => {
                        let warrior = object.warrior()?;
                        warrior.hp -= value;
                    }
                    ContextType::Enemy => {
                        let enemy = object.enemy()?;
                        enemy.hp -= value;
                    }
                    ContextType::Card => unreachable!(),
                };
                logs.push(FightLog::SystemDamage(object.offset(), *damage));
                Ok(())
            })
            .collect::<Result<_, _>>()?;
    }
    Ok(SystemReturn::SystemLog(logs))
}
