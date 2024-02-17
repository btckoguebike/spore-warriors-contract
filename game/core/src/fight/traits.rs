extern crate alloc;
use alloc::vec::Vec;
use rand::RngCore;

use crate::contexts::{EnemySnapshot, WarriorContext, WarriorSnapshot};
use crate::errors::Error;
use crate::systems::GameSystem;
use crate::wrappings::Enemy;

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

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Copy, PartialEq)]
pub enum IterationOutput {
    Continue,
    GameWin,
    GameLose,
    RequireCardSelect,
    PlayerTurn,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub enum FightLog {
    CharactorSet(WarriorSnapshot, Vec<EnemySnapshot>),
    EnemyTurn(u8),
    PlayerTurn(u8),
    PowerCost(u8),
    SpecialCardUse,
    HandCardUse(usize),
    ItemUse(usize),
    Draw(usize),
    DiscardHandDeck,
    RecoverGraveDeck,
    RecoverPower,
    RecoverHp(u16),
    CallEffectId(u16),
    SystemDamage(usize, u16),
}

pub trait SimplePVE<'a>
where
    Self: Sized,
{
    fn create(player: &'a mut WarriorContext<'a>, enemies: &'a [Enemy]) -> Result<Self, Error>;

    fn start(
        &mut self,
        system: &mut GameSystem<'a, impl RngCore>,
    ) -> Result<(IterationOutput, Vec<FightLog>), Error>;

    fn run(
        &mut self,
        operations: Vec<IterationInput>,
        system: &mut GameSystem<'a, impl RngCore>,
    ) -> Result<(IterationOutput, Vec<FightLog>), Error>;

    fn peak_target(&self, hand_card_selection: Selection) -> Result<bool, Error>;
}
