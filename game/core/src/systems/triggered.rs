use spore_warriors_generated as generated;

use crate::apply_system;
use crate::battle::traits::FightLog;
use crate::contexts::{ContextType, CtxAdaptor, SystemContext};
use crate::errors::Error;
use crate::game::SporeRng;
use crate::systems::applications::healing_apply;
use crate::systems::{Command, SystemInput, SystemReturn};
use crate::wrappings::Value;

use super::applications::attack_apply;

enum TriggerResult {
    BreakOut(SystemReturn),
    Continue(Vec<FightLog>, SystemContext, FightLog),
}

fn run_trigger(
    mut ctx: SystemContext,
    objects: &mut [&mut dyn CtxAdaptor],
    input: &Option<SystemInput>,
) -> Result<TriggerResult, Error> {
    let Some(SystemInput::Trigger(trigger)) = &input else {
        return Ok(TriggerResult::BreakOut(SystemReturn::Continue(vec![])));
    };
    let Some(duration) = ctx.system.duration else {
        return Err(Error::BattleUnexpectedSystemContextDuration);
    };
    let mut logs = vec![];
    if ctx.duration_counter > 0 {
        if trigger == &duration.trigger {
            ctx.duration_counter -= 1;
            objects.iter_mut().for_each(|object| {
                if ctx.duration_counter == 0 {
                    object.remove_mounting_system(&ctx);
                    logs.push(FightLog::RemoveSystem(object.offset(), ctx.clone()));
                } else {
                    object.update_mounting_system(&ctx);
                    logs.push(FightLog::UpdateSystem(object.offset(), ctx.clone()));
                }
            });
        }
    } else {
        assert!(duration.count > 0);
        ctx.duration_counter = duration.count;
        objects.iter_mut().for_each(|object| {
            object.add_mounting_system(&ctx);
            logs.push(FightLog::AddSystem(object.offset(), ctx.clone()));
        });
        return Ok(TriggerResult::BreakOut(SystemReturn::Continue(vec![
            Command::AddLogs(logs),
        ])));
    }
    Ok(TriggerResult::Continue(logs, ctx, trigger.clone()))
}

// recover hp after specific trigger triggered, maybe can last for some period
pub fn recover_hp(
    _: &generated::ResourcePool,
    _: &mut SporeRng,
    ctx: SystemContext,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let (mut logs, ctx) = match run_trigger(ctx, objects, &input)? {
        TriggerResult::BreakOut(result) => return Ok(result),
        TriggerResult::Continue(logs, ctx, _) => (logs, ctx),
    };
    let mut iter = ctx.system.args.iter();
    apply_system!(logs, iter, input, objects, u16, healing_apply);
    Ok(SystemReturn::Continue(vec![Command::AddLogs(logs)]))
}

pub fn recover_hp_to_attack(
    _: &generated::ResourcePool,
    _: &mut SporeRng,
    ctx: SystemContext,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let (mut logs, _, trigger) = match run_trigger(ctx, objects, &input)? {
        TriggerResult::BreakOut(result) => return Ok(result),
        TriggerResult::Continue(logs, ctx, trigger) => (logs, ctx, trigger),
    };
    let FightLog::SystemRecoverHp(_, hp) = trigger else {
        return Ok(SystemReturn::Continue(vec![Command::AddLogs(logs)]));
    };
    if let Some(SystemInput::Trigger(FightLog::GameOver)) = input {
        return Ok(SystemReturn::Continue(vec![]));
    }
    for object in objects.iter_mut() {
        attack_apply(&mut logs, hp, object)?;
    }
    Ok(SystemReturn::Continue(vec![Command::AddLogs(logs)]))
}

pub fn attack_up(
    _: &generated::ResourcePool,
    _: &mut SporeRng,
    ctx: SystemContext,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let (mut logs, ctx) = match run_trigger(ctx, objects, &input)? {
        TriggerResult::BreakOut(result) => return Ok(result),
        TriggerResult::Continue(logs, ctx, _) => (logs, ctx),
    };
    let mut iter = ctx.system.args.iter();
    apply_system!(
        logs,
        iter,
        input,
        objects,
        attack,
        u8,
        saturating_add,
        SystemAttackPowerUp
    );
    Ok(SystemReturn::Continue(vec![Command::AddLogs(logs)]))
}

pub fn attack_down(
    _: &generated::ResourcePool,
    _: &mut SporeRng,
    ctx: SystemContext,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let (mut logs, ctx) = match run_trigger(ctx, objects, &input)? {
        TriggerResult::BreakOut(result) => return Ok(result),
        TriggerResult::Continue(logs, ctx, _) => (logs, ctx),
    };
    let mut iter = ctx.system.args.iter();
    apply_system!(
        logs,
        iter,
        input,
        objects,
        attack,
        u8,
        saturating_sub,
        SystemAttackPowerDown
    );
    Ok(SystemReturn::Continue(vec![Command::AddLogs(logs)]))
}
