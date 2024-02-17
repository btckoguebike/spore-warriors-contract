extern crate alloc;
use alloc::vec;
use core::cmp::{max, min};
use rand::RngCore;

use crate::errors::Error;
use crate::wrappings::{Card, Effect, Enemy, Item, ItemClass, Warrior};

macro_rules! change_value {
    ($this:ident.$field:ident, $diff:ident, $conv:ty, $ori:ty) => {{
        let value = max($this.$field as $conv + $diff as $conv, 0);
        let value = min(value, <$ori>::MAX as $conv);
        $this.$field = value as $ori;
    }};
}

pub trait CtxAdaptor {
    fn offset(&self) -> usize;
    fn change_hp(&mut self, diff: i16);
    fn change_armor(&mut self, diff: i8);
    fn change_shield(&mut self, diff: i8);
    fn change_attack(&mut self, diff: i8);
    fn change_attack_weak(&mut self, diff: i8);
    fn change_defense(&mut self, diff: i8);
    fn change_defense_weak(&mut self, diff: i8);
    fn change_power(&mut self, diff: i8);
    fn change_draw_count(&mut self, diff: i8);
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct EnemySnapshot {
    pub id: u16,
    pub offset: usize,
    pub hp: u16,
    pub armor: u8,
    pub shield: u8,
    pub attack: u8,
    pub attack_weak: u8,
    pub defense: u8,
    pub defense_weak: u8,
    pub pending_effects: Vec<u16>,
}

pub struct EnemyContext<'e> {
    pub enemy: &'e Enemy,
    pub offset: usize,
    pub hp: u16,
    pub armor: u8,
    pub shield: u8,
    pub attack: u8,
    pub attack_weak: u8,
    pub defense: u8,
    pub defense_weak: u8,
    pub strategy: Vec<Vec<&'e Effect>>,
    pub pending_effects: Vec<&'e Effect>,
}

impl<'e> EnemyContext<'e> {
    pub fn new(enemy: &'e Enemy, offset: usize) -> Self {
        Self {
            enemy,
            offset,
            hp: enemy.hp,
            armor: enemy.armor,
            shield: enemy.shield,
            attack: enemy.attack,
            attack_weak: enemy.attack_weak,
            defense: enemy.defense,
            defense_weak: enemy.defense_weak,
            strategy: vec![],
            pending_effects: vec![],
        }
    }

    pub fn pop_action(&mut self, rng: &mut impl RngCore) -> Result<Vec<&'e Effect>, Error> {
        if self.strategy.is_empty() {
            self.reset_strategy(rng);
        }
        if self.strategy.is_empty() {
            return Err(Error::ResourceBrokenEnemyStrategy);
        }
        Ok(self.strategy.remove(0))
    }

    pub fn reset_strategy(&mut self, rng: &mut impl RngCore) {
        let mut randomized_actions = self.enemy.strategy.actions.iter().collect::<Vec<_>>();
        if self.enemy.strategy.random_select {
            let mut actions = randomized_actions.drain(..).collect::<Vec<_>>();
            while !actions.is_empty() {
                let offset = rng.next_u32() as usize % actions.len();
                randomized_actions.push(actions.remove(offset));
            }
        }
        self.strategy.clear();
        randomized_actions.into_iter().for_each(|action| {
            let mut randomized_effects = action.effect_pool.iter().collect::<Vec<_>>();
            if action.random_select {
                let mut effects = randomized_effects.drain(..).collect::<Vec<_>>();
                while !effects.is_empty() {
                    let offset = rng.next_u32() as usize % effects.len();
                    randomized_effects.push(effects.remove(offset));
                }
            }
            self.strategy.push(randomized_effects);
        });
    }

    pub fn snapshot(&self) -> EnemySnapshot {
        EnemySnapshot {
            id: self.enemy.id,
            offset: self.offset,
            hp: self.hp,
            armor: self.armor,
            shield: self.shield,
            attack: self.attack,
            attack_weak: self.attack_weak,
            defense: self.defense,
            defense_weak: self.defense_weak,
            pending_effects: self.pending_effects.iter().map(|v| v.id).collect(),
        }
    }
}

impl<'e> CtxAdaptor for EnemyContext<'e> {
    fn offset(&self) -> usize {
        self.offset
    }

    fn change_hp(&mut self, diff: i16) {
        change_value!(self.hp, diff, i32, u16);
    }

    fn change_armor(&mut self, diff: i8) {
        change_value!(self.armor, diff, i16, u8);
    }

    fn change_shield(&mut self, diff: i8) {
        change_value!(self.shield, diff, i16, u8);
    }

    fn change_attack(&mut self, diff: i8) {
        change_value!(self.attack, diff, i16, u8);
    }

    fn change_attack_weak(&mut self, diff: i8) {
        change_value!(self.attack_weak, diff, i16, u8);
    }

    fn change_defense(&mut self, diff: i8) {
        change_value!(self.defense, diff, i16, u8);
    }

    fn change_defense_weak(&mut self, diff: i8) {
        change_value!(self.defense_weak, diff, i16, u8);
    }

    fn change_power(&mut self, _diff: i8) {
        unimplemented!("power adaptor")
    }

    fn change_draw_count(&mut self, _diff: i8) {
        unimplemented!("draw_count adaptor")
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct WarriorSnapshot {
    pub id: u16,
    pub offset: usize,
    pub hp: u16,
    pub power: u8,
    pub armor: u8,
    pub shield: u8,
    pub attack: u8,
    pub attack_weak: u8,
    pub defense: u8,
    pub defense_weak: u8,
    pub draw_count: u8,
    pub special_card: u16,
    pub props_list: Vec<u16>,
    pub hand_deck: Vec<u16>,
    pub deck: Vec<u16>,
    pub grave_deck: Vec<u16>,
    pub pending_deck: Vec<u16>,
    pub pending_effects: Vec<u16>,
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct WarriorContext<'w> {
    pub warrior: &'w Warrior,
    pub offset: usize,
    pub hp: u16,
    pub power: u8,
    pub armor: u8,
    pub shield: u8,
    pub attack: u8,
    pub attack_weak: u8,
    pub defense: u8,
    pub defense_weak: u8,
    pub draw_count: u8,
    pub special_card: &'w Card,
    pub props_list: Vec<&'w Item>,
    pub hand_deck: Vec<&'w Card>,
    pub deck: Vec<&'w Card>,
    pub grave_deck: Vec<&'w Card>,
    pub pending_deck: Vec<&'w Card>,
    pub pending_effects: Vec<&'w Effect>,
}

impl<'w> WarriorContext<'w> {
    pub fn new(warrior: &'w Warrior) -> Self {
        Self {
            warrior,
            offset: 0,
            hp: warrior.hp,
            power: warrior.power,
            armor: warrior.armor,
            shield: warrior.shield,
            attack: warrior.attack,
            attack_weak: warrior.attack_weak,
            defense: warrior.defense,
            defense_weak: warrior.defense_weak,
            draw_count: warrior.draw_count,
            special_card: &warrior.charactor_card,
            props_list: warrior
                .package_status
                .iter()
                .filter(|v| v.class == ItemClass::Props)
                .collect(),
            deck: warrior.deck_status.iter().collect(),
            hand_deck: vec![],
            grave_deck: vec![],
            pending_deck: vec![],
            pending_effects: vec![],
        }
    }

    pub fn snapshot(&self) -> WarriorSnapshot {
        WarriorSnapshot {
            id: self.warrior.id,
            offset: self.offset,
            hp: self.hp,
            power: self.power,
            armor: self.armor,
            shield: self.shield,
            attack: self.attack,
            attack_weak: self.attack_weak,
            defense: self.defense,
            defense_weak: self.defense_weak,
            draw_count: self.draw_count,
            special_card: self.special_card.id,
            props_list: self.props_list.iter().map(|v| v.id).collect(),
            deck: self.deck.iter().map(|v| v.id).collect(),
            hand_deck: self.hand_deck.iter().map(|v| v.id).collect(),
            grave_deck: self.grave_deck.iter().map(|v| v.id).collect(),
            pending_deck: self.pending_deck.iter().map(|v| v.id).collect(),
            pending_effects: self.pending_effects.iter().map(|v| v.id).collect(),
        }
    }
}

impl<'p> CtxAdaptor for WarriorContext<'p> {
    fn offset(&self) -> usize {
        self.offset
    }

    fn change_hp(&mut self, diff: i16) {
        change_value!(self.hp, diff, i32, u16);
    }

    fn change_armor(&mut self, diff: i8) {
        change_value!(self.armor, diff, i16, u8);
    }

    fn change_shield(&mut self, diff: i8) {
        change_value!(self.shield, diff, i16, u8);
    }

    fn change_attack(&mut self, diff: i8) {
        change_value!(self.attack, diff, i16, u8);
    }

    fn change_attack_weak(&mut self, diff: i8) {
        change_value!(self.attack_weak, diff, i16, u8);
    }

    fn change_defense(&mut self, diff: i8) {
        change_value!(self.defense, diff, i16, u8);
    }

    fn change_defense_weak(&mut self, diff: i8) {
        change_value!(self.defense_weak, diff, i16, u8);
    }

    fn change_power(&mut self, diff: i8) {
        change_value!(self.power, diff, i16, u8);
    }

    fn change_draw_count(&mut self, diff: i8) {
        change_value!(self.draw_count, diff, i16, u8);
    }
}
