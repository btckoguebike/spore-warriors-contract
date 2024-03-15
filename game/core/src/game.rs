extern crate alloc;
use molecule::prelude::Entity;
use rand::rngs::SmallRng;
use rand::{RngCore, SeedableRng};
use spore_warriors_generated as generated;

use crate::contexts::{WarriorContext, WarriorDeckContext};
use crate::errors::Error;
use crate::map::MapSkeleton;
use crate::systems::SystemController;
use crate::wrappings::{Point, Potion, Warrior};

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

pub struct Game {
    pub controller: SystemController,
    pub map: MapSkeleton,
    pub potion: Option<Potion>,
}

impl Game {
    pub fn new(
        raw_resource_pool: &Vec<u8>,
        raw_potion: Option<Vec<u8>>,
        seed: u64,
    ) -> Result<Self, Error> {
        let resource_pool = generated::ResourcePool::from_compatible_slice(raw_resource_pool)
            .map_err(|_| Error::ResourceBroken)?;
        let mut rng = SporeRng::new(seed);
        let potion = {
            if let Some(raw_potion) = raw_potion {
                let potion = generated::Potion::from_compatible_slice(&raw_potion)
                    .map_err(|_| Error::ResourceBroken)?;
                Some(Potion::randomized(&resource_pool, potion, &mut rng)?)
            } else {
                None
            }
        };
        let mut controller = SystemController::new(resource_pool, rng);
        let map = MapSkeleton::randomized(&mut controller)?;
        Ok(Self {
            controller,
            map,
            potion,
        })
    }

    pub fn new_session<'a>(
        &mut self,
        player_id: u16,
        player_point: Point,
    ) -> Result<(WarriorContext, WarriorDeckContext), Error> {
        let resource_pool = &self.controller.resource_pool;
        let rng = &mut self.controller.rng;
        let warrior = {
            let warrior = resource_pool
                .warrior_pool()
                .into_iter()
                .find(|v| u16::from(v.id()) == player_id)
                .ok_or(Error::ResourceBrokenCharactorId)?;
            Warrior::randomized(resource_pool, warrior, rng)?
        };
        self.map.place_player(player_point, true)?;
        Ok(WarriorContext::new(warrior, self.potion.clone()))
    }

    pub fn recover_session<'a>(
        &'a mut self,
        rng_rotation_count: u16,
        player_point: Point,
        raw_context: Vec<u8>,
    ) -> Result<WarriorContext, Error> {
        if !self.controller.rng.rotate_to(rng_rotation_count) {
            return Err(Error::RngRotationError);
        }
        self.map.place_player(player_point, false)?;
        let context = rlp::decode(&raw_context).map_err(|_| Error::DeserializeError)?;
        Ok(context)
    }
}
