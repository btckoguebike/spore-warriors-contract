mod card;
mod enemy;
mod system;
mod warrior;

pub use card::*;
pub use enemy::*;
pub use system::*;
pub use warrior::*;

extern crate alloc;
use alloc::vec::Vec;

use crate::errors::Error;

pub enum ContextType {
    Warrior,
    Enemy,
    Card,
}

pub trait CtxAdaptor {
    fn context_type(&self) -> ContextType;

    fn offset(&self) -> usize;

    fn add_mounting_system(&mut self, ctx: &SystemContext) -> bool;

    fn update_mounting_system(&mut self, ctx: &SystemContext) -> bool;

    fn remove_mounting_system(&mut self, ctx: &SystemContext) -> bool;

    fn warrior(&mut self) -> Result<&mut WarriorContext, Error> {
        Err(Error::BattleUnexpectedSystemContext)
    }

    fn enemy(&mut self) -> Result<&mut EnemyContext, Error> {
        Err(Error::BattleUnexpectedSystemContext)
    }

    fn card(&mut self) -> Result<&mut CardContext, Error> {
        Err(Error::BattleUnexpectedSystemContext)
    }
}

fn add_mounting_system_internal(
    ctx: &SystemContext,
    mounting_systems: &mut Vec<SystemContext>,
) -> bool {
    if !ctx.is_durable() {
        return false;
    }
    let Some(exist_ctx) = mounting_systems.iter_mut().find(|v| v.equal(&ctx)) else {
        return false;
    };
    if !exist_ctx.durable_combine(ctx) {
        mounting_systems.push(ctx.clone());
        return false;
    }
    true
}

fn update_mounting_system_internal(
    ctx: &SystemContext,
    mounting_systems: &mut Vec<SystemContext>,
) -> bool {
    if !ctx.is_durable() {
        return false;
    }
    let Some(exist_ctx) = mounting_systems.iter_mut().find(|v| v.equal(&ctx)) else {
        return false;
    };
    exist_ctx.durable_update(ctx)
}

fn remove_mounting_system_internal(
    ctx: &SystemContext,
    mounting_systems: &mut Vec<SystemContext>,
) -> bool {
    let mut index = None;
    for (i, v) in mounting_systems.iter().enumerate() {
        if v.equal(&ctx) {
            index = Some(i);
            break;
        }
    }
    if let Some(index) = index {
        mounting_systems.remove(index);
        true
    } else {
        false
    }
}
