#[repr(i8)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum Error {
    ResourceBroken,
    ResourceBrokenScenePool,
    ResourceBrokenCharactorId,
    ResourceBrokenCharactorCard,
    ResourceBrokenCardPool,
    ResourceBrokenCardSelection,
    ResourceBrokenPartitionRange,
    ResourceBrokenItemClass,
    ResourceBrokenTargetPosition,
    ResourceBrokenEnemyStrategy,
    ResourceBrokenSystemPool,
    ResourceBrokenSystemId,
    ResourceBrokenPlayerDeck,
    ResourceBrokenHpPercent,
    ResourceBrokenUniqueId,
    ResourceBrokenDurationCount,

    ResourceEffectMultiTargetInSystemPool,
    ResourceEffectSetupConflict,
    ResourceEffectCardSelectInEnemy,
    ResourceEffectNotNegative,

    ScenePlayerPointBeyondMap,
    ScenePlayerPointInvalid,
    SceneInvalidMove,
    SceneUnexpectedSystemReturn,
    SceneUserImportOutOfIndex,
    SceneMerchantInsufficientGold,
    SceneMerchantInsufficientPhysique,
    SceneTreasureChestOutOfBound,

    BattleNotStarted,
    BattleRepeatStart,
    BattleOperationInvalid,
    BattleOperationMismatch,
    BattleSelectionError,
    BattleSelectionMismatch,
    BattleUserSelectionMissing,
    BattlePowerInsufficient,
    BattleUseCountInsufficient,
    BattleEnemyNotFound,
    BattleInstructionNotEmpty,
    BattleInstructionEmpty,
    BattleSystemInvalidReturn,
    BattleSystemNotMounted,
    BattleInternalError,
    BattleCardOffsetNotFound,
    BattleExceedCardSelection,

    BattleUnexpectedDrawCount,
    BattleUnexpectedDiscardCount,
    BattleUnexpectedOutput,
    BattleUnexpectedPosition,
    BattleUnexpectedSystemReturn,
    BattleUnexpectedSystemArgs,
    BattleUnexpectedSystemContext,
    BattleUnexpectedSystemContextOffset,
    BattleUnexpectedSystemContextDuration,
    BattleUnexpectedSelectionPanel,
    BattleUnexpectedDeckType,
    BattleUnexpectedCardOffset,

    SystemTriggerMissing,
    SystemDeserializeError,
    SystemRngRotationError,
    SystemEmptyReference,
    SystemWrongMixedSystem,
}
