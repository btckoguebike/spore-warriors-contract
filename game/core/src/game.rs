extern crate alloc;
use alloc::collections::BTreeMap;
use core::cell::RefCell;
use molecule::prelude::Entity;
use rand::rngs::SmallRng;
use rand::{RngCore, SeedableRng};
use spore_warriors_generated as generated;

use crate::contexts::WarriorContext;
use crate::errors::Error;
use crate::map::MapSkeleton;
use crate::systems::SystemController;
use crate::wrappings::{Card, Item, Node, Point, Potion, Warrior};

pub struct SporeRng {
    rng: SmallRng,
    rotation_count: u16,
}

impl SporeRng {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: SmallRng::seed_from_u64(seed),
            rotation_count: 0,
        }
    }

    pub fn rotate_to(&mut self, rotation_count: u16) -> bool {
        if rotation_count > self.rotation_count {
            let rotation_diff = rotation_count - self.rotation_count;
            (0..rotation_diff).for_each(|_| {
                self.next_u32();
            });
            return true;
        }
        false
    }

    pub fn rotation_count(&self) -> u16 {
        self.rotation_count
    }
}

impl RngCore for SporeRng {
    fn next_u32(&mut self) -> u32 {
        self.rng.next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.rotation_count += 1;
        self.rng.next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.rng.fill_bytes(dest)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        self.rng.try_fill_bytes(dest)
    }
}

pub enum Reference<'a> {
    Card(&'a Card),
    Item(&'a Item),
}

#[derive(Default)]
pub struct ReferContainer<'a> {
    pub refers: RefCell<BTreeMap<u16, Reference<'a>>>,
}

impl<'a> ReferContainer<'a> {
    pub fn refer_cards(&self, cards: &'a [Card]) {
        cards.iter().for_each(|card| {
            self.refers
                .borrow_mut()
                .insert(card.unique_id, Reference::Card(card));
        });
    }

    pub fn refer_items(&self, items: &'a [Item]) {
        items.iter().for_each(|item| {
            self.refers
                .borrow_mut()
                .insert(item.unique_id, Reference::Item(item));
        });
    }

    pub fn get_card(&self, unique_id: u16) -> Result<&'a Card, Error> {
        let value = self.refers.borrow();
        let Some(Reference::Card(card)) = value.get(&unique_id) else {
            return Err(Error::ResourceBrokenUniqueId);
        };
        Ok(card)
    }

    pub fn get_item(&self, unique_id: u16) -> Result<&'a Item, Error> {
        let value = self.refers.borrow();
        let Some(Reference::Item(card)) = value.get(&unique_id) else {
            return Err(Error::ResourceBrokenUniqueId);
        };
        Ok(card)
    }
}

pub struct Game {
    pub resource_pool: generated::ResourcePool,
    pub rng: SporeRng,
    pub player: Warrior,
    pub potion: Option<Potion>,
}

impl Game {
    pub fn new(
        raw_resource_pool: &Vec<u8>,
        raw_potion: Option<Vec<u8>>,
        seed: u64,
        player_id: u16,
    ) -> Result<Self, Error> {
        let resource_pool = generated::ResourcePool::from_compatible_slice(raw_resource_pool)
            .map_err(|_| Error::ResourceBroken)?;
        let mut rng = SporeRng::new(seed);
        let warrior = resource_pool
            .warrior_pool()
            .into_iter()
            .find(|v| u16::from(v.id()) == player_id)
            .ok_or(Error::ResourceBrokenCharactorId)?;
        let potion = {
            if let Some(raw_potion) = raw_potion {
                let potion = generated::Potion::from_compatible_slice(&raw_potion)
                    .map_err(|_| Error::ResourceBroken)?;
                Some(Potion::randomized(&resource_pool, potion, &mut rng)?)
            } else {
                None
            }
        };
        Ok(Self {
            player: Warrior::randomized(&resource_pool, warrior, &mut rng)?,
            resource_pool,
            rng,
            potion,
        })
    }

    pub fn new_session<'a>(
        &'a mut self,
        player_point: Point,
    ) -> Result<(MapSkeleton, SystemController<'a>, WarriorContext<'a>), Error> {
        let mut map = MapSkeleton::randomized(&self.resource_pool, &mut self.rng)?;
        map.place_player(player_point, true)?;
        let context = WarriorContext::new(&self.player, self.potion.as_ref());
        let controller = SystemController::new(&self.resource_pool, &mut self.rng);
        Ok((map, controller, context))
    }

    pub fn recover_session<'a>(
        &'a mut self,
        refers: ReferContainer<'a>,
        rng_rotation_count: u16,
        player_point: Point,
        mut raw_context: Vec<u8>,
    ) -> Result<(MapSkeleton, SystemController<'a>, WarriorContext<'a>), Error> {
        let mut map = MapSkeleton::randomized(&self.resource_pool, &mut self.rng)?;
        if !self.rng.rotate_to(rng_rotation_count) {
            return Err(Error::SystemRngRotationError);
        }
        map.place_player(player_point, false)?;
        let context = WarriorContext::deserialize(&refers, &self.player, &mut raw_context)?;
        let controller = SystemController::new(&self.resource_pool, &mut self.rng);
        Ok((map, controller, context))
    }

    pub fn unique_reference<'a>(&'a self, map: &'a MapSkeleton) -> ReferContainer<'a> {
        let refers = ReferContainer::default();
        refers.refer_items(&self.player.package_status);
        refers.refer_cards(&self.player.deck_status);
        map.skeleton.iter().for_each(|level| match &level.node {
            Node::TreasureChest(items, _) | Node::ItemMerchant(items) => refers.refer_items(items),
            Node::CardMerchant(cards) => refers.refer_cards(cards),
            _ => {}
        });
        refers
    }
}
