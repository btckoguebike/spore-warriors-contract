use spore_warriors_generated as generated;

use crate::battle::traits::FightLog;
use crate::contexts::{ContextType, CtxAdaptor, SystemContext};
use crate::errors::Error;
use crate::game::SporeRng;
use crate::systems::{Command, SystemInput, SystemReturn};
use crate::wrappings::Value;

pub fn recover_hp(
    _: &generated::ResourcePool,
    _: &mut SporeRng,
    mut ctx: SystemContext,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let Some(SystemInput::Trigger(trigger)) = input else {
        return Ok(SystemReturn::Continue(vec![]));
    };
    let Some(duration) = ctx.system.duration else {
        return Err(Error::BattleUnexpectedSystemContextDuration);
    };
    let mut logs = vec![];
    let mut remove = false;
    if ctx.duration_counter > 0 {
        if trigger == duration.trigger {
            ctx.duration_counter -= 1;
            remove = ctx.duration_counter == 0;
        }
    } else {
        ctx.duration_counter = duration.count;
        objects.iter_mut().for_each(|object| {
            object.add_mounting_system(&ctx);
            logs.push(FightLog::AddSystem(object.offset(), ctx.clone()));
        });
        return Ok(SystemReturn::Continue(vec![Command::AddLogs(logs)]));
    }
    let mut iter = ctx.system.args.iter();
    let Some(Value(hp)) = iter.next() else {
        return Err(Error::BattleUnexpectedSystemArgs);
    };
    let value = *hp;
    for object in objects.iter_mut() {
        match object.context_type() {
            ContextType::Warrior => {
                let warrior = object.warrior()?;
                warrior.hp = warrior.hp.saturating_add(value);
            }
            ContextType::Enemy => {
                let enemy = object.enemy()?;
                enemy.hp = enemy.hp.saturating_add(value);
            }
            ContextType::Card => continue,
        };
        logs.push(FightLog::SystemRecoverHp(object.offset(), value));
    }
    if remove {
        objects.iter_mut().for_each(|object| {
            object.remove_mounting_system(&ctx);
            logs.push(FightLog::RemoveSystem(object.offset(), ctx.clone()));
        });
    }
    Ok(SystemReturn::Continue(vec![]))
}
