use alloc::string::String;
use alloc::{vec, vec::Vec};

use crate::{
    base::{WIPICContext, WIPICMethodBody, WIPICWord},
    method::MethodImpl,
    WIPICResult,
};

fn gen_stub(id: WIPICWord, name: &'static str) -> WIPICMethodBody {
    let body = move |_: &mut dyn WIPICContext| async move { Err::<(), _>(anyhow::anyhow!("Unimplemented database{}: {}", id, name)) };

    body.into_body()
}

async fn open_database(_context: &mut dyn WIPICContext, name: String, record_size: i32, create: i32, mode: i32) -> WIPICResult<i32> {
    tracing::warn!("stub MC_dbOpenDataBase({}, {}, {}, {})", name, record_size, create, mode);
    Ok(0)
}

async fn close_database(_context: &mut dyn WIPICContext, db_id: i32) -> WIPICResult<i32> {
    tracing::warn!("stub MC_dbCloseDataBase({})", db_id);
    Ok(0)
}

async fn delete_database(_context: &mut dyn WIPICContext, name: WIPICWord, mode: i32) -> WIPICResult<i32> {
    tracing::warn!("stub MC_dbCloseDataBase({:#x}, {})", name, mode);

    Ok(0)
}

async fn insert_record(_context: &mut dyn WIPICContext, db_id: i32, buf_ptr: WIPICWord, buf_len: WIPICWord) -> WIPICResult<i32> {
    tracing::warn!("stub MC_dbInsertRecord({}, {:#x}, {})", db_id, buf_ptr, buf_len);

    Ok(0)
}

async fn delete_record(_context: &mut dyn WIPICContext, db_id: i32, rec_id: i32) -> WIPICResult<i32> {
    tracing::warn!("stub MC_dbDeleteRecord({}, {})", db_id, rec_id);

    Ok(0)
}

pub fn get_database_method_table() -> Vec<WIPICMethodBody> {
    vec![
        open_database.into_body(),
        close_database.into_body(),
        delete_database.into_body(),
        insert_record.into_body(),
        gen_stub(4, "MC_dbSelectRecord"),
        gen_stub(5, "MC_dbUpdateRecord"),
        delete_record.into_body(),
        gen_stub(7, "MC_dbListRecords"),
        gen_stub(8, "MC_dbSortRecords"),
        gen_stub(9, "MC_dbGetAccessMode"),
        gen_stub(10, "MC_dbGetNumberOfRecords"),
        gen_stub(11, "MC_dbGetRecordSize"),
        gen_stub(12, "MC_dbListDataBases"),
        gen_stub(13, ""),
    ]
}
