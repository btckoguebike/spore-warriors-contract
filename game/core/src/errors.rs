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
    ResourceBrokenSystemId,
    ResourceBrokenPlayerDeck,

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

    BattleUnexpectedDrawCount,
    BattleUnexpectedOutput,
    BattleUnexpectedPosition,
    BattleUnexpectedSystemReturn,
    BattleUnexpectedSystemArgs,

    SystemMissing,
    SystemError,
}
