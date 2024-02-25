extern crate alloc;
use alloc::vec;

use crate::contexts::{BytesExtractor, BytesPusher, CardContext, ContextType, CtxAdaptor};
use crate::errors::Error;
use crate::game::ReferContainer;
use crate::wrappings::{Card, Item, ItemClass, Potion, System, Warrior};

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct WarriorSnapshot {
    pub id: u16,
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
    pub special_card: u16,
    pub equipment_list: Vec<u16>,
    pub props_list: Vec<u16>,
    pub hand_deck: Vec<u16>,
    pub deck: Vec<u16>,
    pub grave_deck: Vec<u16>,
    pub selection_deck: Vec<u16>,
    pub mounting_systems: Vec<u16>,
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct WarriorContext<'a> {
    pub warrior: &'a Warrior,
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
    pub special_card: CardContext<'a>,
    pub equipment_list: Vec<&'a Item>,
    pub props_list: Vec<&'a Item>,
    pub hand_deck: Vec<CardContext<'a>>,
    pub deck: Vec<CardContext<'a>>,
    pub grave_deck: Vec<CardContext<'a>>,
    pub selection_deck: Vec<&'a Card>,
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
            .map(|card| CardContext::new(card))
            .collect();
        let mut player = Self {
            warrior,
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
            special_card: CardContext::new(&warrior.charactor_card),
            equipment_list,
            props_list,
            deck,
            hand_deck: vec![],
            grave_deck: vec![],
            selection_deck: vec![],
            mounting_systems: vec![],
        };
        if let Some(potion) = potion {
            let mut package = potion.package_status.iter().map(|v| v).collect();
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
                    .iter()
                    .map(|card| CardContext::new(card))
                    .collect(),
            );
        };
        player
    }

    pub fn reset(&mut self) {
        let origin = self.warrior;
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

    pub fn snapshot(&self) -> WarriorSnapshot {
        WarriorSnapshot {
            id: self.warrior.id,
            offset: self.offset,
            max_hp: self.max_hp,
            hp: self.hp,
            gold: self.gold,
            power: self.power,
            armor: self.armor,
            shield: self.shield,
            attack: self.attack,
            attack_weak: self.attack_weak,
            defense: self.defense,
            defense_weak: self.defense_weak,
            draw_count: self.draw_count,
            special_card: self.special_card.card.id,
            equipment_list: self.equipment_list.iter().map(|v| v.id).collect(),
            props_list: self.props_list.iter().map(|v| v.id).collect(),
            deck: self.deck.iter().map(|v| v.card.id).collect(),
            hand_deck: self.hand_deck.iter().map(|v| v.card.id).collect(),
            grave_deck: self.grave_deck.iter().map(|v| v.card.id).collect(),
            selection_deck: self.selection_deck.iter().map(|v| v.id).collect(),
            mounting_systems: self.mounting_systems.iter().map(|v| v.id.into()).collect(),
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut pusher = BytesPusher::new();
        pusher.push_u16(self.warrior.id);
        pusher.push_usize(self.offset);
        pusher.push_u16(self.max_hp);
        pusher.push_u16(self.hp);
        pusher.push_u16(self.gold);
        pusher.push_u8(self.power);
        pusher.push_u8(self.armor);
        pusher.push_u8(self.shield);
        pusher.push_u8(self.attack);
        pusher.push_u8(self.attack_weak);
        pusher.push_u8(self.defense);
        pusher.push_u8(self.defense_weak);
        pusher.push_u8(self.draw_count);
        pusher.push_bytes(self.special_card.serialize());
        pusher.push_usize(self.equipment_list.len());
        self.equipment_list.iter().for_each(|v| {
            pusher.push_u16(v.unique_id);
        });
        pusher.push_usize(self.props_list.len());
        self.props_list.iter().for_each(|v| {
            pusher.push_u16(v.unique_id);
        });
        pusher.push_usize(self.deck.len());
        self.deck.iter().for_each(|v| {
            pusher.push_bytes(v.serialize());
        });
        pusher.data()
    }

    pub fn deserialize(
        refers: &ReferContainer<'a>,
        warrior: &'a Warrior,
        data: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        let mut extractor = BytesExtractor::new(data);
        Ok(Self {
            warrior,
            offset: extractor.pop_usize()?,
            max_hp: extractor.pop_u16()?,
            hp: extractor.pop_u16()?,
            gold: extractor.pop_u16()?,
            power: extractor.pop_u8()?,
            armor: extractor.pop_u8()?,
            shield: extractor.pop_u8()?,
            attack: extractor.pop_u8()?,
            attack_weak: extractor.pop_u8()?,
            defense: extractor.pop_u8()?,
            defense_weak: extractor.pop_u8()?,
            draw_count: extractor.pop_u8()?,
            special_card: CardContext::deserialize(refers, &mut extractor)?,
            equipment_list: {
                let mut list = Vec::with_capacity(extractor.pop_usize()?);
                for _ in 0..list.len() {
                    let unique_id = extractor.pop_u16()?;
                    list.push(refers.get_item(unique_id)?);
                }
                list
            },
            props_list: {
                let count = extractor.pop_usize()?;
                (0..count)
                    .map(|_| {
                        let unique_id = extractor.pop_u16()?;
                        refers.get_item(unique_id)
                    })
                    .collect::<Result<_, _>>()?
            },
            deck: {
                let count = extractor.pop_usize()?;
                (0..count)
                    .map(|_| CardContext::deserialize(refers, &mut extractor))
                    .collect::<Result<_, _>>()?
            },
            hand_deck: vec![],
            grave_deck: vec![],
            selection_deck: vec![],
            mounting_systems: vec![],
        })
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
