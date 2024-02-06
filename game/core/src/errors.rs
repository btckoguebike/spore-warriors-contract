#[repr(i8)]
pub enum Error {
    ResourceBroken,
    ResourceBrokenScenePool,
    ResourceBrokenCardPool,
    ResourceBrokenPartitionRange,

    ScenePlayerPointBeyondMap,
    ScenePlayerPointInvalid,
}
