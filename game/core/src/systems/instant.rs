extern crate alloc;
use core::slice::Iter;
use rand::RngCore;
use spore_warriors_generated as generated;

use crate::apply_system;
use crate::battle::traits::FightLog;
use crate::contexts::{CardContext, ContextType, CtxAdaptor, SystemContext};
use crate::errors::Error;
use crate::game::SporeRng;
use crate::systems::applications::{
    armor_up_apply, attack_apply, attack_power_weak_apply, defense_power_weak_apply,
    draw_count_down_apply, draw_count_up_apply, healing_apply, max_hp_down_apply, max_hp_up_apply,
    shield_up_apply,
};
use crate::systems::{Command, SystemInput, SystemReturn};
use crate::wrappings::{Card, System, Value};

enum DeckType {
    FromDeck,
    FromGrave,
    FromResource,
}

impl TryFrom<&u16> for DeckType {
    type Error = Error;

    fn try_from(value: &u16) -> Result<Self, Self::Error> {
        match value {
            &0u16 => Ok(Self::FromDeck),
            &1u16 => Ok(Self::FromGrave),
            &2u16 => Ok(Self::FromResource),
            _ => Err(Error::BattleUnexpectedDeckType),
        }
    }
}

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
    let mut logs = vec![];
    let mut iter = ctx.system.args.iter();
    apply_system!(
        logs,
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
    let mut logs = vec![];
    let mut iter = ctx.system.args.iter();
    apply_system!(
        logs,
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

// simplely increase max hp value
pub fn max_hp_up(
    resource_pool: &generated::ResourcePool,
    rng: &mut SporeRng,
    ctx: SystemContext,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut logs = vec![];
    let mut iter = ctx.system.args.iter();
    apply_system!(logs, iter, input, objects, u16, max_hp_up_apply);
    check_extensions(iter, resource_pool, rng, vec![Command::AddLogs(logs)])
}

// simplely decrease max hp value
pub fn max_hp_down(
    resource_pool: &generated::ResourcePool,
    rng: &mut SporeRng,
    ctx: SystemContext,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut logs = vec![];
    let mut iter = ctx.system.args.iter();
    apply_system!(logs, iter, input, objects, u16, max_hp_down_apply);
    check_extensions(iter, resource_pool, rng, vec![Command::AddLogs(logs)])
}

// normally increase ATK, which impacts object's damage
pub fn attack_up(
    resource_pool: &generated::ResourcePool,
    rng: &mut SporeRng,
    ctx: SystemContext,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut logs = vec![];
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
    check_extensions(iter, resource_pool, rng, vec![Command::AddLogs(logs)])
}

// normally decrease ATK
pub fn attack_down(
    resource_pool: &generated::ResourcePool,
    rng: &mut SporeRng,
    ctx: SystemContext,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut logs = vec![];
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
    check_extensions(iter, resource_pool, rng, vec![Command::AddLogs(logs)])
}

// normally increase DEF, which impacts object's sheild and defense
pub fn defense_up(
    resource_pool: &generated::ResourcePool,
    rng: &mut SporeRng,
    ctx: SystemContext,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut logs = vec![];
    let mut iter = ctx.system.args.iter();
    apply_system!(
        logs,
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

// normally decrease DEF
pub fn defense_down(
    resource_pool: &generated::ResourcePool,
    rng: &mut SporeRng,
    ctx: SystemContext,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut logs = vec![];
    let mut iter = ctx.system.args.iter();
    apply_system!(
        logs,
        iter,
        input,
        objects,
        defense,
        u8,
        saturating_sub,
        SystemDefensePowerDown
    );
    check_extensions(iter, resource_pool, rng, vec![Command::AddLogs(logs)])
}

// normally increase ATK_WEAK value
pub fn attack_weak_up(
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

// normally decrease ATK_WEAK value
pub fn attack_weak_down(
    resource_pool: &generated::ResourcePool,
    rng: &mut SporeRng,
    ctx: SystemContext,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut logs = vec![];
    let mut iter = ctx.system.args.iter();
    apply_system!(
        logs,
        iter,
        input,
        objects,
        attack_weak,
        u8,
        saturating_sub,
        SystemAttackWeakDown
    );
    check_extensions(iter, resource_pool, rng, vec![Command::AddLogs(logs)])
}

// normally increase DEF_WEAK value
pub fn defense_weak_up(
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

// normally decrease DEF_WEAK value
pub fn defense_weak_down(
    resource_pool: &generated::ResourcePool,
    rng: &mut SporeRng,
    ctx: SystemContext,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut logs = vec![];
    let mut iter = ctx.system.args.iter();
    apply_system!(
        logs,
        iter,
        input,
        objects,
        defense_weak,
        u8,
        saturating_sub,
        SystemDefenseWeakDown
    );
    check_extensions(iter, resource_pool, rng, vec![Command::AddLogs(logs)])
}

// just draw cards from main deck
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

// select cards randomly from particular deck, no extensions
pub fn draw_select_cards(
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
    let (Some(Value(deck_type)), Some(Value(select_count))) = (iter.next(), iter.next()) else {
        return Err(Error::BattleUnexpectedSystemArgs);
    };
    let mut resource_pick_info = None;
    if let (Some(Value(card_class)), Some(Value(pick_count))) = (iter.next(), iter.next()) {
        if pick_count < select_count {
            return Err(Error::ResourceBrokenCardSelection);
        }
        resource_pick_info = Some((*card_class, *pick_count));
    }
    for object in objects {
        match object.context_type() {
            ContextType::Warrior => {
                let warrior = object.warrior()?;
                match deck_type.try_into()? {
                    DeckType::FromDeck => {
                        warrior.card_selection.selection_pool =
                            warrior.deck.iter().map(|v| v.offset()).collect::<Vec<_>>();
                    }
                    DeckType::FromGrave => {
                        warrior.card_selection.selection_pool = warrior
                            .grave_deck
                            .iter()
                            .map(|v| v.offset())
                            .collect::<Vec<_>>();
                    }
                    DeckType::FromResource => {
                        let Some((card_class, pick_count)) = resource_pick_info else {
                            return Err(Error::BattleUnexpectedSystemArgs);
                        };
                        let mut avaliable_cards = resource_pool
                            .card_pool()
                            .into_iter()
                            .filter(|card| u8::from(card.class()) == card_class as u8)
                            .collect::<Vec<_>>();
                        if pick_count as usize > avaliable_cards.len() {
                            return Err(Error::BattleUnexpectedSystemArgs);
                        }
                        let mut pick_cards = (0..pick_count)
                            .into_iter()
                            .map(|_| {
                                let card_index = rng.next_u32() as usize % avaliable_cards.len();
                                let card = avaliable_cards.remove(card_index);
                                Ok(CardContext::new(Card::randomized(
                                    resource_pool,
                                    card,
                                    rng,
                                )?))
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        warrior.card_selection.selection_pool =
                            pick_cards.iter().map(|v| v.offset()).collect::<Vec<_>>();
                        warrior
                            .card_selection
                            .unbelonging_deck
                            .append(&mut pick_cards);
                    }
                }
            }
            ContextType::Enemy | ContextType::Card => continue,
        }
    }
    Ok(SystemReturn::RequireCardSelect(
        *select_count as u8,
        true,
        vec![],
    ))
}

// discard from hand deck, no extensions
pub fn discard_select_cards(
    _: &generated::ResourcePool,
    _: &mut SporeRng,
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
    Ok(SystemReturn::RequireCardSelect(*count as u8, false, vec![]))
}

pub fn discard_random_cards(
    _: &generated::ResourcePool,
    _: &mut SporeRng,
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
    Ok(SystemReturn::Continue(vec![Command::DiscardHandCards(
        *count as u8,
        true,
    )]))
}
