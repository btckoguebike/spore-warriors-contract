extern crate alloc;
use alloc::vec;
use rlp::{RlpDecodable, RlpEncodable};

use crate::contexts::{CardContext, ContextType, CtxAdaptor};
use crate::errors::Error;
use crate::wrappings::{Card, Item, ItemClass, Potion, System, Warrior};

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, RlpEncodable, RlpDecodable)]
pub struct WarriorContext {
    pub warrior: Warrior,
    pub offset: usize,
    pub max_hp: u16,
    pub hp: u16,
    pub gold: u16,
    pub power: u8,
    pub armor: u8,
    pub shield: u8,
    pub attack: u8,
    pub attack_weak: u8,
    pub defense: u8,
    pub defense_weak: u8,
    pub draw_count: u8,
    pub special_card: CardContext,
    pub equipment_list: Vec<Item>,
    pub props_list: Vec<Item>,
    pub hand_deck: Vec<CardContext>,
    pub deck: Vec<CardContext>,
    pub grave_deck: Vec<CardContext>,
    pub selection_deck: Vec<Card>,
    pub mounting_systems: Vec<System>,
}

impl WarriorContext {
    pub fn new(warrior: Warrior, potion: Option<Potion>) -> Self {
        let mut equipment_list = vec![];
        let mut props_list = vec![];
        warrior.package_status.iter().for_each(|v| match v.class {
            ItemClass::Equipment => equipment_list.push(v.clone()),
            ItemClass::Props => props_list.push(v.clone()),
        });
        let deck = warrior
            .deck_status
            .iter()
            .map(|card| CardContext::new(card.clone()))
            .collect();
        let mut player = Self {
            offset: 0,
            max_hp: warrior.hp,
            hp: warrior.hp,
            gold: warrior.gold,
            power: warrior.power,
            armor: warrior.armor,
            shield: warrior.shield,
            attack: warrior.attack,
            attack_weak: warrior.attack_weak,
            defense: warrior.defense,
            defense_weak: warrior.defense_weak,
            draw_count: warrior.draw_count,
            special_card: CardContext::new(warrior.charactor_card.clone()),
            equipment_list,
            props_list,
            deck,
            hand_deck: vec![],
            grave_deck: vec![],
            selection_deck: vec![],
            mounting_systems: vec![],
            warrior,
        };
        if let Some(potion) = potion {
            let mut package = potion.package_status;
            player.hp += potion.hp as u16;
            player.power += potion.power;
            player.armor += potion.armor;
            player.shield += potion.shield;
            player.attack += potion.attack;
            player.draw_count += potion.draw_count;
            player.props_list.append(&mut package);
            player.deck.append(
                &mut potion
                    .deck_status
                    .into_iter()
                    .map(|card| CardContext::new(card))
                    .collect(),
            );
        };
        player
    }

    pub fn reset(&mut self) {
        let origin = &self.warrior;
        self.power = origin.power;
        self.armor = origin.armor;
        self.shield = origin.shield;
        self.attack = origin.attack;
        self.attack_weak = origin.attack_weak;
        self.defense = origin.defense;
        self.defense_weak = origin.defense_weak;
        self.draw_count = origin.draw_count;
        self.deck.append(&mut self.hand_deck.drain(..).collect());
        self.deck.append(&mut self.grave_deck.drain(..).collect());
        self.selection_deck.clear();
        self.mounting_systems.clear();
    }
}

impl CtxAdaptor for WarriorContext {
    fn context_type(&self) -> ContextType {
        ContextType::Warrior
    }

    fn offset(&self) -> usize {
        self.offset
    }

    fn warrior(&mut self) -> Result<&mut WarriorContext, Error> {
        Ok(self)
    }
}
