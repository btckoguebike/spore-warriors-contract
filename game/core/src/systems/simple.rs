extern crate alloc;
use spore_warriors_generated as generated;

use crate::battle::traits::FightLog;
use crate::contexts::{ContextType, CtxAdaptor};
use crate::errors::Error;
use crate::game::SporeRng;
use crate::systems::{SystemInput, SystemReturn};
use crate::wrappings::{System, Value};

macro_rules! field_change {
    ($iter:ident, $input:ident, $ctxs:ident, $field:ident, $ft:ty, $meth:ident, $log:ident) => {{
        let mut logs = vec![];
        if let Some(SystemInput::GameOver) = $input {
            return Ok(SystemReturn::SystemLog(vec![]));
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
}

macro_rules! continue_system {
    ($iter:ident, $resource:ident, $rng:ident, $ctxs:ident) => {{
        let mut continue_systems = vec![];
        while let (Some(Value(instant)), Some(Value(system_id))) = ($iter.next(), $iter.next()) {
            let system = {
                let system = $resource
                    .system_pool()
                    .into_iter()
                    .find(|v| u16::from(v.id()) == *system_id)
                    .ok_or(Error::ResourceBrokenSystemId)?;
                System::randomized($resource, system, $rng)?
            };
            if instant == &1u16 {
                continue_systems.push(system);
            } else {
                for object in $ctxs.iter_mut() {
                    match object.context_type() {
                        ContextType::Warrior => {
                            object.warrior()?.mounting_systems.push(system.clone());
                        }
                        ContextType::Enemy => {
                            object.enemy()?.mounting_systems.push(system.clone());
                        }
                        ContextType::Card => {}
                    }
                }
            }
        }
        continue_systems
    }};
}

// normally inflict single damage
pub fn attack(
    resource_pool: &generated::ResourcePool,
    rng: &mut SporeRng,
    args: &[Value],
    contexts: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut iter = args.iter();
    let logs = field_change!(iter, input, contexts, hp, u16, saturating_sub, SystemDamage);
    let systems = continue_system!(iter, resource_pool, rng, contexts);
    if !systems.is_empty() {
        Ok(SystemReturn::PendingSystems(systems, logs))
    } else {
        Ok(SystemReturn::SystemLog(logs))
    }
}

// normally inflict multiple damage
pub fn multiple_attack(
    resource_pool: &generated::ResourcePool,
    rng: &mut SporeRng,
    args: &[Value],
    contexts: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    if let Some(SystemInput::GameOver) = input {
        return Ok(SystemReturn::SystemLog(vec![]));
    }
    let mut iter = args.iter();
    let (Some(Value(damage)), Some(Value(count))) = (iter.next(), iter.next()) else {
        return Err(Error::BattleUnexpectedSystemArgs);
    };
    let value = *damage;
    let mut logs = vec![];
    for object in contexts.iter_mut() {
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
                    ContextType::Card => return Ok(()),
                };
                logs.push(FightLog::SystemDamage(object.offset(), value));
                Ok(())
            })
            .collect::<Result<_, _>>()?;
    }
    let systems = continue_system!(iter, resource_pool, rng, contexts);
    if !systems.is_empty() {
        Ok(SystemReturn::PendingSystems(systems, logs))
    } else {
        Ok(SystemReturn::SystemLog(logs))
    }
}

// normally cure object's hp
pub fn healing(
    resource_pool: &generated::ResourcePool,
    rng: &mut SporeRng,
    args: &[Value],
    contexts: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut iter = args.iter();
    let logs = field_change!(
        iter,
        input,
        contexts,
        hp,
        u16,
        saturating_add,
        SystemRecoverHp
    );
    let systems = continue_system!(iter, resource_pool, rng, contexts);
    if !systems.is_empty() {
        Ok(SystemReturn::PendingSystems(systems, logs))
    } else {
        Ok(SystemReturn::SystemLog(logs))
    }
}

// normally increate ATK, which impacts object's damage
pub fn attack_power_up(
    resource_pool: &generated::ResourcePool,
    rng: &mut SporeRng,
    args: &[Value],
    contexts: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut iter = args.iter();
    let logs = field_change!(
        iter,
        input,
        contexts,
        attack,
        u8,
        saturating_add,
        SystemAttackPowerUp
    );
    let systems = continue_system!(iter, resource_pool, rng, contexts);
    if !systems.is_empty() {
        Ok(SystemReturn::PendingSystems(systems, logs))
    } else {
        Ok(SystemReturn::SystemLog(logs))
    }
}

pub fn defense_power_up(
    resource_pool: &generated::ResourcePool,
    rng: &mut SporeRng,
    args: &[Value],
    contexts: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut iter = args.iter();
    let logs = field_change!(
        iter,
        input,
        contexts,
        defense,
        u8,
        saturating_add,
        SystemDefensePowerUp
    );
    let systems = continue_system!(iter, resource_pool, rng, contexts);
    if !systems.is_empty() {
        Ok(SystemReturn::PendingSystems(systems, logs))
    } else {
        Ok(SystemReturn::SystemLog(logs))
    }
}
