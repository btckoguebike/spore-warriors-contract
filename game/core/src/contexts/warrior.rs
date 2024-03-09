extern crate alloc;
use alloc::{vec, vec::Vec};
use rlp::{RlpDecodable, RlpEncodable};

use crate::contexts::system::SystemContext;
use crate::contexts::{
    add_mounting_system_internal, remove_mounting_system_internal, update_mounting_system_internal,
    CardContext, ContextType, CtxAdaptor,
};
use crate::errors::Error;
use crate::wrappings::{Item, ItemClass, Potion, System, Warrior};

#[cfg_attr(feature = "debug", derive(Debug, PartialEq))]
#[derive(Default, Clone, RlpEncodable, RlpDecodable)]
pub struct CardSelection {
    pub selection_pool: Vec<usize>,
    pub unbelonging_deck: Vec<CardContext>,
}

#[cfg_attr(feature = "debug", derive(Debug, PartialEq))]
#[derive(Clone, RlpEncodable, RlpDecodable)]
pub struct WarriorContext {
    pub warrior: Warrior,
    pub offset: usize,
    pub max_hp: u16,
    pub hp: u16,
    pub gold: u16,
    pub power: u8,
    pub armor: u16,
    pub shield: u16,
    pub attack: u8,
    pub attack_weak: u8,
    pub defense: u8,
    pub defense_weak: u8,
    pub draw_count: u8,
    pub physique: u8,
    pub special_card: CardContext,
    pub equipment_list: Vec<Item>,
    pub props_list: Vec<Item>,
    pub hand_deck: Vec<CardContext>,
    pub deck: Vec<CardContext>,
    pub grave_deck: Vec<CardContext>,
    pub unavaliable_deck: Vec<CardContext>,
    pub mounting_systems: Vec<SystemContext>,
    pub card_selection: CardSelection,
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
            armor: warrior.armor as u16,
            shield: warrior.shield as u16,
            attack: warrior.attack,
            attack_weak: warrior.attack_weak,
            defense: warrior.defense,
            defense_weak: warrior.defense_weak,
            draw_count: warrior.draw_count,
            physique: warrior.physique,
            special_card: CardContext::new(warrior.charactor_card.clone()),
            equipment_list,
            props_list,
            deck,
            hand_deck: vec![],
            grave_deck: vec![],
            unavaliable_deck: vec![],
            mounting_systems: vec![],
            card_selection: Default::default(),
            warrior,
        };
        if let Some(potion) = potion {
            let mut package = potion.package_status;
            player.hp += potion.hp as u16;
            player.power += potion.power;
            player.armor += potion.armor as u16;
            player.shield += potion.shield as u16;
            player.attack += potion.attack;
            player.draw_count += potion.draw_count;
            player.physique += potion.physique;
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

    pub fn refer_card(&mut self, offset: usize) -> Option<&mut CardContext> {
        if self.special_card.offset() == offset {
            return Some(&mut self.special_card);
        }
        if let Some(card) = self.hand_deck.iter_mut().find(|v| v.offset() == offset) {
            return Some(card);
        }
        if let Some(card) = self.deck.iter_mut().find(|v| v.offset() == offset) {
            return Some(card);
        }
        if let Some(card) = self.grave_deck.iter_mut().find(|v| v.offset() == offset) {
            return Some(card);
        }
        None
    }

    pub fn collect_card_mountings(&self) -> Vec<(usize, Vec<SystemContext>)> {
        let mut collection = vec![];
        collection.push((
            self.special_card.offset(),
            self.special_card.mounting_systems.clone(),
        ));
        self.deck
            .iter()
            .for_each(|v| collection.push((v.offset(), v.mounting_systems.clone())));
        self.hand_deck
            .iter()
            .for_each(|v| collection.push((v.offset(), v.mounting_systems.clone())));
        self.grave_deck
            .iter()
            .for_each(|v| collection.push((v.offset(), v.mounting_systems.clone())));
        collection
    }

    pub fn collect_card_systems(&self) -> Vec<(usize, Vec<System>)> {
        let mut collection = vec![];
        collection.push((
            self.special_card.offset(),
            self.special_card.card.system_pool.clone(),
        ));
        self.deck
            .iter()
            .for_each(|v| collection.push((v.offset(), v.card.system_pool.clone())));
        self.hand_deck
            .iter()
            .for_each(|v| collection.push((v.offset(), v.card.system_pool.clone())));
        self.grave_deck
            .iter()
            .for_each(|v| collection.push((v.offset(), v.card.system_pool.clone())));
        collection
    }

    pub fn reset(&mut self) {
        let origin = &self.warrior;
        self.power = origin.power;
        self.armor = origin.armor as u16;
        self.shield = origin.shield as u16;
        self.attack = origin.attack;
        self.attack_weak = origin.attack_weak;
        self.defense = origin.defense;
        self.defense_weak = origin.defense_weak;
        self.draw_count = origin.draw_count;
        self.deck.append(&mut self.hand_deck.drain(..).collect());
        self.deck.append(&mut self.grave_deck.drain(..).collect());
        self.deck.iter_mut().for_each(|card| card.reset());
        self.card_selection.selection_pool.clear();
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

    fn add_mounting_system(&mut self, ctx: &SystemContext) -> bool {
        add_mounting_system_internal(ctx, &mut self.mounting_systems)
    }

    fn update_mounting_system(&mut self, ctx: &SystemContext) -> bool {
        update_mounting_system_internal(ctx, &mut self.mounting_systems)
    }

    fn remove_mounting_system(&mut self, ctx: &SystemContext) -> bool {
        remove_mounting_system_internal(ctx, &mut self.mounting_systems)
    }

    fn warrior(&mut self) -> Result<&mut WarriorContext, Error> {
        Ok(self)
    }
}
