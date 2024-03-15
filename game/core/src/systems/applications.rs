extern crate alloc;
use alloc::vec::Vec;
use core::cmp::min;

use crate::battle::traits::FightLog;
use crate::contexts::{ContextType, CtxAdaptor};
use crate::errors::Error;
use crate::systems::MAX_WEAK_COUNT;

pub fn attack_apply(
    logs: &mut Vec<FightLog>,
    mut value: u16,
    object: &mut &mut dyn CtxAdaptor,
) -> Result<(), Error> {
    let damage = value;
    match object.context_type() {
        ContextType::Warrior => {
            let warrior = object.warrior()?;
            value += warrior.attack as u16;
            value *= ((MAX_WEAK_COUNT - warrior.attack_weak) / MAX_WEAK_COUNT) as u16;
            if value > warrior.shield {
                value = value - warrior.shield;
                warrior.shield = 0;
                if value > warrior.armor {
                    value = value - warrior.armor;
                    warrior.armor = 0;
                    warrior.hp = warrior.hp.saturating_sub(value);
                } else {
                    warrior.armor -= value;
                }
            } else {
                warrior.shield -= value;
            }
        }
        ContextType::Enemy => {
            let enemy = object.enemy()?;
            value += enemy.attack as u16;
            value *= ((MAX_WEAK_COUNT - enemy.attack_weak) / MAX_WEAK_COUNT) as u16;
            if value > enemy.shield {
                value = value - enemy.shield;
                enemy.shield = 0;
                if value > enemy.armor {
                    value = value - enemy.armor;
                    enemy.armor = 0;
                    enemy.hp = enemy.hp.saturating_sub(value);
                } else {
                    enemy.armor -= value;
                }
            } else {
                enemy.shield -= value;
            }
        }
        ContextType::Card => return Ok(()),
    };
    logs.push(FightLog::SystemDamage(object.offset(), damage));
    Ok(())
}

pub fn healing_apply(
    logs: &mut Vec<FightLog>,
    mut value: u16,
    object: &mut &mut dyn CtxAdaptor,
) -> Result<(), Error> {
    match object.context_type() {
        ContextType::Warrior => {
            let warrior = object.warrior()?;
            value = min(value, warrior.max_hp - warrior.hp);
            warrior.hp += value;
        }
        ContextType::Enemy => {
            let enemy = object.enemy()?;
            value = min(value, enemy.enemy.hp - enemy.hp);
            enemy.hp += value;
        }
        ContextType::Card => return Ok(()),
    }
    logs.push(FightLog::SystemRecoverHp(object.offset(), value));
    Ok(())
}

pub fn armor_up_apply(
    logs: &mut Vec<FightLog>,
    mut value: u16,
    object: &mut &mut dyn CtxAdaptor,
) -> Result<(), Error> {
    match object.context_type() {
        ContextType::Warrior => {
            let warrior = object.warrior()?;
            value += warrior.defense as u16;
            value *= ((MAX_WEAK_COUNT - warrior.defense_weak) / MAX_WEAK_COUNT) as u16;
            warrior.armor = warrior.armor.saturating_add(value);
        }
        ContextType::Enemy => {
            let enemy = object.enemy()?;
            value += enemy.defense as u16;
            value *= ((MAX_WEAK_COUNT - enemy.defense_weak) / MAX_WEAK_COUNT) as u16;
            enemy.armor = enemy.armor.saturating_add(value);
        }
        ContextType::Card => return Ok(()),
    };
    logs.push(FightLog::SystemArmorUp(object.offset(), value));
    Ok(())
}

pub fn shield_up_apply(
    logs: &mut Vec<FightLog>,
    mut value: u16,
    object: &mut &mut dyn CtxAdaptor,
) -> Result<(), Error> {
    match object.context_type() {
        ContextType::Warrior => {
            let warrior = object.warrior()?;
            value += warrior.shield as u16;
            value *= ((MAX_WEAK_COUNT - warrior.defense_weak) / MAX_WEAK_COUNT) as u16;
            warrior.shield = warrior.shield.saturating_add(value);
        }
        ContextType::Enemy => {
            let enemy = object.enemy()?;
            value += enemy.shield as u16;
            value *= ((MAX_WEAK_COUNT - enemy.defense_weak) / MAX_WEAK_COUNT) as u16;
            enemy.shield = enemy.shield.saturating_add(value);
        }
        ContextType::Card => return Ok(()),
    };
    logs.push(FightLog::SystemShieldUp(object.offset(), value));
    Ok(())
}

pub fn attack_power_weak_apply(
    logs: &mut Vec<FightLog>,
    mut value: u8,
    object: &mut &mut dyn CtxAdaptor,
) -> Result<(), Error> {
    match object.context_type() {
        ContextType::Warrior => {
            let warrior = object.warrior()?;
            warrior.attack_weak = warrior.attack_weak.saturating_add(value);
            if warrior.attack_weak > MAX_WEAK_COUNT {
                value = MAX_WEAK_COUNT - warrior.attack_weak;
                warrior.attack_weak = MAX_WEAK_COUNT;
            }
        }
        ContextType::Enemy => {
            let enemy = object.enemy()?;
            enemy.attack_weak = enemy.attack_weak.saturating_add(value);
            if enemy.attack_weak > MAX_WEAK_COUNT {
                value = MAX_WEAK_COUNT - enemy.attack_weak;
                enemy.attack_weak = MAX_WEAK_COUNT;
            }
        }
        ContextType::Card => return Ok(()),
    }
    logs.push(FightLog::SystemAttackWeakUp(object.offset(), value));
    Ok(())
}

pub fn defense_power_weak_apply(
    logs: &mut Vec<FightLog>,
    mut value: u8,
    object: &mut &mut dyn CtxAdaptor,
) -> Result<(), Error> {
    match object.context_type() {
        ContextType::Warrior => {
            let warrior = object.warrior()?;
            warrior.defense_weak = warrior.defense_weak.saturating_add(value);
            if warrior.defense_weak > MAX_WEAK_COUNT {
                value = MAX_WEAK_COUNT - warrior.defense_weak;
                warrior.defense_weak = MAX_WEAK_COUNT;
            }
        }
        ContextType::Enemy => {
            let enemy = object.enemy()?;
            enemy.defense_weak = enemy.defense_weak.saturating_add(value);
            if enemy.defense_weak > MAX_WEAK_COUNT {
                value = MAX_WEAK_COUNT - enemy.defense_weak;
                enemy.defense_weak = MAX_WEAK_COUNT;
            }
        }
        ContextType::Card => return Ok(()),
    }
    logs.push(FightLog::SystemDefenseWeakUp(object.offset(), value));
    Ok(())
}

pub fn draw_count_up_apply(
    logs: &mut Vec<FightLog>,
    value: u8,
    object: &mut &mut dyn CtxAdaptor,
) -> Result<(), Error> {
    match object.context_type() {
        ContextType::Warrior => {
            let warrior = object.warrior()?;
            warrior.draw_count = warrior.draw_count.saturating_add(value);
        }
        ContextType::Enemy | ContextType::Card => return Ok(()),
    };
    logs.push(FightLog::SystemDrawCountUp(value));
    Ok(())
}

pub fn draw_count_down_apply(
    logs: &mut Vec<FightLog>,
    value: u8,
    object: &mut &mut dyn CtxAdaptor,
) -> Result<(), Error> {
    match object.context_type() {
        ContextType::Warrior => {
            let warrior = object.warrior()?;
            warrior.draw_count = warrior.draw_count.saturating_sub(value);
        }
        ContextType::Enemy | ContextType::Card => return Ok(()),
    };
    logs.push(FightLog::SystemDrawCountDown(value));
    Ok(())
}

pub fn max_hp_up_apply(
    logs: &mut Vec<FightLog>,
    mut value: u16,
    object: &mut &mut dyn CtxAdaptor,
) -> Result<(), Error> {
    match object.context_type() {
        ContextType::Warrior => {
            let warrior = object.warrior()?;
            let new_max_hp = warrior.max_hp.saturating_add(value);
            value = new_max_hp - warrior.max_hp;
            warrior.max_hp += value;
            warrior.hp = min(warrior.hp + value, warrior.max_hp);
        }
        ContextType::Enemy | ContextType::Card => return Ok(()),
    }
    logs.push(FightLog::SystemMaxHpUp(value));
    Ok(())
}

pub fn max_hp_down_apply(
    logs: &mut Vec<FightLog>,
    mut value: u16,
    object: &mut &mut dyn CtxAdaptor,
) -> Result<(), Error> {
    match object.context_type() {
        ContextType::Warrior => {
            let warrior = object.warrior()?;
            let new_max_hp = warrior.max_hp.saturating_sub(value);
            value = warrior.max_hp - new_max_hp;
            warrior.max_hp -= value;
            warrior.hp = warrior.hp.saturating_sub(value);
        }
        ContextType::Enemy | ContextType::Card => return Ok(()),
    }
    logs.push(FightLog::SystemMaxHpDown(value));
    Ok(())
}

pub fn power_cost_down_apply(
    logs: &mut Vec<FightLog>,
    mut value: u8,
    object: &mut &mut dyn CtxAdaptor,
) -> Result<(), Error> {
    match object.context_type() {
        ContextType::Card => {
            let card = object.card()?;
            card.power_cost = card.power_cost.saturating_sub(value);
            value = card.power_cost;
        }
        ContextType::Warrior | ContextType::Enemy => return Ok(()),
    }
    logs.push(FightLog::SystemPowerCostChange(object.offset(), value));
    Ok(())
}
