extern crate alloc;
use alloc::vec::Vec;

use crate::contexts::{EnemyContext, SystemContext, WarriorContext, WarriorDeckContext};
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
    RequireCardSelect(u8, bool),
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
    DiscardAllHandDeck,
    DiscardHandDeck(usize),
    RecoverGraveDeck,
    RecoverPower,
    RecoverCardCost,
    RecoverHp(u16),
    CallSystem(usize, SystemContext),
    AddSystem(usize, SystemContext),
    UpdateSystem(usize, SystemContext),
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
    SystemAttackPowerDown(usize, u8),
    SystemDefensePowerUp(usize, u8),
    SystemDefensePowerDown(usize, u8),
    SystemAttackWeakUp(usize, u8),
    SystemAttackWeakDown(usize, u8),
    SystemDefenseWeakUp(usize, u8),
    SystemDefenseWeakDown(usize, u8),
    SystemMaxHpUp(u16),
    SystemMaxHpDown(u16),
    SystemPowerCostChange(usize, u8),
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
                FightLog::DiscardAllHandDeck => 10,
                FightLog::DiscardHandDeck(_) => 11,
                FightLog::RecoverGraveDeck => 12,
                FightLog::RecoverPower => 13,
                FightLog::RecoverCardCost => 14,
                FightLog::RecoverHp(_) => 15,
                FightLog::CallSystem(_, _) => 16,
                FightLog::AddSystem(_, _) => 17,
                FightLog::UpdateSystem(_, _) => 18,
                FightLog::RemoveSystem(_, _) => 19,
                FightLog::SystemDamage(_, _) => 20,
                FightLog::SystemArmorUp(_, _) => 21,
                FightLog::SystemArmorDown(_, _) => 22,
                FightLog::SystemShieldUp(_, _) => 23,
                FightLog::SystemShieldDown(_, _) => 24,
                FightLog::SystemRecoverHp(_, _) => 25,
                FightLog::SystemDrawCountUp(_) => 26,
                FightLog::SystemDrawCountDown(_) => 27,
                FightLog::SystemAttackPowerUp(_, _) => 28,
                FightLog::SystemAttackPowerDown(_, _) => 29,
                FightLog::SystemDefensePowerUp(_, _) => 30,
                FightLog::SystemDefensePowerDown(_, _) => 31,
                FightLog::SystemAttackWeakUp(_, _) => 32,
                FightLog::SystemAttackWeakDown(_, _) => 33,
                FightLog::SystemDefenseWeakUp(_, _) => 34,
                FightLog::SystemDefenseWeakDown(_, _) => 35,
                FightLog::SystemMaxHpUp(_) => 36,
                FightLog::SystemMaxHpDown(_) => 37,
                FightLog::SystemPowerCostChange(_, _) => 38,
            }
    }
}

pub trait SimplePVE<'a>
where
    Self: Sized,
{
    fn create(
        player: &'a mut WarriorContext,
        player_deck: &'a mut WarriorDeckContext,
        enemies: Vec<Enemy>,
    ) -> Result<Self, Error>;

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
