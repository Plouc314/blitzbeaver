/// Unique ID type
pub type ID = u64;

/// Returns a new unique ID
pub fn new_id() -> ID {
    let (hi, lo) = uuid::Uuid::new_v4().as_u64_pair();
    hi ^ lo
}
