extern crate alloc;
use alloc::vec;
use rand::RngCore;

use crate::contexts::{
    add_mounting_system_internal, remove_mounting_system_internal, ContextType, CtxAdaptor,
    SystemContext,
};
use crate::errors::Error;
use crate::wrappings::{Enemy, System};

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct EnemyContext {
    pub enemy: Enemy,
    pub offset: usize,
    pub hp: u16,
    pub armor: u16,
    pub shield: u16,
    pub attack: u8,
    pub attack_weak: u8,
    pub defense: u8,
    pub defense_weak: u8,
    pub strategy: Vec<Vec<System>>,
    pub mounting_systems: Vec<SystemContext>,
}

impl EnemyContext {
    pub fn new(enemy: Enemy, offset: usize) -> Self {
        Self {
            offset,
            hp: enemy.hp,
            armor: enemy.armor as u16,
            shield: enemy.shield as u16,
            attack: enemy.attack,
            attack_weak: enemy.attack_weak,
            defense: enemy.defense,
            defense_weak: enemy.defense_weak,
            strategy: vec![],
            mounting_systems: vec![],
            enemy,
        }
    }

    pub fn pop_action(&mut self, rng: &mut impl RngCore) -> Result<Vec<System>, Error> {
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
            let mut randomized_effects = action.system_pool.clone().into_iter().collect::<Vec<_>>();
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
}

impl CtxAdaptor for EnemyContext {
    fn context_type(&self) -> ContextType {
        ContextType::Enemy
    }

    fn offset(&self) -> usize {
        self.offset
    }

    fn add_mounting_system(&mut self, ctx: &SystemContext) -> bool {
        add_mounting_system_internal(ctx, &mut self.mounting_systems)
    }

    fn remove_mounting_system(&mut self, ctx: &SystemContext) -> bool {
        remove_mounting_system_internal(ctx, &mut self.mounting_systems)
    }

    fn enemy(&mut self) -> Result<&mut EnemyContext, Error> {
        Ok(self)
    }
}
