extern crate alloc;
use spore_warriors_generated as generated;

use crate::battle::traits::FightLog;
use crate::contexts::{ContextType, CtxAdaptor, SystemContext};
use crate::errors::Error;
use crate::game::SporeRng;
use crate::systems::{Command, SystemInput, SystemReturn};
use crate::wrappings::{System, Value};

macro_rules! simple_change {
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
}

macro_rules! continue_system {
    ($iter:ident, $resource:ident, $rng:ident) => {{
        let mut continue_systems = vec![];
        while let Some(Value(system_id)) = $iter.next() {
            let system = {
                let system = $resource
                    .system_pool()
                    .into_iter()
                    .find(|v| u16::from(v.id()) == *system_id)
                    .ok_or(Error::ResourceBrokenSystemId)?;
                System::randomized($resource, system, $rng)?
            };
            continue_systems.push(system);
        }
        continue_systems
    }};
}

// normally inflict single damage
pub fn attack(
    resource_pool: &generated::ResourcePool,
    rng: &mut SporeRng,
    ctx: SystemContext,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut iter = ctx.system.args.iter();
    let logs = simple_change!(iter, input, objects, hp, u16, saturating_sub, SystemDamage);
    let commands = vec![Command::AddLogs(logs)];
    let pending = continue_system!(iter, resource_pool, rng);
    if !pending.is_empty() {
        Ok(SystemReturn::PendingSystems(pending, commands))
    } else {
        Ok(SystemReturn::Continue(commands))
    }
}

// normally inflict multiple damage
pub fn multiple_attack(
    resource_pool: &generated::ResourcePool,
    rng: &mut SporeRng,
    ctx: SystemContext,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    if let Some(SystemInput::Trigger(FightLog::GameOver)) = input {
        return Ok(SystemReturn::Continue(vec![]));
    }
    let mut iter = ctx.system.args.iter();
    let (Some(Value(damage)), Some(Value(count))) = (iter.next(), iter.next()) else {
        return Err(Error::BattleUnexpectedSystemArgs);
    };
    let value = *damage;
    let mut logs = vec![];
    for object in objects.iter_mut() {
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
    let commands = vec![Command::AddLogs(logs)];
    let pending = continue_system!(iter, resource_pool, rng);
    if !pending.is_empty() {
        Ok(SystemReturn::PendingSystems(pending, commands))
    } else {
        Ok(SystemReturn::Continue(commands))
    }
}

// normally increase DEF
pub fn armor_up(
    resource_pool: &generated::ResourcePool,
    rng: &mut SporeRng,
    ctx: SystemContext,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut iter = ctx.system.args.iter();
    let logs = simple_change!(
        iter,
        input,
        objects,
        armor,
        u16,
        saturating_add,
        SystemArmorUp
    );
    let commands = vec![Command::AddLogs(logs)];
    let pending = continue_system!(iter, resource_pool, rng);
    if !pending.is_empty() {
        Ok(SystemReturn::PendingSystems(pending, commands))
    } else {
        Ok(SystemReturn::Continue(commands))
    }
}

// normally decrease DEF
pub fn armor_down(
    resource_pool: &generated::ResourcePool,
    rng: &mut SporeRng,
    ctx: SystemContext,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut iter = ctx.system.args.iter();
    let logs = simple_change!(
        iter,
        input,
        objects,
        armor,
        u16,
        saturating_sub,
        SystemArmorDown
    );
    let commands = vec![Command::AddLogs(logs)];
    let pending = continue_system!(iter, resource_pool, rng);
    if !pending.is_empty() {
        Ok(SystemReturn::PendingSystems(pending, commands))
    } else {
        Ok(SystemReturn::Continue(commands))
    }
}

// normally increase SHD
pub fn shield_up(
    resource_pool: &generated::ResourcePool,
    rng: &mut SporeRng,
    ctx: SystemContext,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut iter = ctx.system.args.iter();
    let logs = simple_change!(
        iter,
        input,
        objects,
        shield,
        u16,
        saturating_add,
        SystemShieldUp
    );
    let commands = vec![Command::AddLogs(logs)];
    let pending = continue_system!(iter, resource_pool, rng);
    if !pending.is_empty() {
        Ok(SystemReturn::PendingSystems(pending, commands))
    } else {
        Ok(SystemReturn::Continue(commands))
    }
}

// normally decrease SHD
pub fn shield_down(
    resource_pool: &generated::ResourcePool,
    rng: &mut SporeRng,
    ctx: SystemContext,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut iter = ctx.system.args.iter();
    let logs = simple_change!(
        iter,
        input,
        objects,
        shield,
        u16,
        saturating_sub,
        SystemShieldDown
    );
    let commands = vec![Command::AddLogs(logs)];
    let pending = continue_system!(iter, resource_pool, rng);
    if !pending.is_empty() {
        Ok(SystemReturn::PendingSystems(pending, commands))
    } else {
        Ok(SystemReturn::Continue(commands))
    }
}

// normally cure object's hp
pub fn healing(
    resource_pool: &generated::ResourcePool,
    rng: &mut SporeRng,
    ctx: SystemContext,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut iter = ctx.system.args.iter();
    let logs = simple_change!(
        iter,
        input,
        objects,
        hp,
        u16,
        saturating_add,
        SystemRecoverHp
    );
    let commands = vec![Command::AddLogs(logs)];
    let pending = continue_system!(iter, resource_pool, rng);
    if !pending.is_empty() {
        Ok(SystemReturn::PendingSystems(pending, commands))
    } else {
        Ok(SystemReturn::Continue(commands))
    }
}

// normally increase the drawn cards count
pub fn draw_count_up(
    resource_pool: &generated::ResourcePool,
    rng: &mut SporeRng,
    ctx: SystemContext,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    if let Some(SystemInput::Trigger(FightLog::GameOver)) = input {
        return Ok(SystemReturn::Continue(vec![]));
    }
    let mut iter = ctx.system.args.iter();
    let Some(Value(draw_count)) = iter.next() else {
        return Err(Error::BattleUnexpectedSystemArgs);
    };
    let value = *draw_count as u8;
    let mut logs = vec![];
    for object in objects.iter_mut() {
        match object.context_type() {
            ContextType::Warrior => {
                let warrior = object.warrior()?;
                warrior.draw_count = warrior.draw_count.saturating_add(value);
            }
            ContextType::Enemy => continue,
            ContextType::Card => continue,
        };
        logs.push(FightLog::SystemDrawCountUp(value));
    }
    let commands = vec![Command::AddLogs(logs)];
    let pending = continue_system!(iter, resource_pool, rng);
    if !pending.is_empty() {
        Ok(SystemReturn::PendingSystems(pending, commands))
    } else {
        Ok(SystemReturn::Continue(commands))
    }
}

// normally increase the drawn cards count
pub fn draw_count_down(
    resource_pool: &generated::ResourcePool,
    rng: &mut SporeRng,
    ctx: SystemContext,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    if let Some(SystemInput::Trigger(FightLog::GameOver)) = input {
        return Ok(SystemReturn::Continue(vec![]));
    }
    let mut iter = ctx.system.args.iter();
    let Some(Value(draw_count)) = iter.next() else {
        return Err(Error::BattleUnexpectedSystemArgs);
    };
    let value = *draw_count as u8;
    let mut logs = vec![];
    for object in objects.iter_mut() {
        match object.context_type() {
            ContextType::Warrior => {
                let warrior = object.warrior()?;
                warrior.draw_count = warrior.draw_count.saturating_sub(value);
            }
            ContextType::Enemy => continue,
            ContextType::Card => continue,
        };
        logs.push(FightLog::SystemDrawCountDown(value));
    }
    let commands = vec![Command::AddLogs(logs)];
    let pending = continue_system!(iter, resource_pool, rng);
    if !pending.is_empty() {
        Ok(SystemReturn::PendingSystems(pending, commands))
    } else {
        Ok(SystemReturn::Continue(commands))
    }
}

// normally increase ATK, which impacts object's damage
pub fn attack_power_up(
    resource_pool: &generated::ResourcePool,
    rng: &mut SporeRng,
    ctx: SystemContext,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut iter = ctx.system.args.iter();
    let logs = simple_change!(
        iter,
        input,
        objects,
        attack,
        u8,
        saturating_add,
        SystemAttackPowerUp
    );
    let commands = vec![Command::AddLogs(logs)];
    let pending = continue_system!(iter, resource_pool, rng);
    if !pending.is_empty() {
        Ok(SystemReturn::PendingSystems(pending, commands))
    } else {
        Ok(SystemReturn::Continue(commands))
    }
}

// normally increase DEF, which impacts object's sheild and defense
pub fn defense_power_up(
    resource_pool: &generated::ResourcePool,
    rng: &mut SporeRng,
    ctx: SystemContext,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut iter = ctx.system.args.iter();
    let logs = simple_change!(
        iter,
        input,
        objects,
        defense,
        u8,
        saturating_add,
        SystemDefensePowerUp
    );
    let commands = vec![Command::AddLogs(logs)];
    let pending = continue_system!(iter, resource_pool, rng);
    if !pending.is_empty() {
        Ok(SystemReturn::PendingSystems(pending, commands))
    } else {
        Ok(SystemReturn::Continue(commands))
    }
}

// normally increase ATK_WEAK value
pub fn attack_power_weak(
    resource_pool: &generated::ResourcePool,
    rng: &mut SporeRng,
    ctx: SystemContext,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut iter = ctx.system.args.iter();
    let logs = simple_change!(
        iter,
        input,
        objects,
        attack_weak,
        u8,
        saturating_add,
        SystemAttackWeak
    );
    let commands = vec![Command::AddLogs(logs)];
    let pending = continue_system!(iter, resource_pool, rng);
    if !pending.is_empty() {
        Ok(SystemReturn::PendingSystems(pending, commands))
    } else {
        Ok(SystemReturn::Continue(commands))
    }
}

// normally increase DEF_WEAK value
pub fn defense_power_weak(
    resource_pool: &generated::ResourcePool,
    rng: &mut SporeRng,
    ctx: SystemContext,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut iter = ctx.system.args.iter();
    let logs = simple_change!(
        iter,
        input,
        objects,
        defense_weak,
        u8,
        saturating_add,
        SystemDefenseWeak
    );
    let commands = vec![Command::AddLogs(logs)];
    let pending = continue_system!(iter, resource_pool, rng);
    if !pending.is_empty() {
        Ok(SystemReturn::PendingSystems(pending, commands))
    } else {
        Ok(SystemReturn::Continue(commands))
    }
}

// return DrawCard command
pub fn draw_cards(
    resource_pool: &generated::ResourcePool,
    rng: &mut SporeRng,
    ctx: SystemContext,
    _: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    if let Some(SystemInput::Trigger(FightLog::GameOver)) = input {
        return Ok(SystemReturn::Continue(vec![]));
    }
    let mut iter = ctx.system.args.iter();
    let Some(Value(count)) = iter.next() else {
        return Err(Error::BattleUnexpectedSystemArgs);
    };
    let commands = vec![Command::DrawCards(*count as u8)];
    let pending = continue_system!(iter, resource_pool, rng);
    if !pending.is_empty() {
        Ok(SystemReturn::PendingSystems(pending, commands))
    } else {
        Ok(SystemReturn::Continue(commands))
    }
}
