#[cfg(feature = "debug")]
use thiserror::Error;

#[repr(i8)]
#[cfg_attr(feature = "debug", derive(Debug, Error))]
pub enum Error {
    #[cfg_attr(feature = "debug", error("resource binary was broken"))]
    ResourceBroken,
    #[cfg_attr(feature = "debug", error("scene pool was broken"))]
    ResourceBrokenScenePool,
    #[cfg_attr(feature = "debug", error("charactor id was broken"))]
    ResourceBrokenCharactorId,
    #[cfg_attr(feature = "debug", error("charactor card was broken"))]
    ResourceBrokenCharactorCard,
    #[cfg_attr(feature = "debug", error("card pool was broken"))]
    ResourceBrokenCardPool,
    #[cfg_attr(feature = "debug", error("map partition range was broken"))]
    ResourceBrokenPartitionRange,
    #[cfg_attr(feature = "debug", error("item class was broken"))]
    ResourceBrokenItemClass,
    #[cfg_attr(feature = "debug", error("target position was broken"))]
    ResourceBrokenTargetPosition,
    #[cfg_attr(feature = "debug", error("enemy strategy was broken"))]
    ResourceBrokenEnemyStrategy,
    #[cfg_attr(feature = "debug", error("system pool was broken"))]
    ResourceBrokenSystemPool,
    #[cfg_attr(feature = "debug", error("system id was broken"))]
    ResourceBrokenSystemId,
    #[cfg_attr(feature = "debug", error("player deck was broken"))]
    ResourceBrokenPlayerDeck,
    #[cfg_attr(feature = "debug", error("player hp recover percent was broken"))]
    ResourceBrokenHpPercent,
    #[cfg_attr(feature = "debug", error("system duration count was broken"))]
    ResourceBrokenDurationCount,
    #[cfg_attr(feature = "debug", error("deck type was broken"))]
    ResourceBrokenDeckType,
    #[cfg_attr(feature = "debug", error("duplicated system target"))]
    ResourceSystemTargetInSystemPoolDuplicated,
    #[cfg_attr(feature = "debug", error("invalid card selection in enemy"))]
    ResourceSystemCardSelectionInEnemy,
    #[cfg_attr(feature = "debug", error("player point exceeded map's boundary"))]
    ScenePlayerPointBeyondMap,
    #[cfg_attr(feature = "debug", error("invalid player point in map"))]
    ScenePlayerPointInvalid,
    #[cfg_attr(feature = "debug", error("invalid player movement"))]
    SceneInvalidMove,
    #[cfg_attr(feature = "debug", error("unexpected system result in map"))]
    SceneUnexpectedSystemReturn,
    #[cfg_attr(feature = "debug", error("purchase or item offset out of index"))]
    SceneUserImportOutOfIndex,
    #[cfg_attr(feature = "debug", error("insufficient gold in purchase"))]
    SceneMerchantInsufficientGold,
    #[cfg_attr(feature = "debug", error("insufficient physique in purchase"))]
    SceneMerchantInsufficientPhysique,
    #[cfg_attr(feature = "debug", error("overwheelmed treasure chest picking"))]
    SceneTreasureChestOutOfBound,
    #[cfg_attr(feature = "debug", error("battle not start"))]
    BattleNotStarted,
    #[cfg_attr(feature = "debug", error("battle repeat start"))]
    BattleRepeatStart,
    #[cfg_attr(feature = "debug", error("invalid operation in battle"))]
    BattleOperationInvalid,
    #[cfg_attr(feature = "debug", error("invalid peak operation in battle"))]
    BattleInvalidPeakOperation,
    #[cfg_attr(feature = "debug", error("invalid target offset"))]
    BattleTargetOffsetError,
    #[cfg_attr(feature = "debug", error("invalid card or item selection"))]
    BattleSelectionError,
    #[cfg_attr(feature = "debug", error("invalid operation in battle iteration"))]
    BattleInvalidIterationOperation,
    #[cfg_attr(feature = "debug", error("insufficient power"))]
    BattlePowerInsufficient,
    #[cfg_attr(feature = "debug", error("insufficient special card use count"))]
    BattleUseCountInsufficient,
    #[cfg_attr(feature = "debug", error("enemy not found in target offset"))]
    BattleEnemyNotFound,
    #[cfg_attr(feature = "debug", error("non-empty battle instructions"))]
    BattleInstructionNotEmpty,
    #[cfg_attr(feature = "debug", error("empty battle instructions"))]
    BattleInstructionEmpty,
    #[cfg_attr(feature = "debug", error("cannot refer card in target offset"))]
    BattleInvalidCardOffsetToRefer,
    #[cfg_attr(feature = "debug", error("player deck in battle was broken"))]
    BattlePlayerDeckBroken,
    #[cfg_attr(feature = "debug", error("card offset not found in draw step"))]
    BattleDrawCardOffsetNotFound,
    #[cfg_attr(feature = "debug", error("card selection count exceeeded"))]
    BattleExceedCardSelection,
    #[cfg_attr(feature = "debug", error("battle unexpected draw count"))]
    BattleUnexpectedDrawCount,
    #[cfg_attr(feature = "debug", error("battle unexpected discard count"))]
    BattleUnexpectedDiscardCount,
    #[cfg_attr(feature = "debug", error("battle unexpected last output"))]
    BattleUnexpectedLastOutput,
    #[cfg_attr(feature = "debug", error("battle unexpected system args"))]
    BattleUnexpectedSystemArgs,
    #[cfg_attr(feature = "debug", error("battle unexpected system context"))]
    BattleUnexpectedSystemContext,
    #[cfg_attr(feature = "debug", error("battle unexpected system duration"))]
    BattleUnexpectedSystemDuration,
    #[cfg_attr(feature = "debug", error("battle unexpected system duration"))]
    BattleUnexpectedCardOffset,
    #[cfg_attr(feature = "debug", error("missing system trigger type"))]
    SystemTriggerMissing,
    #[cfg_attr(feature = "debug", error("deserialization error"))]
    DeserializeError,
    #[cfg_attr(feature = "debug", error("RNG rotation error"))]
    RngRotationError,
}
