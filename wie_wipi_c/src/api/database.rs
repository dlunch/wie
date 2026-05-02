use alloc::{borrow::ToOwned, boxed::Box, collections::BTreeMap, str, string::String, vec};
use core::mem::size_of;

use bytemuck::{Pod, Zeroable};

use spin::Mutex;

use wipi_types::wipic::WIPICWord;

use wie_backend::Database;
use wie_util::{Result, read_generic, read_null_terminated_string_bytes, write_generic};

use crate::context::WIPICContext;

/// Per-handle byte cursor for `db.stream_read`. Each open starts at zero
/// and the offset advances with every read; the entry is dropped on
/// `db.close`.
///
/// Verified against rhythm.zip's save/load routines (sub_DCE4 / sub_D52C
/// in the ARM client.bin):
///
///   save: open(mode=4) → write(fd, &count, 4) → write(fd, buf, 124*count)
///         → close
///   load: open(mode=1) → read(fd, &count, 4) → loop read(fd, slot, 124)
///         × count → close
///
/// I.e. the same record is appended-to during write and walked with an
/// implicit cursor during read — there is no per-call recid. The KTF SDK
/// header that defines these methods isn't public, so the upstream
/// `WIPICDatabaseInterface` field names (`read_record_single`,
/// `write_record_single`) reflect the maintainer's pre-disasm guess; the
/// behaviour is closer to a record-scoped fread/fwrite pair, which is why
/// the impl-side names are `stream_read` and `stream_write`.
///
/// Without the cursor the game would keep getting the whole record back,
/// `buf_len < record_len` returns `M_E_SHORTBUF`, the game ignores the
/// negative return code, and downstream code corrupts its own allocator
/// using the leftover stack contents as a record count.
static READ_CURSORS: Mutex<BTreeMap<u32, u32>> = Mutex::new(BTreeMap::new());

fn read_cursor(db_id: u32) -> u32 {
    *READ_CURSORS.lock().get(&db_id).unwrap_or(&0)
}

fn set_read_cursor(db_id: u32, value: u32) {
    READ_CURSORS.lock().insert(db_id, value);
}

fn clear_read_cursor(db_id: u32) {
    READ_CURSORS.lock().remove(&db_id);
}

#[derive(Pod, Zeroable, Copy, Clone)]
#[repr(C)]
struct DatabaseHandle {
    name: [u8; 32], // TODO hardcoded max size
}

pub async fn open_database(context: &mut dyn WIPICContext, ptr_name: WIPICWord, mode: i32, r#type: i32) -> Result<i32> {
    tracing::debug!("MC_dbOpenDataBase({ptr_name:#x}, {mode}, {type})");

    let name = String::from_utf8(read_null_terminated_string_bytes(context, ptr_name)?).unwrap();

    let system = context.system();
    let pid = system.pid().to_owned();

    let exists = system.platform().database_repository().exists(system, &name, &pid).await;

    if !exists && mode == 1 {
        return Ok(-12); // M_E_NOENT
    }

    // Mode 4 (`MC_DB_CREATE`) wipes any prior contents so the next
    // `stream_write` starts the stream from offset 0.
    if mode == 4 {
        let mut db = system.platform().database_repository().open(system, &name, &pid).await;
        db.delete(1).await;
    }

    let name_bytes = name.as_bytes();
    let mut handle = DatabaseHandle { name: [0; 32] };

    handle.name[..name_bytes.len()].copy_from_slice(name_bytes);

    let ptr_handle = context.alloc_raw(size_of::<DatabaseHandle>() as _)?;
    write_generic(context, ptr_handle, handle)?;

    tracing::debug!("Created database handle {ptr_handle:#x} for {name}");

    Ok(ptr_handle as _)
}

pub async fn close_database(context: &mut dyn WIPICContext, db_id: i32) -> Result<i32> {
    tracing::debug!("MC_dbCloseDataBase({db_id:#x})");

    if db_id < 0x10000 {
        return Ok(-25); // M_E_INVALIDHANDLE
    }

    clear_read_cursor(db_id as _);
    context.free_raw(db_id as _, size_of::<DatabaseHandle>() as _)?;

    Ok(0) // success
}

pub async fn list_record(context: &mut dyn WIPICContext, db_id: i32, buf_ptr: WIPICWord, buf_len: WIPICWord) -> Result<i32> {
    tracing::debug!("MC_dbListRecords({db_id:#x}, {buf_ptr:#x}, {buf_len})");

    let db = get_database_from_db_id(context, db_id).await;
    let ids = db.get_record_ids().await;

    let mut cursor = 0;
    for &id in &ids {
        write_generic(context, buf_ptr + cursor, id)?;
        cursor += size_of::<WIPICWord>() as u32;
    }

    Ok(ids.len() as _)
}

pub async fn stream_write(context: &mut dyn WIPICContext, db_id: i32, buf_ptr: WIPICWord, buf_len: WIPICWord) -> Result<i32> {
    tracing::debug!("db.stream_write({db_id:#x}, {buf_ptr:#x}, {buf_len})");

    if db_id < 0x10000 {
        return Ok(-25); // M_E_INVALIDHANDLE
    }

    let mut buf = vec![0; buf_len as _];
    context.read_bytes(buf_ptr, &mut buf)?;
    let mut db = get_database_from_db_id(context, db_id).await;

    // Append to record 1: KTF apps stream a single-record DB across multiple
    // calls (header byte count, then payload). Overwriting on each call would
    // lose the header.
    let existing = db.get(1).await.unwrap_or_default();
    let mut combined = existing;
    combined.extend_from_slice(&buf);
    db.set(1, &combined).await;

    Ok(buf_len as _)
}

pub async fn delete_record(context: &mut dyn WIPICContext, db_id: i32, rec_id: i32) -> Result<i32> {
    tracing::debug!("MC_dbDeleteRecord({db_id:#x}, {rec_id})");

    let mut db = get_database_from_db_id(context, db_id).await;

    let result = db.delete(rec_id as _).await;

    if result {
        Ok(0) // success
    } else {
        Ok(-22) // M_E_BADRECID
    }
}

pub async fn stream_read(context: &mut dyn WIPICContext, db_id: i32, buf_ptr: WIPICWord, buf_len: WIPICWord) -> Result<i32> {
    tracing::debug!("db.stream_read({db_id:#x}, {buf_ptr:#x}, {buf_len})");

    if db_id < 0x10000 {
        return Ok(-25); // M_E_INVALIDHANDLE
    }

    if buf_len > 0 {
        // Some KTF apps ignore the negative return code and trust the buffer
        // contents anyway. Any leftover stack bytes look like a record count
        // to downstream code and derail its iteration loop.
        let zero = alloc::vec![0u8; buf_len as usize];
        context.write_bytes(buf_ptr, &zero)?;
    }

    let db = get_database_from_db_id(context, db_id).await;

    let Some(record) = db.get(1).await else {
        return Ok(-22); // M_E_BADRECID
    };

    let cursor = read_cursor(db_id as _) as usize;
    let total = record.len();
    if cursor >= total {
        return Ok(-23); // M_E_EOF
    }
    let take = core::cmp::min(buf_len as usize, total - cursor);
    context.write_bytes(buf_ptr, &record[cursor..cursor + take])?;
    set_read_cursor(db_id as _, (cursor + take) as _);

    Ok(take as _)
}

pub async fn select_record(context: &mut dyn WIPICContext, db_id: i32, rec_id: i32, buf_ptr: WIPICWord, buf_len: WIPICWord) -> Result<i32> {
    tracing::debug!("MC_dbSelectRecord({db_id:#x}, {rec_id}, {buf_ptr:#x}, {buf_len})");

    if db_id < 0x10000 {
        return Ok(-25); // M_E_INVALIDHANDLE
    }

    let db = get_database_from_db_id(context, db_id).await;

    if let Some(x) = db.get(rec_id as _).await {
        if buf_len < x.len() as _ {
            return Ok(-18); // M_E_SHORTBUF
        }
        context.write_bytes(buf_ptr, &x)?;

        Ok(0)
    } else {
        Ok(-22) // M_E_BADRECID
    }
}

pub async fn unk16(_context: &mut dyn WIPICContext) -> Result<i32> {
    tracing::warn!("stub MC_dbUnk16()");

    Ok(1)
}

async fn get_database_from_db_id(context: &mut dyn WIPICContext, db_id: i32) -> Box<dyn Database> {
    let handle: DatabaseHandle = read_generic(context, db_id as _).unwrap();

    let name_length = handle.name.iter().position(|&c| c == 0).unwrap_or(handle.name.len());
    let db_name = str::from_utf8(&handle.name[..name_length]).unwrap();

    let system = context.system();
    let pid = system.pid().to_owned();

    system.platform().database_repository().open(system, db_name, &pid).await
}
