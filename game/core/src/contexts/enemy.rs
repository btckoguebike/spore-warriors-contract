extern crate alloc;
use alloc::vec;
use rand::RngCore;

use crate::contexts::{ContextType, CtxAdaptor};
use crate::errors::Error;
use crate::wrappings::{Enemy, System};

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

pub struct EnemyContext<'a> {
    pub enemy: &'a Enemy,
    pub offset: usize,
    pub hp: u16,
    pub armor: u8,
    pub shield: u8,
    pub attack: u8,
    pub attack_weak: u8,
    pub defense: u8,
    pub defense_weak: u8,
    pub strategy: Vec<Vec<&'a System>>,
    pub mounting_systems: Vec<&'a System>,
}

impl<'a> EnemyContext<'a> {
    pub fn new(enemy: &'a Enemy, offset: usize) -> Self {
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
            mounting_systems: vec![],
        }
    }

    pub fn pop_action(&mut self, rng: &mut impl RngCore) -> Result<Vec<&'a System>, Error> {
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
            let mut randomized_effects = action.system_pool.iter().collect::<Vec<_>>();
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
            pending_effects: self.mounting_systems.iter().map(|v| v.id.into()).collect(),
        }
    }
}

impl<'a> CtxAdaptor<'a> for EnemyContext<'a> {
    fn context_type(&self) -> ContextType {
        ContextType::Enemy
    }

    fn offset(&self) -> usize {
        self.offset
    }

    fn enemy(&mut self) -> Result<&mut EnemyContext<'a>, Error> {
        Ok(self)
    }
}
