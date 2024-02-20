extern crate alloc;
use alloc::vec;

use crate::contexts::{CardContext, ContextType, CtxAdaptor};
use crate::errors::Error;
use crate::wrappings::{Item, ItemClass, Potion, System, Warrior};

const SPECIAL_CARD_OFFSET: usize = 10;
const DECK_START_OFFSET: usize = 11;

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
    pub mounting_systems: Vec<u16>,
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct WarriorContext<'a> {
    pub warrior: &'a Warrior,
    pub offset: usize,
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
    pub special_card: CardContext<'a>,
    pub equipment_list: Vec<&'a Item>,
    pub props_list: Vec<&'a Item>,
    pub hand_deck: Vec<CardContext<'a>>,
    pub deck: Vec<CardContext<'a>>,
    pub grave_deck: Vec<CardContext<'a>>,
    pub pending_deck: Vec<CardContext<'a>>,
    pub mounting_systems: Vec<&'a System>,
}

impl<'a> WarriorContext<'a> {
    pub fn new(warrior: &'a Warrior, potion: Option<&'a Potion>) -> Self {
        let mut equipment_list = vec![];
        let mut props_list = vec![];
        warrior.package_status.iter().for_each(|v| match v.class {
            ItemClass::Equipment => equipment_list.push(v),
            ItemClass::Props => props_list.push(v),
        });
        let deck = warrior
            .deck_status
            .iter()
            .enumerate()
            .map(|(offset, card)| CardContext::new(card, DECK_START_OFFSET + offset))
            .collect();
        let mut player = Self {
            warrior,
            offset: 0,
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
            special_card: CardContext::new(&warrior.charactor_card, SPECIAL_CARD_OFFSET),
            equipment_list,
            props_list,
            deck,
            hand_deck: vec![],
            grave_deck: vec![],
            pending_deck: vec![],
            mounting_systems: vec![],
        };
        if let Some(potion) = potion {
            let start_offset = DECK_START_OFFSET + player.deck.len();
            player.hp += potion.hp as u16;
            player.power += potion.power;
            player.armor += potion.armor;
            player.shield += potion.shield;
            player.attack += potion.attack;
            player.draw_count += potion.draw_count;
            player
                .props_list
                .append(&mut potion.package_status.iter().collect());
            player.deck.append(
                &mut potion
                    .deck_status
                    .iter()
                    .enumerate()
                    .map(|(offset, card)| CardContext::new(card, start_offset + offset))
                    .collect(),
            );
        };
        player
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
            special_card: self.special_card.card.id,
            props_list: self.props_list.iter().map(|v| v.id).collect(),
            deck: self.deck.iter().map(|v| v.card.id).collect(),
            hand_deck: self.hand_deck.iter().map(|v| v.card.id).collect(),
            grave_deck: self.grave_deck.iter().map(|v| v.card.id).collect(),
            pending_deck: self.pending_deck.iter().map(|v| v.card.id).collect(),
            mounting_systems: self.mounting_systems.iter().map(|v| v.id.into()).collect(),
        }
    }
}

impl<'a> CtxAdaptor<'a> for WarriorContext<'a> {
    fn context_type(&self) -> ContextType {
        ContextType::Warrior
    }

    fn offset(&self) -> usize {
        self.offset
    }

    fn warrior(&mut self) -> Result<&mut WarriorContext<'a>, Error> {
        Ok(self)
    }
}