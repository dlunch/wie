pub type RecordId = u32;

pub trait Database: Send {
    fn add(&mut self, data: &[u8]) -> RecordId;
    fn get(&self, id: RecordId) -> Option<Vec<u8>>;
    fn set(&mut self, id: RecordId, data: &[u8]) -> bool;
    fn delete(&mut self, id: RecordId) -> bool;

    fn get_record_ids(&self) -> Vec<RecordId>;
}

pub trait DatabaseRepository {
    fn open(&self, name: &str, app_id: &str) -> Box<dyn Database>;
}
