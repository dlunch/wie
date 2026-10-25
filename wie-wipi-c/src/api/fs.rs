use alloc::{borrow::ToOwned, string::String, vec};
use core::{mem::size_of, str};

use bytemuck::{Pod, Zeroable};

use wie_util::{Result, read_generic, read_null_terminated_string_bytes, write_generic};

use crate::context::WIPICContext;

#[derive(Pod, Zeroable, Copy, Clone)]
#[repr(C)]
struct FileHandle {
    magic: u32,
    position: i32,
    mode: i32,
    name: [u8; 128],
}

const FILE_HANDLE_MAGIC: u32 = 0x46534648;
const STORAGE_LIMIT: u64 = 1024 * 1024;
const OPEN_READ_ONLY: i32 = 1;
const OPEN_WRITE_ONLY: i32 = 2;
const OPEN_WRITE_TRUNCATE: i32 = 4;

pub async fn open(context: &mut dyn WIPICContext, name: u32, mode: i32, access_mode: i32) -> Result<i32> {
    tracing::debug!("MC_fsOpen({name:#x}, {mode}, {access_mode})");

    let Ok(name) = String::from_utf8(read_null_terminated_string_bytes(context, name)?) else {
        return Ok(-3); // M_E_BADFILENAME
    };
    let filesystem = context.system().filesystem().clone();
    if name.len() >= 128 || !filesystem.is_valid_path(&name) {
        return Ok(-3);
    }

    let exists = filesystem.exists(&name).await;
    if !exists && mode == OPEN_READ_ONLY {
        return Ok(-12); // M_E_NOENT
    }
    if (mode == OPEN_WRITE_TRUNCATE || !exists) && !filesystem.truncate(&name, 0).await {
        return Ok(-1); // M_E_ERROR
    }

    let mut handle = FileHandle {
        magic: FILE_HANDLE_MAGIC,
        position: 0,
        mode,
        name: [0; 128],
    };
    handle.name[..name.len()].copy_from_slice(name.as_bytes());
    let fd = context.alloc_raw(size_of::<FileHandle>() as u32)?;
    write_generic(context, fd, handle)?;
    Ok(fd as i32)
}

pub async fn close(context: &mut dyn WIPICContext, fd: i32) -> Result<i32> {
    tracing::debug!("MC_fsClose({fd:#x})");

    if load_handle(context, fd).is_none() {
        return Ok(-2); // M_E_BADFD
    }
    write_generic(context, fd as u32, 0u32)?;
    context.free_raw(fd as u32, size_of::<FileHandle>() as u32)?;
    Ok(0)
}

pub async fn read(context: &mut dyn WIPICContext, fd: i32, buffer: u32, length: i32) -> Result<i32> {
    tracing::debug!("MC_fsRead({fd:#x}, {buffer:#x}, {length})");

    let Some((mut handle, name)) = load_handle(context, fd) else {
        return Ok(-2); // M_E_BADFD
    };
    if matches!(handle.mode, OPEN_WRITE_ONLY | OPEN_WRITE_TRUNCATE) {
        return Ok(-24); // M_E_ACCESS
    }
    if length < 0 {
        return Ok(-1);
    }
    if length == 0 {
        return Ok(0);
    }
    let filesystem = context.system().filesystem().clone();
    let Some(size) = filesystem.size(&name).await else {
        return Ok(-12); // M_E_NOENT
    };
    let count = (length as usize).min(size.saturating_sub(handle.position as usize));
    if count == 0 {
        return Ok(-23); // M_E_EOF
    }
    let mut data = vec![0; count];
    let Some(count) = filesystem.read(&name, handle.position as usize, count, &mut data).await else {
        return Ok(-1);
    };
    let Some(position) = handle.position.checked_add(count as i32) else {
        return Ok(-1);
    };
    context.write_bytes(buffer, &data[..count])?;
    handle.position = position;
    write_generic(context, fd as u32, handle)?;
    Ok(count as i32)
}

pub async fn write(context: &mut dyn WIPICContext, fd: i32, buffer: u32, length: i32) -> Result<i32> {
    tracing::debug!("MC_fsWrite({fd:#x}, {buffer:#x}, {length})");

    let Some((mut handle, name)) = load_handle(context, fd) else {
        return Ok(-2); // M_E_BADFD
    };
    if handle.mode == OPEN_READ_ONLY {
        return Ok(-24); // M_E_ACCESS
    }
    if length < 0 || handle.position.checked_add(length).is_none() {
        return Ok(-1);
    }
    let mut data = vec![0; length as usize];
    context.read_bytes(buffer, &mut data)?;
    let count = context.system().filesystem().write(&name, handle.position as usize, &data).await;
    if count == 0 && length != 0 {
        return Ok(-1);
    }
    handle.position += count as i32;
    write_generic(context, fd as u32, handle)?;
    Ok(count as i32)
}

pub async fn seek(context: &mut dyn WIPICContext, fd: i32, offset: i32, origin: i32) -> Result<i32> {
    tracing::debug!("MC_fsSeek({fd:#x}, {offset}, {origin})");

    let Some((mut handle, name)) = load_handle(context, fd) else {
        return Ok(-2); // M_E_BADFD
    };
    let base = match origin {
        0 => 0,
        1 => i64::from(handle.position),
        2 => match context.system().filesystem().size(&name).await {
            Some(size) => size as i64,
            None => return Ok(-12),
        },
        _ => return Ok(-1),
    };
    let Ok(position) = i32::try_from(base + i64::from(offset)) else {
        return Ok(-1);
    };
    if position < 0 {
        return Ok(-1);
    }
    handle.position = position;
    write_generic(context, fd as u32, handle)?;
    Ok(position)
}

pub async fn tell(context: &mut dyn WIPICContext, fd: i32) -> Result<i32> {
    tracing::debug!("MC_fsTell({fd:#x})");

    let Some((handle, _)) = load_handle(context, fd) else {
        return Ok(-2); // M_E_BADFD
    };
    Ok(handle.position)
}

pub async fn list(_context: &mut dyn WIPICContext, name: u32, buffer: u32, buffer_size: i32, access_mode: i32) -> Result<i32> {
    tracing::warn!("stub MC_fsList({name:#x}, {buffer:#x}, {buffer_size}, {access_mode})");

    Ok(-1) // M_E_ERROR
}

pub async fn file_attribute(context: &mut dyn WIPICContext, name: u32, output: u32, access_mode: i32) -> Result<i32> {
    tracing::debug!("MC_fsFileAttribute({name:#x}, {output:#x}, {access_mode})");
    let Ok(name) = String::from_utf8(read_null_terminated_string_bytes(context, name)?) else {
        return Ok(-3);
    };
    let filesystem = context.system().filesystem().clone();
    if !filesystem.is_valid_path(&name) {
        return Ok(-3);
    }
    let Some(size) = filesystem.size(&name).await else {
        return Ok(-12);
    };
    write_generic(context, output, [0u32, 0, size as u32])?;
    Ok(0)
}

pub async fn exists(context: &mut dyn WIPICContext, name: u32, access_mode: i32) -> Result<i32> {
    tracing::debug!("MC_fsIsExist({name:#x}, {access_mode})");
    let Ok(name) = String::from_utf8(read_null_terminated_string_bytes(context, name)?) else {
        return Ok(-3);
    };
    let filesystem = context.system().filesystem().clone();
    if !filesystem.is_valid_path(&name) {
        return Ok(-3);
    }
    if filesystem.exists(&name).await { Ok(0) } else { Ok(-12) }
}

pub async fn remove(context: &mut dyn WIPICContext, name: u32, access_mode: i32) -> Result<i32> {
    tracing::debug!("MC_fsRemove({name:#x}, {access_mode})");
    let Ok(name) = String::from_utf8(read_null_terminated_string_bytes(context, name)?) else {
        return Ok(-3);
    };
    let filesystem = context.system().filesystem();
    if !filesystem.is_valid_path(&name) {
        return Ok(-3);
    }
    // Packaged resources are read-only; only persistent files can be removed.
    if filesystem.remove(&name).await { Ok(0) } else { Ok(-12) }
}

pub async fn total_space(_context: &mut dyn WIPICContext) -> Result<i32> {
    tracing::debug!("MC_fsTotalSpace()");

    Ok(STORAGE_LIMIT as i32)
}

pub async fn available(context: &mut dyn WIPICContext) -> Result<i32> {
    tracing::debug!("MC_fsAvailable()");

    Ok(STORAGE_LIMIT.saturating_sub(context.system().filesystem().usage().await) as i32)
}

pub async fn rename(_context: &mut dyn WIPICContext, old_name: u32, new_name: u32, access_mode: i32) -> Result<i32> {
    tracing::warn!("stub MC_fsRename({old_name:#x}, {new_name:#x}, {access_mode})");
    Ok(-1)
}

pub async fn make_directory(_context: &mut dyn WIPICContext, name: u32, access_mode: i32) -> Result<i32> {
    tracing::warn!("stub MC_fsMkDir({name:#x}, {access_mode})");
    Ok(-1)
}

pub async fn remove_directory(_context: &mut dyn WIPICContext, name: u32, access_mode: i32) -> Result<i32> {
    tracing::warn!("stub MC_fsRmDir({name:#x}, {access_mode})");
    Ok(-1)
}

pub async fn set_mode(_context: &mut dyn WIPICContext, name: u32, mode: i32, access_mode: i32) -> Result<i32> {
    tracing::warn!("stub MC_fsSetMode({name:#x}, {mode}, {access_mode})");
    Ok(-1)
}

pub async fn get_counts(_context: &mut dyn WIPICContext, name: u32, access_mode: i32) -> Result<i32> {
    tracing::warn!("stub MC_fsGetCounts({name:#x}, {access_mode})");
    Ok(-1)
}

fn load_handle(context: &mut dyn WIPICContext, fd: i32) -> Option<(FileHandle, String)> {
    if fd <= 0 {
        return None;
    }
    let handle = read_generic::<FileHandle, _>(context, fd as u32).ok()?;
    if handle.magic != FILE_HANDLE_MAGIC {
        return None;
    }
    let length = handle.name.iter().position(|&byte| byte == 0).unwrap_or(handle.name.len());
    let name = str::from_utf8(&handle.name[..length]).ok()?.to_owned();
    Some((handle, name))
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use test_utils::TestPlatform;
    use wie_backend::{DefaultTaskRunner, System};
    use wie_util::{ByteRead, ByteWrite, read_generic};

    use crate::{WIPICContext, context::test::TestContext};

    use super::{OPEN_READ_ONLY, STORAGE_LIMIT, available, close, exists, file_attribute, open, read, remove, seek, tell, write};

    #[futures_test::test]
    async fn file_cursor_and_persistence_are_independent_of_database_records() {
        let system = System::new(Box::new(TestPlatform::new()), "test-pid", "test-aid", DefaultTaskRunner);
        let mut context = TestContext::with_system(system);
        context.write_bytes(0x1000, b"save\0").unwrap();
        context.write_bytes(0x2000, &[1, 2, 3, 4]).unwrap();

        let mut database = context.system().platform().database_repository().open("save", "test-pid").await;
        assert!(database.set(1, &[9, 8]).await);
        assert_eq!(open(&mut context, 0x1000, 1, 1).await.unwrap(), -12);
        assert_eq!(exists(&mut context, 0x1000, 1).await.unwrap(), -12);
        assert_eq!(available(&mut context).await.unwrap(), STORAGE_LIMIT as i32);

        let fd = open(&mut context, 0x1000, 8, 1).await.unwrap();
        assert!(fd > 0);
        assert_eq!(tell(&mut context, fd).await.unwrap(), 0);
        assert_eq!(write(&mut context, fd, 0x2000, 4).await.unwrap(), 4);
        assert_eq!(context.system().platform().filesystem().size("test-pid", "save").await, Some(4));
        assert!(!context.system().platform().filesystem().exists("test-aid", "save").await);
        assert_eq!(tell(&mut context, fd).await.unwrap(), 4);
        assert_eq!(seek(&mut context, fd, -2, 1).await.unwrap(), 2);
        assert_eq!(read(&mut context, fd, 0x2100, 2).await.unwrap(), 2);
        assert_eq!(tell(&mut context, fd).await.unwrap(), 4);
        let mut tail = [0; 2];
        context.read_bytes(0x2100, &mut tail).unwrap();
        assert_eq!(tail, [3, 4]);
        assert_eq!(seek(&mut context, fd, 2, 2).await.unwrap(), 6);
        assert_eq!(write(&mut context, fd, 0x2000, 2).await.unwrap(), 2);
        assert_eq!(available(&mut context).await.unwrap(), STORAGE_LIMIT as i32 - 8);
        assert_eq!(file_attribute(&mut context, 0x1000, 0x2200, 1).await.unwrap(), 0);
        assert_eq!(read_generic::<[u32; 3], _>(&context, 0x2200).unwrap(), [0, 0, 8]);
        assert_eq!(close(&mut context, fd).await.unwrap(), 0);
        assert_eq!(tell(&mut context, fd).await.unwrap(), -2);

        let fd = open(&mut context, 0x1000, 1, 1).await.unwrap();
        assert_eq!(read(&mut context, fd, 0x2100, 8).await.unwrap(), 8);
        let mut contents = [0; 8];
        context.read_bytes(0x2100, &mut contents).unwrap();
        assert_eq!(contents, [1, 2, 3, 4, 0, 0, 1, 2]);
        assert_eq!(read(&mut context, fd, 0, 1).await.unwrap(), -23);
        assert_eq!(write(&mut context, fd, 0x2000, 1).await.unwrap(), -24);
        assert_eq!(seek(&mut context, fd, -1, 0).await.unwrap(), -1);
        assert_eq!(close(&mut context, fd).await.unwrap(), 0);

        let fd = open(&mut context, 0x1000, 4, 1).await.unwrap();
        assert_eq!(context.system().filesystem().size("save").await, Some(0));
        assert_eq!(read(&mut context, fd, 0, 1).await.unwrap(), -24);
        assert_eq!(close(&mut context, fd).await.unwrap(), 0);
        assert_eq!(exists(&mut context, 0x1000, 1).await.unwrap(), 0);
        assert_eq!(remove(&mut context, 0x1000, 1).await.unwrap(), 0);
        assert_eq!(exists(&mut context, 0x1000, 1).await.unwrap(), -12);
        assert_eq!(available(&mut context).await.unwrap(), STORAGE_LIMIT as i32);
        assert_eq!(database.get(1).await.unwrap(), [9, 8]);
    }

    #[futures_test::test]
    async fn filesystem_overlay_is_independent_of_resources_and_rejects_invalid_paths() {
        let system = System::new(Box::new(TestPlatform::new()), "test-pid", "test-aid", DefaultTaskRunner);
        let mut context = TestContext::with_system(system).with_resource("data/seed", b"seed");
        context.write_bytes(0x1000, b"data/seed\0").unwrap();

        assert_eq!(open(&mut context, 0x1000, OPEN_READ_ONLY, 1).await.unwrap(), -12);
        assert_eq!(exists(&mut context, 0x1000, 1).await.unwrap(), -12);
        assert_eq!(file_attribute(&mut context, 0x1000, 0x2100, 1).await.unwrap(), -12);
        context.system().filesystem().add_virtual("data/seed", b"seed".to_vec());

        let fd = open(&mut context, 0x1000, 8, 1).await.unwrap();
        assert_eq!(read(&mut context, fd, 0x2000, 4).await.unwrap(), 4);
        let mut contents = [0; 4];
        context.read_bytes(0x2000, &mut contents).unwrap();
        assert_eq!(&contents, b"seed");
        assert_eq!(seek(&mut context, fd, 0, 0).await.unwrap(), 0);
        context.write_bytes(0x2000, b"n").unwrap();
        assert_eq!(write(&mut context, fd, 0x2000, 1).await.unwrap(), 1);
        assert_eq!(close(&mut context, fd).await.unwrap(), 0);
        assert_eq!(context.system().filesystem().read("data/seed", 0, 4, &mut contents).await, Some(4));
        assert_eq!(&contents, b"need");
        assert!(!context.system().platform().database_repository().exists("data/seed", "test-pid").await);

        for path in [b"../save\0".as_slice(), b"\xff\0"] {
            context.write_bytes(0x1000, path).unwrap();
            assert_eq!(open(&mut context, 0x1000, 4, 1).await.unwrap(), -3);
        }
    }
}
