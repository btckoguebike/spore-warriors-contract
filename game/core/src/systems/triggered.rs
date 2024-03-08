use spore_warriors_generated as generated;

use crate::apply_system;
use crate::battle::traits::FightLog;
use crate::contexts::{CtxAdaptor, SystemContext};
use crate::errors::Error;
use crate::game::SporeRng;
use crate::systems::applications::healing_apply;
use crate::systems::{Command, SystemInput, SystemReturn};
use crate::wrappings::Value;

enum TriggerResult {
    BreakOut(SystemReturn),
    Continue(Vec<FightLog>, SystemContext),
}

fn check_trigger(
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
    let mut remove = false;
    if ctx.duration_counter > 0 {
        if trigger == &duration.trigger {
            ctx.duration_counter -= 1;
            remove = ctx.duration_counter == 0;
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
    if remove {
        objects.iter_mut().for_each(|object| {
            object.remove_mounting_system(&ctx);
            logs.push(FightLog::RemoveSystem(object.offset(), ctx.clone()));
        });
    }
    Ok(TriggerResult::Continue(logs, ctx))
}

// recover hp after specific trigger triggered, maybe can last for some period
pub fn recover_hp(
    _: &generated::ResourcePool,
    _: &mut SporeRng,
    ctx: SystemContext,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let (mut logs, ctx) = match check_trigger(ctx, objects, &input)? {
        TriggerResult::BreakOut(result) => return Ok(result),
        TriggerResult::Continue(logs, ctx) => (logs, ctx),
    };
    let mut iter = ctx.system.args.iter();
    apply_system!(logs, iter, input, objects, u16, healing_apply);
    Ok(SystemReturn::Continue(vec![Command::AddLogs(logs)]))
}
