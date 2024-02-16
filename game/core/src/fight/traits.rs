extern crate alloc;
use alloc::vec::Vec;
use rand::RngCore;

use crate::errors::Error;
use crate::systems::GameSystem;
use crate::wrappings::{Enemy, Potion, Warrior};

#[derive(Clone)]
pub enum Target {
    Player,
    Enemy(usize),
    RandomEnemy,
    AllEnemy,
    AllCharactor,
}

#[derive(Clone)]
pub enum Selection {
    Item(usize),
    SingleCard(usize),
    MultiCards(Vec<usize>),
}

pub enum IterationInput {
    ItemUse(Selection, Option<usize>),
    SpecialCardUse(Option<usize>),
    HandCardUse(Selection, Option<usize>),
    PendingCardSelect(Selection),
    EnemyTurn,
}

#[derive(Clone, Copy, PartialEq)]
pub enum IterationOutput {
    Continue,
    GameWin,
    GameLose,
    RequireCardSelect,
    PlayerTurn,
}

#[derive(Clone, PartialEq)]
pub enum FightLog {
    TurnToEnemy(u8),
    TurnToPlayer(u8),
    PowerCost(u8),
    SpecialCardUse,
    HandCardUse(usize),
    ItemUse(usize),
    Draw(u8),
}

pub trait SimplePVE<'a, T: RngCore>
where
    Self: Sized,
{
    fn create(
        player: &'a Warrior,
        potion: Option<&'a Potion>,
        enemies: &'a [Enemy],
    ) -> Result<Self, Error>;

    fn start(
        &mut self,
        system: &mut GameSystem<'a, T>,
    ) -> Result<(IterationOutput, &Vec<FightLog>), Error>;

    fn run(
        &mut self,
        operations: Vec<IterationInput>,
        system: &mut GameSystem<'a, T>,
    ) -> Result<(IterationOutput, &Vec<FightLog>), Error>;

    fn peak_target(&self, hand_card_selection: Selection) -> Result<bool, Error>;
}
