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
/// KTF's `stream_read` / `stream_write` slots behave like a record-scoped
/// `fread` / `fwrite` pair rather than the standard WIPI record-by-id API
/// — the same record id 1 is walked sequentially with an implicit cursor.
/// The original interface field names (`read_record_single`,
/// `write_record_single`) were a pre-disassembly guess; the impl-side names
/// `stream_read` / `stream_write` reflect the verified semantics.
///
/// Without the cursor the game keeps getting the whole record back,
/// `buf_len < record_len` returns `M_E_SHORTBUF`, the game ignores the
/// negative return code, and downstream code corrupts its own allocator
/// using the leftover stack contents as a record count.
static READ_CURSORS: Mutex<BTreeMap<u32, u32>> = Mutex::new(BTreeMap::new());

/// Per-handle in-memory mirror of record 1, plus a seek cursor.
///
/// `stream_write` writes through this buffer (and to disk, see
/// `stream_write` for why); `close_database` flushes a final time if dirty.
///
/// On `open_database`:
///   - mode 4 (`MC_DB_CREATE`)        — start empty
///   - other modes (1, 2, 8, ...)     — pre-load existing record 1
///
/// `select_record` with a non-zero recid is treated as a seek — KTF apps
/// use slot 4 to position the cursor at known byte offsets within the
/// single backing record, e.g. for multi-slot save files.
struct WriteState {
    buffer: alloc::vec::Vec<u8>,
    cursor: u32,
    dirty: bool,
}

static WRITE_STATES: Mutex<BTreeMap<u32, WriteState>> = Mutex::new(BTreeMap::new());

fn read_cursor(db_id: u32) -> u32 {
    *READ_CURSORS.lock().get(&db_id).unwrap_or(&0)
}

fn set_read_cursor(db_id: u32, value: u32) {
    READ_CURSORS.lock().insert(db_id, value);
}

fn clear_read_cursor(db_id: u32) {
    READ_CURSORS.lock().remove(&db_id);
}

fn init_write_state(db_id: u32, initial: alloc::vec::Vec<u8>) {
    WRITE_STATES.lock().insert(
        db_id,
        WriteState {
            buffer: initial,
            cursor: 0,
            dirty: false,
        },
    );
}

fn write_at_cursor(db_id: u32, data: &[u8]) {
    let mut map = WRITE_STATES.lock();
    let Some(state) = map.get_mut(&db_id) else {
        return;
    };
    let off = state.cursor as usize;
    let end = off + data.len();
    if end > state.buffer.len() {
        state.buffer.resize(end, 0);
    }
    state.buffer[off..end].copy_from_slice(data);
    state.cursor = end as u32;
    state.dirty = true;
}

fn seek_write_cursor(db_id: u32, offset: u32) {
    if let Some(state) = WRITE_STATES.lock().get_mut(&db_id) {
        state.cursor = offset;
    }
}

fn take_write_state(db_id: u32) -> Option<WriteState> {
    WRITE_STATES.lock().remove(&db_id)
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

    // Mode 4 (`MC_DB_CREATE`) wipes any prior contents up front so the new
    // session writes a fresh blob. Other modes seed the per-handle buffer
    // with the existing record so seek+overlay writes preserve unrelated
    // bytes.
    let initial: alloc::vec::Vec<u8> = if mode == 4 {
        let mut db = system.platform().database_repository().open(system, &name, &pid).await;
        db.delete(1).await;
        alloc::vec::Vec::new()
    } else {
        let db = system.platform().database_repository().open(system, &name, &pid).await;
        db.get(1).await.unwrap_or_default()
    };

    let name_bytes = name.as_bytes();
    let mut handle = DatabaseHandle { name: [0; 32] };

    handle.name[..name_bytes.len()].copy_from_slice(name_bytes);

    let ptr_handle = context.alloc_raw(size_of::<DatabaseHandle>() as _)?;
    write_generic(context, ptr_handle, handle)?;

    init_write_state(ptr_handle, initial);

    tracing::debug!("Created database handle {ptr_handle:#x} for {name}");

    Ok(ptr_handle as _)
}

pub async fn close_database(context: &mut dyn WIPICContext, db_id: i32) -> Result<i32> {
    tracing::debug!("MC_dbCloseDataBase({db_id:#x})");

    if db_id < 0x10000 {
        return Ok(-25); // M_E_INVALIDHANDLE
    }

    // Flush per-handle write state if it was modified during the session.
    // Read-only sessions leave dirty=false and skip the write. Note that
    // `stream_write` already does write-through, so this flush is mainly a
    // belt-and-suspenders for cases that bypass it.
    if let Some(state) = take_write_state(db_id as _)
        && state.dirty
    {
        let mut db = get_database_from_db_id(context, db_id).await;
        db.set(1, &state.buffer).await;
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
    if !WRITE_STATES.lock().contains_key(&(db_id as u32)) {
        // Range-valid pointer but no live session — the handle is either
        // stale (already closed) or never came from `open_database`. Don't
        // pretend the bytes landed on disk.
        return Ok(-25); // M_E_INVALIDHANDLE
    }

    let mut buf = vec![0; buf_len as _];
    context.read_bytes(buf_ptr, &mut buf)?;

    // Overlay-write at the per-handle cursor. Default cursor is 0 on open;
    // `select_record(_, recid, ...)` with a non-zero recid moves it before
    // the next write.
    write_at_cursor(db_id as _, &buf);

    // Write-through to disk on every stream_write. Some titles tear down
    // the game without making a final `close_database` call after their
    // save sequence — relying on close as the only flush point loses all
    // the writes that landed since the session opened. Flushing eagerly
    // costs an extra small file write per call but keeps the on-disk state
    // consistent if the process exits or the title forgets to close.
    let snapshot = WRITE_STATES.lock().get(&(db_id as u32)).map(|s| s.buffer.clone());
    if let Some(buffer) = snapshot {
        let mut db = get_database_from_db_id(context, db_id).await;
        db.set(1, &buffer).await;
    }

    Ok(buf_len as _)
}

pub async fn delete_record(context: &mut dyn WIPICContext, a0: i32, a1: i32) -> Result<i32> {
    tracing::debug!("MC_dbDeleteRecord({a0:#x}, {a1})");

    // KTF reuses slot 6 with a name-keyed signature `(name_ptr, type)`,
    // separate from the standard `delete_record(handle, rec_id)`. They are
    // told apart by whether `a0` matches a currently-open handle: only the
    // standard form has a live `WriteState` entry. The KTF form is treated
    // as a no-op so a just-flushed record isn't destroyed by being
    // misinterpreted as a live handle (the bytes of a name string read as
    // a `DatabaseHandle` happen to round-trip through `db.delete(1)`).
    let is_open_handle = WRITE_STATES.lock().contains_key(&(a0 as u32));
    if !is_open_handle {
        tracing::debug!("delete_record: a0={a0:#x} not an open handle (KTF name-keyed call) — no-op");
        return Ok(0);
    }

    let mut db = get_database_from_db_id(context, a0).await;
    let result = db.delete(a1 as _).await;

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

    // Prefer the in-memory write buffer when it has been touched this
    // session. Some titles write a header, rewind via slot 4, then read it
    // back to validate before continuing — going to disk would miss the
    // write between the stream_write call and the next flush.
    let buffer_snapshot = {
        let map = WRITE_STATES.lock();
        map.get(&(db_id as u32)).filter(|s| s.dirty).map(|s| s.buffer.clone())
    };

    let record = if let Some(b) = buffer_snapshot {
        b
    } else {
        let db = get_database_from_db_id(context, db_id).await;
        match db.get(1).await {
            Some(r) => r,
            None => return Ok(-22), // M_E_BADRECID
        }
    };

    let cursor = read_cursor(db_id as _) as usize;
    let total = record.len();
    if cursor >= total {
        // Don't touch buf — caller may have passed a sentinel (NULL) that
        // we shouldn't write to. Some titles do this past EOF.
        return Ok(-23); // M_E_EOF
    }

    let take = core::cmp::min(buf_len as usize, total - cursor);
    if take == 0 {
        return Ok(0);
    }
    context.write_bytes(buf_ptr, &record[cursor..cursor + take])?;
    set_read_cursor(db_id as _, (cursor + take) as _);

    Ok(take as _)
}

pub async fn select_record(_context: &mut dyn WIPICContext, db_id: i32, rec_id: i32, mode: WIPICWord, _buf_len: WIPICWord) -> Result<i32> {
    tracing::debug!("MC_dbSelectRecord({db_id:#x}, {rec_id}, mode={mode:#x}, {_buf_len})");

    if db_id < 0x10000 {
        return Ok(-25); // M_E_INVALIDHANDLE
    }

    // KTF reuses slot 4 as a stream-control op `(handle, offset, mode)`. The
    // shapes observed across games:
    //
    //   - `(handle, slot_offset, 0)` — multi-slot save files store each
    //     slot at a known byte offset within record 1; this seeks both
    //     cursors so the next read/write hits the right slot while
    //     preserving the bytes belonging to the other slots.
    //   - `(handle, 0, 0)` and `(handle, 0, 2)` — rewinds both cursors.
    //     mode=0 vs 2 isn't a length and isn't truncate (truncating on
    //     mode=2 on the read path destroys a prefetched buffer during a
    //     subsequent re-open and wipes the saved record). Both are treated
    //     as plain seek-and-rewind.
    if rec_id >= 0 {
        let offset = rec_id as u32;
        seek_write_cursor(db_id as _, offset);
        set_read_cursor(db_id as _, offset);
        return Ok(0);
    }

    Ok(-22) // M_E_BADRECID
}

/// Slot 5 — KTF custom `db_stat_by_name`. From observed call shape:
///
/// ```text
/// int32 v2[3];
/// ret = slot5(name_ptr, &v2, mode, fn_self_ptr);
/// if (ret == 0 && v2[2] > 0xC7) "valid save";
/// ```
///
/// Takes a name plus a 12-byte (3-int) output struct, and returns 0 when
/// the DB exists with a non-trivial payload. The third int is treated as a
/// size threshold (must exceed 199 bytes). We fill the struct with
/// `{0, 0, record_size}` and return 0 on hit, -22 on miss.
pub async fn stat_by_name(context: &mut dyn WIPICContext, name_ptr: WIPICWord, out_buf: WIPICWord, mode: i32, _arg3: i32) -> Result<i32> {
    let name = match read_null_terminated_string_bytes(context, name_ptr) {
        Ok(bytes) => match String::from_utf8(bytes) {
            Ok(s) => s,
            Err(_) => return Ok(-22),
        },
        Err(_) => return Ok(-22),
    };

    let system = context.system();
    let pid = system.pid().to_owned();
    let exists = system.platform().database_repository().exists(system, &name, &pid).await;
    if !exists {
        tracing::debug!("db.stat_by_name({name:?}, mode={mode}) -> -22 (not found)");
        return Ok(-22);
    }

    // Pull record 1's size as the "valid save" indicator the game checks
    // against 0xC7 in v2[2].
    let db = system.platform().database_repository().open(system, &name, &pid).await;
    let record_size = db.get(1).await.map(|x| x.len() as u32).unwrap_or(0);

    if out_buf != 0 {
        write_generic(context, out_buf, 0u32)?;
        write_generic(context, out_buf + 4, 0u32)?;
        write_generic(context, out_buf + 8, record_size)?;
    }

    tracing::debug!("db.stat_by_name({name:?}, mode={mode}) -> 0 (size={record_size})");
    Ok(0)
}

/// Slot 16 — KTF custom. Observed call shape across multiple titles is
/// `(name_ptr, 1, size_hint_or_zero, callback_garbage)`. Best fit is
/// "does this DB exist?": titles call it before deciding whether to take
/// the load or fresh-init path. Returning 1 unconditionally makes them try
/// to load nonexistent state on first run and trip later, so we read the
/// C string at `a0` and answer based on the real persisted state.
pub async fn unk16(context: &mut dyn WIPICContext, name_ptr: WIPICWord, _arg1: i32, _arg2: i32) -> Result<i32> {
    let name = match read_null_terminated_string_bytes(context, name_ptr) {
        Ok(bytes) => match String::from_utf8(bytes) {
            Ok(s) => s,
            Err(_) => {
                tracing::warn!("MC_dbUnk16 invalid utf8 name @ {name_ptr:#x}, defaulting to 0");
                return Ok(0);
            }
        },
        Err(_) => {
            tracing::warn!("MC_dbUnk16 unreadable name @ {name_ptr:#x}, defaulting to 0");
            return Ok(0);
        }
    };

    let system = context.system();
    let pid = system.pid().to_owned();
    let exists = system.platform().database_repository().exists(system, &name, &pid).await;

    let result = if exists { 1 } else { 0 };
    tracing::debug!("MC_dbUnk16({name:?}) -> {result}");
    Ok(result)
}

async fn get_database_from_db_id(context: &mut dyn WIPICContext, db_id: i32) -> Box<dyn Database> {
    let handle: DatabaseHandle = read_generic(context, db_id as _).unwrap();

    let name_length = handle.name.iter().position(|&c| c == 0).unwrap_or(handle.name.len());
    let db_name = str::from_utf8(&handle.name[..name_length]).unwrap();

    let system = context.system();
    let pid = system.pid().to_owned();

    system.platform().database_repository().open(system, db_name, &pid).await
}
