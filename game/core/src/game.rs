use molecule::prelude::Entity;
use rand::rngs::SmallRng;
use rand::{RngCore, SeedableRng};
use spore_warriors_generated as generated;

use crate::contexts::WarriorContext;
use crate::errors::Error;
use crate::map::MapSkeleton;
use crate::systems::GameSystem;
use crate::wrappings::{Point, Potion, Warrior};

pub struct Game {
    resource_pool: generated::ResourcePool,
    rng: SmallRng,
    player: Warrior,
    potion: Option<Potion>,
}

pub struct GameSession<'a, T: RngCore> {
    pub player: WarriorContext<'a>,
    pub system: GameSystem<'a, T>,
    pub map: MapSkeleton,
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
        let mut rng = SmallRng::seed_from_u64(seed);
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
        let mut system = GameSystem::new(&self.resource_pool, &mut self.rng);
        let map = MapSkeleton::randomized(player_point, &mut system)?;
        Ok(GameSession {
            player,
            system,
            map,
        })
    }
}
