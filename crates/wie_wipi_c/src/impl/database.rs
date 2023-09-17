use alloc::string::String;
use alloc::{vec, vec::Vec};

use crate::{
    base::{CContext, CMethodBody},
    method::MethodImpl,
    CResult,
};

fn gen_stub(id: u32, name: &'static str) -> CMethodBody {
    let body = move |_: &mut dyn CContext| async move { Err::<(), _>(anyhow::anyhow!("Unimplemented database{}: {}", id, name)) };

    body.into_body()
}

async fn open_database(_context: &mut dyn CContext, name: String, record_size: u32, create: u32, mode: u32) -> CResult<u32> {
    tracing::warn!("stub MC_dbOpenDataBase({}, {}, {}, {})", name, record_size, create, mode);
    Ok(0)
}

async fn close_database(_context: &mut dyn CContext, db_id: u32) -> CResult<u32> {
    tracing::warn!("stub MC_dbCloseDataBase({})", db_id);
    Ok(0)
}

async fn insert_record(_context: &mut dyn CContext, db_id: u32, buf_ptr: u32, buf_len: u32) -> CResult<u32> {
    tracing::warn!("stub MC_dbInsertRecord({}, {:#x}, {})", db_id, buf_ptr, buf_len);
    Ok(0)
}

pub fn get_database_method_table() -> Vec<CMethodBody> {
    vec![
        open_database.into_body(),
        close_database.into_body(),
        gen_stub(2, "MC_dbDeleteDataBase"),
        insert_record.into_body(),
        gen_stub(4, "MC_dbSelectRecord"),
        gen_stub(5, "MC_dbUpdateRecord"),
        gen_stub(6, "MC_dbDeleteRecord"),
        gen_stub(7, "MC_dbListRecords"),
        gen_stub(8, "MC_dbSortRecords"),
        gen_stub(9, "MC_dbGetAccessMode"),
        gen_stub(10, "MC_dbGetNumberOfRecords"),
        gen_stub(11, "MC_dbGetRecordSize"),
        gen_stub(12, "MC_dbListDataBases"),
        gen_stub(13, ""),
    ]
}
