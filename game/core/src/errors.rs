#[repr(i8)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum Error {
    ResourceBroken,
    ResourceBrokenScenePool,
    ResourceBrokenCharactorId,
    ResourceBrokenCharactorCard,
    ResourceBrokenCardPool,
    ResourceBrokenPartitionRange,
    ResourceBrokenItemClass,
    ResourceBrokenTargetPosition,
    ResourceBrokenEnemyStrategy,
    ResourceBrokenSystemPool,
    ResourceBrokenSystemId,
    ResourceBrokenPlayerDeck,
    ResourceBrokenHpPercent,
    ResourceBrokenUniqueId,

    ResourceEffectMultiTargetInEffectPool,
    ResourceEffectSetupConflict,
    ResourceEffectCardSelectInEnemy,
    ResourceEffectNotNegative,

    ScenePlayerPointBeyondMap,
    ScenePlayerPointInvalid,
    SceneInvalidMove,
    SceneUnexpectedSystemReturn,
    SceneUserImportOutOfIndex,
    SceneMerchantInsufficientGold,
    SceneTreasureChestOutOfBound,

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
    BattleUnexpectedSystemContext,

    SystemMissing,
    SystemError,
    SystemDeserializeError,
}
