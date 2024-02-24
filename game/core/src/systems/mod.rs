extern crate alloc;
use alloc::collections::BTreeMap;
use rand::RngCore;
use spore_warriors_generated as generated;

use crate::battle::traits::FightLog;
use crate::contexts::{ContextType, CtxAdaptor};
use crate::errors::Error;
use crate::wrappings::{SystemId, Value};

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

pub type SystemController = BTreeMap<SystemId, SystemCallback>;

pub type SystemCallback = fn(
    &generated::ResourcePool,
    &mut dyn RngCore,
    &[Value],
    &mut [&mut dyn CtxAdaptor],
    Option<SystemInput>,
) -> Result<SystemReturn, Error>;

pub fn setup_system_controllers(controller: &mut SystemController) {
    controller.insert(SystemId::Damage, attack as SystemCallback);
    controller.insert(SystemId::MultipleDamage, multiple_attack as SystemCallback);
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
