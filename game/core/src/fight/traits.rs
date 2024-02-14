extern crate alloc;
use alloc::vec::Vec;
use rand::RngCore;

use crate::errors::Error;
use crate::wrappings::{Enemy, Potion, RequireTarget, Warrior};

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
    Deck(Vec<usize>),
}

pub enum IterationInput {
    ItemUse(Selection, Target),
    SpecialCardUse(Target),
    HandCardUse(Selection, Target),
    HandCardSelect(Selection),
    DeckCardSelect(Selection),
    GraveCardSelect(Selection),
    OutsideCardSelect(Selection),
    PlayerRoundEnd,
    EnemyTurn,
}

#[derive(Clone, Copy, PartialEq)]
pub enum IterationOutput {
    GameWin,
    GameLose,
    RequireHandCardSelect,
    RequireDeckCardSelect,
    RequireGraveCardSelect,
    RequireOutsideCardSelect,
    Continue,
    PlayerTurn,
}

pub enum FightLog {}

pub trait SimplePVE<'a, T: RngCore>
where
    Self: Sized,
{
    fn create(
        player: &'a Warrior,
        potion: Option<&'a Potion>,
        enemies: &'a [Enemy],
        rng: &'a mut T,
    ) -> Result<Self, Error>;

    fn start(&mut self) -> Result<IterationOutput, Error>;

    fn run(&mut self, operations: Vec<IterationInput>) -> Result<IterationOutput, Error>;

    fn peak_target(&self, hand_card_selection: Selection) -> Result<Vec<&'a RequireTarget>, Error>;
}
