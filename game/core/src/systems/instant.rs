extern crate alloc;
use core::slice::Iter;
use spore_warriors_generated as generated;

use crate::apply_system;
use crate::battle::traits::FightLog;
use crate::contexts::{ContextType, CtxAdaptor, SystemContext};
use crate::errors::Error;
use crate::game::SporeRng;
use crate::systems::applications::{
    armor_up_apply, attack_apply, attack_power_weak_apply, defense_power_weak_apply,
    draw_count_down_apply, draw_count_up_apply, healing_apply, shield_up_apply,
};
use crate::systems::{Command, SystemInput, SystemReturn};
use crate::wrappings::{System, Value};

fn check_extensions<'a>(
    mut iter: Iter<'a, Value>,
    resource_pool: &generated::ResourcePool,
    rng: &mut SporeRng,
    commands: Vec<Command>,
) -> Result<SystemReturn, Error> {
    let mut continue_systems = vec![];
    while let Some(Value(system_id)) = iter.next() {
        let system = {
            let system = resource_pool
                .system_pool()
                .into_iter()
                .find(|v| u16::from(v.id()) == *system_id)
                .ok_or(Error::ResourceBrokenSystemId)?;
            System::randomized(resource_pool, system, rng)?
        };
        continue_systems.push(system);
    }
    if continue_systems.is_empty() {
        Ok(SystemReturn::Continue(commands))
    } else {
        Ok(SystemReturn::PendingSystems(continue_systems, commands))
    }
}

// normally inflict single damage
pub fn attack(
    resource_pool: &generated::ResourcePool,
    rng: &mut SporeRng,
    ctx: SystemContext,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut logs = vec![];
    let mut iter = ctx.system.args.iter();
    apply_system!(logs, iter, input, objects, u16, attack_apply);
    check_extensions(iter, resource_pool, rng, vec![Command::AddLogs(logs)])
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
    let mut logs = vec![];
    for object in objects.iter_mut() {
        (0..*count)
            .map(|_| attack_apply(&mut logs, *damage, object))
            .collect::<Result<_, _>>()?;
    }
    check_extensions(iter, resource_pool, rng, vec![Command::AddLogs(logs)])
}

// normally increase DEF
pub fn armor_up(
    resource_pool: &generated::ResourcePool,
    rng: &mut SporeRng,
    ctx: SystemContext,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut logs = vec![];
    let mut iter = ctx.system.args.iter();
    apply_system!(logs, iter, input, objects, u16, armor_up_apply);
    check_extensions(iter, resource_pool, rng, vec![Command::AddLogs(logs)])
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
    let logs = apply_system!(
        iter,
        input,
        objects,
        armor,
        u16,
        saturating_sub,
        SystemArmorDown
    );
    check_extensions(iter, resource_pool, rng, vec![Command::AddLogs(logs)])
}

// normally increase SHD
pub fn shield_up(
    resource_pool: &generated::ResourcePool,
    rng: &mut SporeRng,
    ctx: SystemContext,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut logs = vec![];
    let mut iter = ctx.system.args.iter();
    apply_system!(logs, iter, input, objects, u16, shield_up_apply);
    check_extensions(iter, resource_pool, rng, vec![Command::AddLogs(logs)])
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
    let logs = apply_system!(
        iter,
        input,
        objects,
        shield,
        u16,
        saturating_sub,
        SystemShieldDown
    );
    check_extensions(iter, resource_pool, rng, vec![Command::AddLogs(logs)])
}

// normally cure object's hp
pub fn healing(
    resource_pool: &generated::ResourcePool,
    rng: &mut SporeRng,
    ctx: SystemContext,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut logs = vec![];
    let mut iter = ctx.system.args.iter();
    apply_system!(logs, iter, input, objects, u16, healing_apply);
    check_extensions(iter, resource_pool, rng, vec![Command::AddLogs(logs)])
}

// normally increase the drawn cards count
pub fn draw_count_up(
    resource_pool: &generated::ResourcePool,
    rng: &mut SporeRng,
    ctx: SystemContext,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut logs = vec![];
    let mut iter = ctx.system.args.iter();
    apply_system!(logs, iter, input, objects, u8, draw_count_up_apply);
    check_extensions(iter, resource_pool, rng, vec![Command::AddLogs(logs)])
}

// normally increase the drawn cards count
pub fn draw_count_down(
    resource_pool: &generated::ResourcePool,
    rng: &mut SporeRng,
    ctx: SystemContext,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut logs = vec![];
    let mut iter = ctx.system.args.iter();
    apply_system!(logs, iter, input, objects, u8, draw_count_down_apply);
    check_extensions(iter, resource_pool, rng, vec![Command::AddLogs(logs)])
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
    let logs = apply_system!(
        iter,
        input,
        objects,
        attack,
        u8,
        saturating_add,
        SystemAttackPowerUp
    );
    check_extensions(iter, resource_pool, rng, vec![Command::AddLogs(logs)])
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
    let logs = apply_system!(
        iter,
        input,
        objects,
        defense,
        u8,
        saturating_add,
        SystemDefensePowerUp
    );
    check_extensions(iter, resource_pool, rng, vec![Command::AddLogs(logs)])
}

// normally increase ATK_WEAK value
pub fn attack_power_weak(
    resource_pool: &generated::ResourcePool,
    rng: &mut SporeRng,
    ctx: SystemContext,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut logs = vec![];
    let mut iter = ctx.system.args.iter();
    apply_system!(logs, iter, input, objects, u8, attack_power_weak_apply);
    check_extensions(iter, resource_pool, rng, vec![Command::AddLogs(logs)])
}

// normally increase DEF_WEAK value
pub fn defense_power_weak(
    resource_pool: &generated::ResourcePool,
    rng: &mut SporeRng,
    ctx: SystemContext,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut logs = vec![];
    let mut iter = ctx.system.args.iter();
    apply_system!(logs, iter, input, objects, u8, defense_power_weak_apply);
    check_extensions(iter, resource_pool, rng, vec![Command::AddLogs(logs)])
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
    check_extensions(iter, resource_pool, rng, commands)
}
