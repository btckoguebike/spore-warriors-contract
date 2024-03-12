extern crate alloc;
use rand::RngCore;
use spore_warriors_generated as generated;

use crate::battle::traits::FightLog;
use crate::contexts::{CardContext, ContextType, CtxAdaptor, SystemContext};
use crate::errors::Error;
use crate::game::SporeRng;
use crate::systems::applications::{
    armor_up_apply, attack_apply, attack_power_weak_apply, defense_power_weak_apply,
    draw_count_down_apply, draw_count_up_apply, healing_apply, max_hp_down_apply, max_hp_up_apply,
    power_cost_down_apply, shield_up_apply,
};
use crate::systems::{Command, SystemInput, SystemReturn};
use crate::wrappings::{Card, Value};
use crate::{apply_system, filter_objects};

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

// normally inflict single damage
pub fn attack(
    _: &generated::ResourcePool,
    _: &mut SporeRng,
    ctx: SystemContext,
    _: usize,
    targets: Vec<usize>,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut logs = vec![];
    let mut iter = ctx.system.args.iter();
    let objects = filter_objects!(objects, targets);
    apply_system!(logs, iter, input, objects, u16, attack_apply);
    Ok(SystemReturn::Continue(vec![Command::AddLogs(logs)]))
}

// normally inflict multiple damage
pub fn multiple_attack(
    _: &generated::ResourcePool,
    _: &mut SporeRng,
    ctx: SystemContext,
    _: usize,
    targets: Vec<usize>,
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
    let objects = filter_objects!(objects, targets);
    for object in objects {
        (0..*count)
            .map(|_| attack_apply(&mut logs, *damage, object))
            .collect::<Result<_, _>>()?;
    }
    Ok(SystemReturn::Continue(vec![Command::AddLogs(logs)]))
}

// normally increase DEF
pub fn armor_up(
    _: &generated::ResourcePool,
    _: &mut SporeRng,
    ctx: SystemContext,
    _: usize,
    targets: Vec<usize>,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut logs = vec![];
    let mut iter = ctx.system.args.iter();
    let objects = filter_objects!(objects, targets);
    apply_system!(logs, iter, input, objects, u16, armor_up_apply);
    Ok(SystemReturn::Continue(vec![Command::AddLogs(logs)]))
}

// normally decrease DEF
pub fn armor_down(
    _: &generated::ResourcePool,
    _: &mut SporeRng,
    ctx: SystemContext,
    _: usize,
    targets: Vec<usize>,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut logs = vec![];
    let mut iter = ctx.system.args.iter();
    let objects = filter_objects!(objects, targets);
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
    Ok(SystemReturn::Continue(vec![Command::AddLogs(logs)]))
}

// normally increase SHD
pub fn shield_up(
    _: &generated::ResourcePool,
    _: &mut SporeRng,
    ctx: SystemContext,
    _: usize,
    targets: Vec<usize>,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut logs = vec![];
    let mut iter = ctx.system.args.iter();
    let objects = filter_objects!(objects, targets);
    apply_system!(logs, iter, input, objects, u16, shield_up_apply);
    Ok(SystemReturn::Continue(vec![Command::AddLogs(logs)]))
}

// normally decrease SHD
pub fn shield_down(
    _: &generated::ResourcePool,
    _: &mut SporeRng,
    ctx: SystemContext,
    _: usize,
    targets: Vec<usize>,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut logs = vec![];
    let mut iter = ctx.system.args.iter();
    let objects = filter_objects!(objects, targets);
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
    Ok(SystemReturn::Continue(vec![Command::AddLogs(logs)]))
}

// normally cure object's hp
pub fn healing(
    _: &generated::ResourcePool,
    _: &mut SporeRng,
    ctx: SystemContext,
    _: usize,
    targets: Vec<usize>,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut logs = vec![];
    let mut iter = ctx.system.args.iter();
    let objects = filter_objects!(objects, targets);
    apply_system!(logs, iter, input, objects, u16, healing_apply);
    Ok(SystemReturn::Continue(vec![Command::AddLogs(logs)]))
}

// normally increase the drawn cards count
pub fn draw_count_up(
    _: &generated::ResourcePool,
    _: &mut SporeRng,
    ctx: SystemContext,
    _: usize,
    targets: Vec<usize>,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut logs = vec![];
    let mut iter = ctx.system.args.iter();
    let objects = filter_objects!(objects, targets);
    apply_system!(logs, iter, input, objects, u8, draw_count_up_apply);
    Ok(SystemReturn::Continue(vec![Command::AddLogs(logs)]))
}

// normally increase the drawn cards count
pub fn draw_count_down(
    _: &generated::ResourcePool,
    _: &mut SporeRng,
    ctx: SystemContext,
    _: usize,
    targets: Vec<usize>,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut logs = vec![];
    let mut iter = ctx.system.args.iter();
    let objects = filter_objects!(objects, targets);
    apply_system!(logs, iter, input, objects, u8, draw_count_down_apply);
    Ok(SystemReturn::Continue(vec![Command::AddLogs(logs)]))
}

// simplely increase max hp value
pub fn max_hp_up(
    _: &generated::ResourcePool,
    _: &mut SporeRng,
    ctx: SystemContext,
    _: usize,
    targets: Vec<usize>,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut logs = vec![];
    let mut iter = ctx.system.args.iter();
    let objects = filter_objects!(objects, targets);
    apply_system!(logs, iter, input, objects, u16, max_hp_up_apply);
    Ok(SystemReturn::Continue(vec![Command::AddLogs(logs)]))
}

// simplely decrease max hp value
pub fn max_hp_down(
    _: &generated::ResourcePool,
    _: &mut SporeRng,
    ctx: SystemContext,
    _: usize,
    targets: Vec<usize>,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut logs = vec![];
    let mut iter = ctx.system.args.iter();
    let objects = filter_objects!(objects, targets);
    apply_system!(logs, iter, input, objects, u16, max_hp_down_apply);
    Ok(SystemReturn::Continue(vec![Command::AddLogs(logs)]))
}

// normally increase ATK, which impacts object's damage
pub fn attack_up(
    _: &generated::ResourcePool,
    _: &mut SporeRng,
    ctx: SystemContext,
    _: usize,
    targets: Vec<usize>,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut logs = vec![];
    let mut iter = ctx.system.args.iter();
    let objects = filter_objects!(objects, targets);
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

// normally decrease ATK
pub fn attack_down(
    _: &generated::ResourcePool,
    _: &mut SporeRng,
    ctx: SystemContext,
    _: usize,
    targets: Vec<usize>,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut logs = vec![];
    let mut iter = ctx.system.args.iter();
    let objects = filter_objects!(objects, targets);
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

// normally increase DEF, which impacts object's sheild and defense
pub fn defense_up(
    _: &generated::ResourcePool,
    _: &mut SporeRng,
    ctx: SystemContext,
    _: usize,
    targets: Vec<usize>,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut logs = vec![];
    let mut iter = ctx.system.args.iter();
    let objects = filter_objects!(objects, targets);
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
    Ok(SystemReturn::Continue(vec![Command::AddLogs(logs)]))
}

// normally decrease DEF
pub fn defense_down(
    _: &generated::ResourcePool,
    _: &mut SporeRng,
    ctx: SystemContext,
    _: usize,
    targets: Vec<usize>,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut logs = vec![];
    let mut iter = ctx.system.args.iter();
    let objects = filter_objects!(objects, targets);
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
    Ok(SystemReturn::Continue(vec![Command::AddLogs(logs)]))
}

// normally increase ATK_WEAK value
pub fn attack_weak_up(
    _: &generated::ResourcePool,
    _: &mut SporeRng,
    ctx: SystemContext,
    _: usize,
    targets: Vec<usize>,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut logs = vec![];
    let mut iter = ctx.system.args.iter();
    let objects = filter_objects!(objects, targets);
    apply_system!(logs, iter, input, objects, u8, attack_power_weak_apply);
    Ok(SystemReturn::Continue(vec![Command::AddLogs(logs)]))
}

// normally decrease ATK_WEAK value
pub fn attack_weak_down(
    _: &generated::ResourcePool,
    _: &mut SporeRng,
    ctx: SystemContext,
    _: usize,
    targets: Vec<usize>,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut logs = vec![];
    let mut iter = ctx.system.args.iter();
    let objects = filter_objects!(objects, targets);
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
    Ok(SystemReturn::Continue(vec![Command::AddLogs(logs)]))
}

// normally increase DEF_WEAK value
pub fn defense_weak_up(
    _: &generated::ResourcePool,
    _: &mut SporeRng,
    ctx: SystemContext,
    _: usize,
    targets: Vec<usize>,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut logs = vec![];
    let mut iter = ctx.system.args.iter();
    let objects = filter_objects!(objects, targets);
    apply_system!(logs, iter, input, objects, u8, defense_power_weak_apply);
    Ok(SystemReturn::Continue(vec![Command::AddLogs(logs)]))
}

// normally decrease DEF_WEAK value
pub fn defense_weak_down(
    _: &generated::ResourcePool,
    _: &mut SporeRng,
    ctx: SystemContext,
    _: usize,
    targets: Vec<usize>,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut logs = vec![];
    let mut iter = ctx.system.args.iter();
    let objects = filter_objects!(objects, targets);
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
    Ok(SystemReturn::Continue(vec![Command::AddLogs(logs)]))
}

// just draw cards from main deck
pub fn draw_cards(
    _: &generated::ResourcePool,
    _: &mut SporeRng,
    ctx: SystemContext,
    _: usize,
    _: Vec<usize>,
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
    Ok(SystemReturn::Continue(vec![Command::DrawCards(
        *count as u8,
    )]))
}

// select cards randomly from particular deck, no extensions
pub fn draw_select_cards(
    resource_pool: &generated::ResourcePool,
    rng: &mut SporeRng,
    ctx: SystemContext,
    _: usize,
    targets: Vec<usize>,
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
    let objects = filter_objects!(objects, targets);
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

// discard from hand deck randomly, no extensions
pub fn discard_cards(
    _: &generated::ResourcePool,
    _: &mut SporeRng,
    ctx: SystemContext,
    _: usize,
    _: Vec<usize>,
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
    let commands = vec![Command::DiscardHandCards(*count as u8, true)];
    Ok(SystemReturn::Continue(commands))
}

// discard from hand deck, no extensions
pub fn discard_select_cards(
    _: &generated::ResourcePool,
    _: &mut SporeRng,
    ctx: SystemContext,
    _: usize,
    _: Vec<usize>,
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

// decrease card power cost only in battle
pub fn card_power_cost_down(
    _: &generated::ResourcePool,
    _: &mut SporeRng,
    ctx: SystemContext,
    _: usize,
    targets: Vec<usize>,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    let mut logs = vec![];
    let mut iter = ctx.system.args.iter();
    let objects = filter_objects!(objects, targets);
    apply_system!(logs, iter, input, objects, u8, power_cost_down_apply);
    Ok(SystemReturn::Continue(vec![Command::AddLogs(logs)]))
}

// randomly select card in hand and change its power cost
pub fn change_random_card_cost(
    _: &generated::ResourcePool,
    rng: &mut SporeRng,
    ctx: SystemContext,
    _: usize,
    targets: Vec<usize>,
    objects: &mut [&mut dyn CtxAdaptor],
    input: Option<SystemInput>,
) -> Result<SystemReturn, Error> {
    if let Some(SystemInput::Trigger(FightLog::GameOver)) = input {
        return Ok(SystemReturn::Continue(vec![]));
    }
    let mut iter = ctx.system.args.iter();
    let Some(Value(cost)) = iter.next() else {
        return Err(Error::BattleUnexpectedSystemArgs);
    };
    let mut logs = vec![];
    let objects = filter_objects!(objects, targets);
    for object in objects {
        match object.context_type() {
            ContextType::Warrior => {
                let warrior = object.warrior()?;
                if warrior.hand_deck.is_empty() {
                    continue;
                }
                let card_index = rng.next_u32() as usize % warrior.hand_deck.len();
                let card = warrior.hand_deck.get_mut(card_index).unwrap();
                card.power_cost = *cost as u8;
            }
            ContextType::Enemy | ContextType::Card => continue,
        }
        logs.push(FightLog::SystemPowerCostChange(
            object.offset(),
            *cost as u8,
        ));
    }
    Ok(SystemReturn::Continue(vec![Command::AddLogs(logs)]))
}
