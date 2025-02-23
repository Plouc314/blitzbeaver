/// Unique ID type
pub type ID = uuid::Uuid;

/// Returns a new unique ID
pub fn new_id() -> ID {
    uuid::Uuid::new_v4()
}
