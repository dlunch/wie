use alloc::{vec, vec::Vec};

use crate::{
    base::{CContext, CMethodBody},
    method::MethodImpl,
};

fn gen_stub(id: u32, name: &'static str) -> CMethodBody {
    let body = move |_: &mut dyn CContext| async move { Err::<(), _>(anyhow::anyhow!("Unimplemented database{}: {}", id, name)) };

    body.into_body()
}

pub fn get_database_method_table() -> Vec<CMethodBody> {
    vec![
        gen_stub(0, "MC_dbOpenDataBase"),
        gen_stub(1, "MC_dbCloseDataBase"),
        gen_stub(2, "MC_dbDeleteDataBase"),
        gen_stub(3, "MC_dbInsertRecord"),
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
