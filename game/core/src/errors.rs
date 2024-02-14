#[repr(i8)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum Error {
    ResourceBroken,
    ResourceBrokenScenePool,
    ResourceBrokenCharactorCard,
    ResourceBrokenCardPool,
    ResourceBrokenPartitionRange,
    ResourceBrokenItemClass,
    ResourceEffectSetupConflict,
    ResourceBrokenTargetPosition,

    ScenePlayerPointBeyondMap,
    ScenePlayerPointInvalid,

    BattleNotStarted,
    BattleRepeatStart,
    BattleOperationInvalid,
    BattleOperationMismatch,
    BattleSelectionError,
    BattleSelectionMismatch,
    BattlePowerInsufficient,
    BattleUnexpectedOutput,
    BattleUnexpectedPosition,
    BattleNoPendingContext,
}
