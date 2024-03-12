use spore_warriors_generated as generated;

use crate::battle::traits::FightLog;
use crate::contexts::{ContextType, CtxAdaptor, SystemContext};
use crate::errors::Error;
use crate::game::SporeRng;
use crate::systems::applications::healing_apply;
use crate::systems::{Command, SystemInput, SystemReturn};
use crate::wrappings::Value;
use crate::{apply_system, filter_objects};

use super::applications::attack_apply;

enum TriggerResult {
    BreakOut(SystemReturn),
    Continue(Vec<FightLog>, SystemContext, FightLog),
}

fn run_trigger(
    mut ctx: SystemContext,
    objects: &mut Vec<&mut &mut dyn CtxAdaptor>,
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
    _: usize,
    targets: Vec<usize>,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut objects = filter_objects!(objects, targets);
    let (mut logs, ctx) = match run_trigger(ctx, &mut objects, &input)? {
        TriggerResult::BreakOut(result) => return Ok(result),
        TriggerResult::Continue(logs, ctx, _) => (logs, ctx),
    };
    let mut iter = ctx.system.args.iter();
    apply_system!(logs, iter, input, objects, u16, healing_apply);
    Ok(SystemReturn::Continue(vec![Command::AddLogs(logs)]))
}

// recover hp and then turn to damage on other object
pub fn recover_hp_to_attack(
    _: &generated::ResourcePool,
    _: &mut SporeRng,
    ctx: SystemContext,
    caster: usize,
    targets: Vec<usize>,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut trigger_objects = filter_objects!(objects, { vec![caster] });
    let (mut logs, mut ctx, trigger) = match run_trigger(ctx, &mut trigger_objects, &input)? {
        TriggerResult::BreakOut(result) => return Ok(result),
        TriggerResult::Continue(logs, ctx, trigger) => (logs, ctx, trigger),
    };
    if let Some(SystemInput::Trigger(FightLog::GameOver)) = input {
        return Ok(SystemReturn::Continue(vec![]));
    }
    let FightLog::SystemRecoverHp(_, hp) = trigger else {
        return Ok(SystemReturn::Continue(vec![Command::AddLogs(logs)]));
    };
    if ctx.register_3.is_empty() {
        ctx.register_3 = targets;
        trigger_objects.iter_mut().for_each(|object| {
            object.update_mounting_system(&ctx);
            logs.push(FightLog::UpdateSystem(object.offset(), ctx.clone()));
        });
    }
    let objects = filter_objects!(objects, { &ctx.register_3 });
    for object in objects {
        attack_apply(&mut logs, hp, object)?;
    }
    Ok(SystemReturn::Continue(vec![Command::AddLogs(logs)]))
}

// roundly increase object's attack power
pub fn attack_up(
    _: &generated::ResourcePool,
    _: &mut SporeRng,
    ctx: SystemContext,
    caster: usize,
    _: Vec<usize>,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut objects = filter_objects!(objects, { vec![caster] });
    let (mut logs, ctx) = match run_trigger(ctx, &mut objects, &input)? {
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

// roundly decrease object's attack power
pub fn attack_down(
    _: &generated::ResourcePool,
    _: &mut SporeRng,
    ctx: SystemContext,
    caster: usize,
    _: Vec<usize>,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut objects = filter_objects!(objects, { vec![caster] });
    let (mut logs, ctx) = match run_trigger(ctx, &mut objects, &input)? {
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

// decrease card power cost when the other one used with the same card class
// pub fn card_cost_light_up(
//     _: &generated::ResourcePool,
//     _: &mut SporeRng,
//     ctx: SystemContext,
//     caster: usize,
//     _: Vec<usize>,
//     objects: &mut [&mut dyn CtxAdaptor],
//     input: Option<SystemInput>,
// ) -> Result<SystemReturn, Error> {
//     let mut trigger_objects = filter_objects!(objects, { vec![caster] });
//     let (logs, trigger) = match run_trigger(ctx, &mut trigger_objects, &input)? {
//         TriggerResult::BreakOut(result) => return Ok(result),
//         TriggerResult::Continue(logs, _, trigger) => (logs, trigger),
//     };
//     let FightLog::HandCardUse(card_offset) = trigger else {
//         return Ok(SystemReturn::Continue(vec![Command::AddLogs(logs)]));
//     };
//     let mut used_card = None;
//     let mut trigger_card = None;
//     let objects = filter_objects!(objects, { vec![card_offset, caster] });
//     for object in objects {
//         if object.offset() == caster {
//             trigger_card = Some(object.card()?);
//         } else if object.offset() == card_offset {
//             used_card = Some(object.card()?);
//         }
//     }
//     let (Some(used_card), Some(trigger_card)) = (used_card, trigger_card) else {
//         return Err(Error::BattleCardOffsetNotFound);
//     };
//     if used_card.card.class == trigger_card.card.class {
//         trigger_card.power_cost = trigger_card.power_cost.saturating_sub(1);
//     }
//     Ok(SystemReturn::Continue(vec![Command::AddLogs(logs)]))
// }
