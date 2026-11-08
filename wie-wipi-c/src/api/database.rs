use alloc::{string::String, vec};
use core::mem::size_of;

use bytemuck::{Pod, Zeroable};

use wie_backend::normalize_guest_path;
use wie_util::{Result, read_generic, read_null_terminated_string_bytes, write_generic};

use crate::WIPICContext;

const M_E_ERROR: i32 = -1;
const M_E_BADFD: i32 = -2;
const M_E_INVALID: i32 = -9;
const M_E_NOENT: i32 = -12;
const M_E_SHORTBUF: i32 = -18;
const M_E_DATABIG: i32 = -21;
const M_E_BADRECID: i32 = -22;

const DATABASE_HANDLE_MAGIC: u32 = 0x44424844;

// Name bytes follow the fixed-size guest handle.
#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct DatabaseHandle {
    magic: u32,
    record_size: u32,
    name_length: u32,
}

pub async fn open(context: &mut dyn WIPICContext, name: u32, record_size: i32, create: i32, mode: i32) -> Result<i32> {
    tracing::debug!("MC_dbOpenDataBase({name:#x}, {record_size}, {create}, {mode})");
    if !(1..=3).contains(&mode) || (create != 0 && record_size <= 0) {
        return Ok(M_E_INVALID);
    }
    let Some(name) = read_name(context, name)? else {
        return Ok(M_E_INVALID);
    };
    let system = context.system().clone();
    let exists = system.platform().database_repository().exists(&name, system.pid()).await;
    if !exists {
        if create == 0 {
            return Ok(M_E_NOENT);
        }
        system.platform().database_repository().create(&name, system.pid()).await;
    }
    let mut database = system.platform().database_repository().open(&name, system.pid()).await;
    if !exists && !database.set(0, &[record_size.to_le_bytes(), mode.to_le_bytes()].concat()).await {
        return Ok(M_E_ERROR);
    }
    let Some(header) = database.get(0).await else {
        return Ok(M_E_ERROR);
    };
    let Ok(header) = <[u8; 8]>::try_from(header.as_slice()) else {
        return Ok(M_E_ERROR);
    };
    let record_size = i32::from_le_bytes(header[..4].try_into().unwrap());
    if record_size <= 0 {
        return Ok(M_E_ERROR);
    }
    let handle = DatabaseHandle {
        magic: DATABASE_HANDLE_MAGIC,
        record_size: record_size as u32,
        name_length: name.len() as u32,
    };
    let address = context.alloc_raw(size_of::<DatabaseHandle>() as u32 + handle.name_length)?;
    write_generic(context, address, handle)?;
    context.write_bytes(address + size_of::<DatabaseHandle>() as u32, name.as_bytes())?;
    Ok(address as i32)
}

pub async fn close(context: &mut dyn WIPICContext, db_id: i32) -> Result<i32> {
    tracing::debug!("MC_dbCloseDataBase({db_id:#x})");
    let Some((handle, _)) = load_handle(context, db_id)? else {
        return Ok(M_E_BADFD);
    };
    write_generic(context, db_id as u32, 0u32)?;
    context.free_raw(db_id as u32, size_of::<DatabaseHandle>() as u32 + handle.name_length)?;
    Ok(0)
}

pub async fn delete(context: &mut dyn WIPICContext, name: u32, mode: i32) -> Result<i32> {
    tracing::debug!("MC_dbDeleteDataBase({name:#x}, {mode})");
    if !(1..=3).contains(&mode) {
        return Ok(M_E_INVALID);
    }
    let Some(name) = read_name(context, name)? else {
        return Ok(M_E_INVALID);
    };
    let system = context.system();
    if !system.platform().database_repository().exists(&name, system.pid()).await {
        return Ok(M_E_NOENT);
    }
    Ok(if system.platform().database_repository().delete(&name, system.pid()).await {
        0
    } else {
        M_E_ERROR
    })
}

pub async fn insert_record(context: &mut dyn WIPICContext, db_id: i32, buffer: u32, length: i32) -> Result<i32> {
    tracing::debug!("MC_dbInsertRecord({db_id:#x}, {buffer:#x}, {length})");
    let Some((handle, name)) = load_handle(context, db_id)? else {
        return Ok(M_E_BADFD);
    };
    if buffer == 0 || length <= 0 {
        return Ok(M_E_INVALID);
    }
    if length as u32 > handle.record_size {
        return Ok(M_E_DATABIG);
    }
    let mut bytes = vec![0; handle.record_size as usize];
    context.read_bytes(buffer, &mut bytes[..length as usize])?;
    let system = context.system();
    let mut database = system.platform().database_repository().open(&name, system.pid()).await;
    let mut ids = database.get_record_ids().await;
    ids.sort_unstable();
    let mut id = 1;
    for existing in ids {
        if existing == id {
            id += 1;
        }
    }
    Ok(if database.set(id, &bytes).await { id as i32 } else { M_E_ERROR })
}

pub async fn select_record(context: &mut dyn WIPICContext, db_id: i32, record_id: i32, buffer: u32, length: i32) -> Result<i32> {
    tracing::debug!("MC_dbSelectRecord({db_id:#x}, {record_id}, {buffer:#x}, {length})");
    let Some((handle, name)) = load_handle(context, db_id)? else {
        return Ok(M_E_BADFD);
    };
    if buffer == 0 || length <= 0 {
        return Ok(M_E_INVALID);
    }
    if (length as u32) < handle.record_size {
        return Ok(M_E_SHORTBUF);
    }
    if record_id <= 0 {
        return Ok(M_E_BADRECID);
    }
    let system = context.system();
    let database = system.platform().database_repository().open(&name, system.pid()).await;
    let Some(bytes) = database.get(record_id as u32).await else {
        return Ok(M_E_BADRECID);
    };
    context.write_bytes(buffer, &bytes)?;
    Ok(0)
}

pub async fn update_record(context: &mut dyn WIPICContext, db_id: i32, record_id: i32, buffer: u32, length: i32) -> Result<i32> {
    tracing::debug!("MC_dbUpdateRecord({db_id:#x}, {record_id}, {buffer:#x}, {length})");
    let Some((handle, name)) = load_handle(context, db_id)? else {
        return Ok(M_E_BADFD);
    };
    if buffer == 0 || length <= 0 {
        return Ok(M_E_INVALID);
    }
    if length as u32 > handle.record_size {
        return Ok(M_E_DATABIG);
    }
    if record_id <= 0 {
        return Ok(M_E_BADRECID);
    }
    let system = context.system().clone();
    let mut database = system.platform().database_repository().open(&name, system.pid()).await;
    let Some(mut bytes) = database.get(record_id as u32).await else {
        return Ok(M_E_BADRECID);
    };
    context.read_bytes(buffer, &mut bytes[..length as usize])?;
    Ok(if database.set(record_id as u32, &bytes).await { 0 } else { M_E_ERROR })
}

pub async fn delete_record(context: &mut dyn WIPICContext, db_id: i32, record_id: i32) -> Result<i32> {
    tracing::debug!("MC_dbDeleteRecord({db_id:#x}, {record_id})");
    let Some((_, name)) = load_handle(context, db_id)? else {
        return Ok(M_E_BADFD);
    };
    if record_id <= 0 {
        return Ok(M_E_BADRECID);
    }
    let system = context.system();
    let mut database = system.platform().database_repository().open(&name, system.pid()).await;
    if database.get(record_id as u32).await.is_none() {
        return Ok(M_E_BADRECID);
    }
    Ok(if database.delete(record_id as u32).await { 0 } else { M_E_ERROR })
}

pub async fn list_records(context: &mut dyn WIPICContext, db_id: i32, buffer: u32, length: i32) -> Result<i32> {
    tracing::debug!("MC_dbListRecords({db_id:#x}, {buffer:#x}, {length})");
    let Some((_, name)) = load_handle(context, db_id)? else {
        return Ok(M_E_BADFD);
    };
    if buffer == 0 || length <= 0 {
        return Ok(M_E_INVALID);
    }
    let system = context.system();
    let database = system.platform().database_repository().open(&name, system.pid()).await;
    let mut ids = database.get_record_ids().await;
    ids.retain(|id| *id != 0);
    if (length as usize) < ids.len() * size_of::<u32>() {
        return Ok(M_E_SHORTBUF);
    }
    for (index, id) in ids.iter().enumerate() {
        write_generic(context, buffer + index as u32 * 4, *id)?;
    }
    Ok(ids.len() as i32)
}

pub async fn get_access_mode(context: &mut dyn WIPICContext, name: u32) -> Result<i32> {
    tracing::debug!("MC_dbGetAccessMode({name:#x})");
    let Some(name) = read_name(context, name)? else {
        return Ok(M_E_INVALID);
    };
    let system = context.system();
    if !system.platform().database_repository().exists(&name, system.pid()).await {
        return Ok(M_E_NOENT);
    }
    let database = system.platform().database_repository().open(&name, system.pid()).await;
    let Some(header) = database.get(0).await else {
        return Ok(M_E_ERROR);
    };
    let Ok(header) = <[u8; 8]>::try_from(header.as_slice()) else {
        return Ok(M_E_ERROR);
    };
    Ok(i32::from_le_bytes(header[4..].try_into().unwrap()))
}

pub async fn get_number_of_records(context: &mut dyn WIPICContext, db_id: i32) -> Result<i32> {
    tracing::debug!("MC_dbGetNumberOfRecords({db_id:#x})");
    let Some((_, name)) = load_handle(context, db_id)? else {
        return Ok(M_E_BADFD);
    };
    let system = context.system();
    let database = system.platform().database_repository().open(&name, system.pid()).await;
    Ok(database.get_record_ids().await.into_iter().filter(|id| *id != 0).count() as i32)
}

pub async fn get_record_size(context: &mut dyn WIPICContext, db_id: i32) -> Result<i32> {
    tracing::debug!("MC_dbGetRecordSize({db_id:#x})");
    Ok(load_handle(context, db_id)?.map_or(M_E_BADFD, |(handle, _)| handle.record_size as i32))
}

fn read_name(context: &dyn WIPICContext, address: u32) -> Result<Option<String>> {
    if address == 0 {
        return Ok(None);
    }
    let Ok(name) = String::from_utf8(read_null_terminated_string_bytes(context, address)?) else {
        return Ok(None);
    };
    Ok(normalize_guest_path(&name))
}

fn load_handle(context: &dyn WIPICContext, db_id: i32) -> Result<Option<(DatabaseHandle, String)>> {
    let Ok(handle) = read_generic::<DatabaseHandle, _>(context, db_id as u32) else {
        return Ok(None);
    };
    if handle.magic != DATABASE_HANDLE_MAGIC {
        return Ok(None);
    }
    let mut bytes = vec![0; handle.name_length as usize];
    context.read_bytes(db_id as u32 + size_of::<DatabaseHandle>() as u32, &mut bytes)?;
    Ok(Some((handle, String::from_utf8_lossy(&bytes).into_owned())))
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use test_utils::TestPlatform;
    use wie_backend::{DefaultTaskRunner, System};
    use wie_util::{ByteRead, ByteWrite, read_generic};

    use crate::context::test::TestContext;

    use super::{
        close, delete, delete_record, get_access_mode, get_number_of_records, get_record_size, insert_record, list_records, open, select_record,
        update_record,
    };

    #[futures_test::test]
    async fn fixed_records_persist_size_and_support_crud() {
        let system = System::new(Box::new(TestPlatform::new()), "test-pid", "same-aid", DefaultTaskRunner);
        let mut context = TestContext::with_system(system.clone());
        context.write_bytes(0x1000, b"/ga/tm\0").unwrap();
        context.write_bytes(0x2000, &[1, 2, 3, 4]).unwrap();
        assert_eq!(open(&mut context, 0x1000, 0, 0, 1).await.unwrap(), -12);
        assert_eq!(open(&mut context, 0x1000, 0, 1, 1).await.unwrap(), -9);
        assert_eq!(open(&mut context, 0x1000, 4, 1, 0).await.unwrap(), -9);
        assert_eq!(open(&mut context, 0, 4, 1, 1).await.unwrap(), -9);
        let first = open(&mut context, 0x1000, 4, 1, 1).await.unwrap();
        assert!(first > 0);
        assert_eq!(get_number_of_records(&mut context, first).await.unwrap(), 0);
        assert_eq!(get_access_mode(&mut context, 0x1000).await.unwrap(), 1);
        assert_eq!(get_record_size(&mut context, first).await.unwrap(), 4);
        assert_eq!(insert_record(&mut context, first, 0x2000, 5).await.unwrap(), -21);
        assert_eq!(insert_record(&mut context, first, 0, 2).await.unwrap(), -9);
        let id = insert_record(&mut context, first, 0x2000, 2).await.unwrap();
        assert!(id > 0);
        assert_eq!(select_record(&mut context, first, id, 0x2100, 3).await.unwrap(), -18);
        assert_eq!(select_record(&mut context, first, 0, 0x2100, 4).await.unwrap(), -22);
        assert_eq!(select_record(&mut context, first, id, 0x2100, 4).await.unwrap(), 0);
        let mut bytes = [0; 4];
        context.read_bytes(0x2100, &mut bytes).unwrap();
        assert_eq!(bytes, [1, 2, 0, 0]);
        context.write_bytes(0x2000, &[9]).unwrap();
        assert_eq!(update_record(&mut context, first, id, 0x2000, 1).await.unwrap(), 0);
        assert_eq!(get_number_of_records(&mut context, first).await.unwrap(), 1);
        assert_eq!(list_records(&mut context, first, 0x2200, 3).await.unwrap(), -18);
        assert_eq!(list_records(&mut context, first, 0x2200, 4).await.unwrap(), 1);
        assert_eq!(read_generic::<u32, _>(&context, 0x2200).unwrap(), id as u32);
        let second = open(&mut context, 0x1000, 99, 1, 1).await.unwrap();
        assert_eq!(get_record_size(&mut context, second).await.unwrap(), 4);
        assert_eq!(close(&mut context, first).await.unwrap(), 0);
        assert_eq!(close(&mut context, first).await.unwrap(), -2);
        assert_eq!(get_record_size(&mut context, first).await.unwrap(), -2);
        assert_eq!(close(&mut context, second).await.unwrap(), 0);

        let repository = system.platform().database_repository();
        assert!(repository.exists("ga/tm", "test-pid").await);
        assert!(!repository.exists("ga/tm", "same-aid").await);
        assert!(!repository.exists("ga/tm", "other-pid").await);
        let mut context = TestContext::with_system(system);
        context.write_bytes(0x1000, b"ga/tm\0").unwrap();
        let reopened = open(&mut context, 0x1000, 0, 0, 1).await.unwrap();
        assert_eq!(get_record_size(&mut context, reopened).await.unwrap(), 4);
        assert_eq!(select_record(&mut context, reopened, id, 0x2100, 4).await.unwrap(), 0);
        context.read_bytes(0x2100, &mut bytes).unwrap();
        assert_eq!(bytes, [9, 2, 0, 0]);
        context.write_bytes(0x2000, &[4, 3, 2, 1]).unwrap();
        let next_id = insert_record(&mut context, reopened, 0x2000, 4).await.unwrap();
        assert_ne!(next_id, id);
        assert_eq!(delete_record(&mut context, reopened, id).await.unwrap(), 0);
        assert_eq!(delete_record(&mut context, reopened, id).await.unwrap(), -22);
        assert_eq!(insert_record(&mut context, reopened, 0x2000, 4).await.unwrap(), id);
        assert_eq!(get_number_of_records(&mut context, reopened).await.unwrap(), 2);
        assert_eq!(select_record(&mut context, reopened, next_id, 0x2100, 4).await.unwrap(), 0);
        context.read_bytes(0x2100, &mut bytes).unwrap();
        assert_eq!(bytes, [4, 3, 2, 1]);
        assert_eq!(delete_record(&mut context, reopened, next_id).await.unwrap(), 0);
        assert_eq!(delete_record(&mut context, reopened, id).await.unwrap(), 0);
        assert_eq!(get_number_of_records(&mut context, reopened).await.unwrap(), 0);
        assert_eq!(close(&mut context, reopened).await.unwrap(), 0);
        assert_eq!(delete(&mut context, 0x1000, 1).await.unwrap(), 0);
        assert_eq!(open(&mut context, 0x1000, 4, 0, 1).await.unwrap(), -12);
        let shared = open(&mut context, 0x1000, 4, 1, 2).await.unwrap();
        assert_eq!(get_access_mode(&mut context, 0x1000).await.unwrap(), 2);
        assert_eq!(delete(&mut context, 0x1000, 2).await.unwrap(), 0);
        assert_eq!(close(&mut context, shared).await.unwrap(), 0);
    }
}
