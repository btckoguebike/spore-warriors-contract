extern crate alloc;
use alloc::vec::Vec;

use crate::contexts::{EnemyContext, WarriorContext};
use crate::errors::Error;
use crate::systems::SystemController;
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
    CharactorSet(WarriorContext, Vec<EnemyContext>),
    EnemyTurn(u8),
    PlayerTurn(u8),
    GameOver,

    PowerCost(u8),
    SpecialCardUse,
    HandCardUse(usize),
    ItemUse(usize),
    Draw(usize),
    DiscardHandDeck,
    RecoverGraveDeck,
    RecoverPower,
    RecoverHp(u16),
    CallSystemId(u16),

    SystemDamage(usize, u16),
    SystemRecoverHp(usize, u16),
    SystemAttackPowerUp(usize, u8),
    SystemDefensePowerUp(usize, u8),
}

pub trait SimplePVE<'a>
where
    Self: Sized,
{
    fn create(player: &'a mut WarriorContext, enemies: Vec<Enemy>) -> Result<Self, Error>;

    fn start(
        &mut self,
        controller: &mut SystemController,
    ) -> Result<(IterationOutput, Vec<FightLog>), Error>;

    fn run(
        &mut self,
        operations: Vec<IterationInput>,
        controller: &mut SystemController,
    ) -> Result<(IterationOutput, Vec<FightLog>), Error>;

    fn peak_target(&self, hand_card_selection: Selection) -> Result<bool, Error>;
}
