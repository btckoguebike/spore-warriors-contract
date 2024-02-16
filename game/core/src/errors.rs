#[repr(i8)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum Error {
    ResourceBroken,
    ResourceBrokenScenePool,
    ResourceBrokenCharactorCard,
    ResourceBrokenCardPool,
    ResourceBrokenPartitionRange,
    ResourceBrokenItemClass,
    ResourceBrokenTargetPosition,
    ResourceBrokenEnemyStrategy,

    ResourceEffectMultiTargetInEffectPool,
    ResourceEffectSetupConflict,
    ResourceEffectCardSelectInEnemy,
    ResourceEffectNotNegative,

    ScenePlayerPointBeyondMap,
    ScenePlayerPointInvalid,

    BattleNotStarted,
    BattleRepeatStart,
    BattleOperationInvalid,
    BattleOperationMismatch,
    BattleSelectionError,
    BattleSelectionMismatch,
    BattleUserSelectionMissing,
    BattlePowerInsufficient,
    BattleEnemyNotFound,
    BattleInstructionNotEmpty,
    BattleInstructionEmpty,
    BattleSystemInvalidReturn,

    BattleUnexpectedOutput,
    BattleUnexpectedPosition,
    BattleUnexpectedSystemReturn,

    SystemMissing,
    SystemError,
}
