extern crate alloc;
use alloc::vec::Vec;

use crate::contexts::{EnemyContext, SystemContext, WarriorContext};
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
    GameStart,
    Snapshot(WarriorContext, Vec<EnemyContext>),
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
    CallSystem(usize, SystemContext),
    AddSystem(usize, SystemContext),
    RemoveSystem(usize, SystemContext),

    SystemDamage(usize, u16),
    SystemArmorUp(usize, u16),
    SystemArmorDown(usize, u16),
    SystemShieldUp(usize, u16),
    SystemShieldDown(usize, u16),
    SystemRecoverHp(usize, u16),
    SystemDrawCountUp(u8),
    SystemDrawCountDown(u8),
    SystemAttackPowerUp(usize, u8),
    SystemDefensePowerUp(usize, u8),
    SystemAttackWeak(usize, u8),
    SystemDefenseWeak(usize, u8),
}

impl PartialEq<u16> for FightLog {
    fn eq(&self, other: &u16) -> bool {
        *other
            == match self {
                FightLog::GameStart => 0,
                FightLog::Snapshot(_, _) => 1,
                FightLog::EnemyTurn(_) => 2,
                FightLog::PlayerTurn(_) => 3,
                FightLog::GameOver => 4,
                FightLog::PowerCost(_) => 5,
                FightLog::SpecialCardUse => 6,
                FightLog::HandCardUse(_) => 7,
                FightLog::ItemUse(_) => 8,
                FightLog::Draw(_) => 9,
                FightLog::DiscardHandDeck => 10,
                FightLog::RecoverGraveDeck => 11,
                FightLog::RecoverPower => 11,
                FightLog::RecoverHp(_) => 12,
                FightLog::CallSystem(_, _) => 13,
                FightLog::AddSystem(_, _) => 14,
                FightLog::RemoveSystem(_, _) => 15,
                FightLog::SystemDamage(_, _) => 16,
                FightLog::SystemArmorUp(_, _) => 17,
                FightLog::SystemArmorDown(_, _) => 18,
                FightLog::SystemShieldUp(_, _) => 19,
                FightLog::SystemShieldDown(_, _) => 20,
                FightLog::SystemRecoverHp(_, _) => 21,
                FightLog::SystemDrawCountUp(_) => 22,
                FightLog::SystemDrawCountDown(_) => 23,
                FightLog::SystemAttackPowerUp(_, _) => 24,
                FightLog::SystemDefensePowerUp(_, _) => 25,
                FightLog::SystemAttackWeak(_, _) => 26,
                FightLog::SystemDefenseWeak(_, _) => 27,
            }
    }
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
