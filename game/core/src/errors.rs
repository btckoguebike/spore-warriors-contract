#[repr(i8)]
pub enum Error {
    ResourceBroken,
    ResourceBrokenScenePool,
    ResourceBrokenCharactorCard,
    ResourceBrokenCardPool,
    ResourceBrokenPartitionRange,

    ScenePlayerPointBeyondMap,
    ScenePlayerPointInvalid,
}
