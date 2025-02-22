pub type ID = uuid::Uuid;

pub fn new_id() -> ID {
    uuid::Uuid::new_v4()
}
