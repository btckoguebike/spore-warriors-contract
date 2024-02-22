use molecule::prelude::Entity;
use rand::rngs::SmallRng;
use rand::{RngCore, SeedableRng};
use spore_warriors_generated as generated;

use crate::contexts::WarriorContext;
use crate::errors::Error;
use crate::map::MapSkeleton;
use crate::systems::SystemController;
use crate::wrappings::{Point, Potion, Warrior};

pub struct SporeRng {
    rng: SmallRng,
    rotation_count: u16,
}

impl SporeRng {
    pub fn recover(seed: u64, rotation_count: u16) -> Self {
        let mut rng = SmallRng::seed_from_u64(seed);
        (0..rotation_count).for_each(|_| {
            rng.next_u32();
        });
        Self {
            rng,
            rotation_count,
        }
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
    pub resource_pool: generated::ResourcePool,
    pub rng: SporeRng,
    pub player: Warrior,
    pub potion: Option<Potion>,
}

pub struct GameSession<'a, T: RngCore> {
    pub player: WarriorContext<'a>,
    pub system: SystemController<'a, T>,
    pub map: MapSkeleton,
}

impl Game {
    pub fn new(
        raw_resource_pool: &Vec<u8>,
        raw_potion: Option<Vec<u8>>,
        seed: u64,
        rng_rotation_count: u16,
        player_id: u16,
    ) -> Result<Self, Error> {
        let resource_pool = generated::ResourcePool::from_compatible_slice(raw_resource_pool)
            .map_err(|_| Error::ResourceBroken)?;
        let mut rng = SporeRng::recover(seed, rng_rotation_count);
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
    ) -> Result<GameSession<'a, impl RngCore>, Error> {
        let player = WarriorContext::new(&self.player, self.potion.as_ref());
        let map = MapSkeleton::randomized(player_point, &self.resource_pool, &mut self.rng)?;
        let system = SystemController::new(&self.resource_pool, &mut self.rng);
        Ok(GameSession {
            player,
            map,
            system,
        })
    }

    pub fn recover_session<'a>(&'a mut self) {
        unimplemented!()
    }
}
