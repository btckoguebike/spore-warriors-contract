use core::cmp::{max, min};

use crate::wrappings::{Card, Effect, Enemy, Item, ItemClass, Warrior};

macro_rules! change_value {
    ($this:ident.$field:ident, $diff:ident, $conv:ty, $ori:ty) => {{
        let value = max($this.$field as $conv + $diff as $conv, 0);
        let value = min(value, <$ori>::MAX as $conv);
        $this.$field = value as $ori;
    }};
}

pub trait CtxAdaptor {
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

pub struct EnemyContext<'e> {
    pub enemy: &'e Enemy,
    pub hp: u16,
    pub armor: u8,
    pub shield: u8,
    pub attack: u8,
    pub attack_weak: u8,
    pub defense: u8,
    pub defense_weak: u8,
    pub fight_effects: Vec<&'e Effect>,
}

impl<'e> EnemyContext<'e> {
    pub fn new(enemy: &'e Enemy) -> Self {
        Self {
            enemy,
            hp: enemy.hp,
            armor: enemy.armor,
            shield: enemy.shield,
            attack: enemy.attack,
            attack_weak: enemy.attack_weak,
            defense: enemy.defense,
            defense_weak: enemy.defense_weak,
            fight_effects: vec![],
        }
    }
}

impl<'e> CtxAdaptor for EnemyContext<'e> {
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
pub struct WarriorContext<'w> {
    pub warrior: &'w Warrior,
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
    pub outside_deck: Vec<&'w Card>,
    pub fight_effects: Vec<&'w Effect>,
}

impl<'w> WarriorContext<'w> {
    pub fn new(warrior: &'w Warrior) -> Self {
        Self {
            warrior,
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
            outside_deck: vec![],
            fight_effects: vec![],
        }
    }
}

impl<'p> CtxAdaptor for WarriorContext<'p> {
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
