/// Unique ID type
pub type ID = (u64, u64);

/// Returns a new unique ID
pub fn new_id() -> ID {
    uuid::Uuid::new_v4().as_u64_pair()
}
