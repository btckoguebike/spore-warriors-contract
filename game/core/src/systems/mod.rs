extern crate alloc;
use alloc::collections::BTreeMap;
use rand::RngCore;
use spore_warriors_generated as generated;

use crate::battle::traits::FightLog;
use crate::contexts::{ContextType, CtxAdaptor};
use crate::errors::Error;
use crate::game::SporeRng;
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

pub type SystemCallback = fn(
    &generated::ResourcePool,
    &mut dyn RngCore,
    &[Value],
    &mut [&mut dyn CtxAdaptor],
    Option<SystemInput>,
) -> Result<SystemReturn, Error>;

pub struct SystemController {
    pub resource_pool: generated::ResourcePool,
    pub rng: SporeRng,
    controller: BTreeMap<SystemId, SystemCallback>,
}

impl SystemController {
    pub fn new(resource_pool: generated::ResourcePool, rng: SporeRng) -> Self {
        let mut controller = BTreeMap::new();
        controller.insert(SystemId::Damage, attack as SystemCallback);
        controller.insert(SystemId::MultipleDamage, multiple_attack as SystemCallback);
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
