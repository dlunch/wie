use alloc::{borrow::ToOwned, boxed::Box, string::String, sync::Arc, vec::Vec};
use core::cmp::min;

use hashbrown::HashMap;
use spin::Mutex;

use crate::platform::Platform;

/// Normalize a guest-supplied path so both overlay layers see the same key.
///
/// - Leading `/` are stripped (archive paths often carry them).
/// - `.` segments are dropped.
/// - `..` segments, trailing `/`, backslashes, and empty results all
///   return `None`.
fn normalize_guest_path(path: &str) -> Option<String> {
    if path.contains('\\') {
        return None;
    }

    let trimmed = path.trim_start_matches('/');
    if trimmed.is_empty() || trimmed.ends_with('/') {
        return None;
    }

    let mut out = String::new();
    for seg in trimmed.split('/') {
        match seg {
            "" => continue,
            "." => continue,
            ".." => return None,
            normal => {
                if !out.is_empty() {
                    out.push('/');
                }
                out.push_str(normal);
            }
        }
    }

    if out.is_empty() { None } else { Some(out) }
}

/// Unified filesystem view exposed by `System::filesystem()`.
///
/// Wraps the persistent `Platform::filesystem()` backend and an in-memory
/// virtual layer holding archive resources. Writes always hit the platform
/// backend; reads prefer the platform backend and fall back to the virtual
/// layer. Paths are normalized internally so callers pass raw guest paths.
#[derive(Clone)]
pub struct FilesystemOverlay {
    platform: Arc<Box<dyn Platform>>,
    virtual_files: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    aid: Arc<str>,
}

impl FilesystemOverlay {
    pub fn new(platform: Arc<Box<dyn Platform>>, aid: &str) -> Self {
        Self {
            platform,
            virtual_files: Arc::new(Mutex::new(HashMap::new())),
            aid: Arc::from(aid),
        }
    }

    pub fn add_virtual(&self, path: &str, data: Vec<u8>) {
        let key = normalize_guest_path(path).unwrap_or_else(|| path.trim_start_matches('/').to_owned());
        self.virtual_files.lock().insert(key, data);
    }

    pub fn is_valid_path(&self, path: &str) -> bool {
        normalize_guest_path(path).is_some()
    }

    async fn materialize_virtual(&self, normalized: &str) -> bool {
        let filesystem = self.platform.filesystem();
        if filesystem.exists(&self.aid, normalized).await {
            return true;
        }

        let virtual_data = self.virtual_files.lock().get(normalized).cloned();
        let Some(virtual_data) = virtual_data else {
            return true;
        };

        let written = filesystem.write(&self.aid, normalized, 0, &virtual_data).await;
        if written != virtual_data.len() {
            tracing::warn!(
                path = normalized,
                expected = virtual_data.len(),
                written,
                "failed to materialize virtual file"
            );
            return false;
        }

        true
    }

    pub async fn exists(&self, path: &str) -> bool {
        let Some(normalized) = normalize_guest_path(path) else {
            return false;
        };

        if self.platform.filesystem().exists(&self.aid, &normalized).await {
            return true;
        }
        self.virtual_files.lock().contains_key(&normalized)
    }

    pub async fn size(&self, path: &str) -> Option<usize> {
        let normalized = normalize_guest_path(path)?;

        if let Some(size) = self.platform.filesystem().size(&self.aid, &normalized).await {
            return Some(size);
        }
        self.virtual_files.lock().get(&normalized).map(|d| d.len())
    }

    pub async fn read(&self, path: &str, offset: usize, count: usize, buf: &mut [u8]) -> Option<usize> {
        let normalized = normalize_guest_path(path)?;

        let plat_fs = self.platform.filesystem();
        if plat_fs.exists(&self.aid, &normalized).await {
            return plat_fs.read(&self.aid, &normalized, offset, count, buf).await;
        }

        let files = self.virtual_files.lock();
        let data = files.get(&normalized)?;
        if offset >= data.len() {
            return Some(0);
        }
        let n = min(count, data.len() - offset);
        buf[..n].copy_from_slice(&data[offset..offset + n]);
        Some(n)
    }

    pub async fn write(&self, path: &str, offset: usize, data: &[u8]) -> usize {
        let Some(normalized) = normalize_guest_path(path) else {
            return 0;
        };
        if !self.materialize_virtual(&normalized).await {
            return 0;
        }
        self.platform.filesystem().write(&self.aid, &normalized, offset, data).await
    }

    pub async fn truncate(&self, path: &str, len: usize) -> bool {
        let Some(normalized) = normalize_guest_path(path) else {
            return false;
        };
        if !self.materialize_virtual(&normalized).await {
            return false;
        }
        let filesystem = self.platform.filesystem();
        filesystem.truncate(&self.aid, &normalized, len).await;
        filesystem.size(&self.aid, &normalized).await == Some(len)
    }
}

#[cfg(test)]
mod tests {
    use alloc::{
        boxed::Box,
        string::{String, ToString},
        sync::Arc,
        vec,
        vec::Vec,
    };

    use hashbrown::HashMap;
    use spin::Mutex;

    use crate::{
        audio_sink::AudioSink,
        database::DatabaseRepository,
        platform::{Filesystem, Platform},
        screen::Screen,
        time::Instant,
    };

    use super::FilesystemOverlay;

    #[derive(Default)]
    struct StubFilesystem {
        files: Mutex<HashMap<(String, String), Vec<u8>>>,
        write_limit: Option<usize>,
        fail_truncate: bool,
    }
    #[async_trait::async_trait]
    impl Filesystem for StubFilesystem {
        async fn exists(&self, aid: &str, path: &str) -> bool {
            self.files.lock().contains_key(&(aid.to_string(), path.to_string()))
        }
        async fn size(&self, aid: &str, path: &str) -> Option<usize> {
            self.files.lock().get(&(aid.to_string(), path.to_string())).map(|v| v.len())
        }
        async fn read(&self, aid: &str, path: &str, offset: usize, count: usize, buf: &mut [u8]) -> Option<usize> {
            let files = self.files.lock();
            let data = files.get(&(aid.to_string(), path.to_string()))?;
            if offset >= data.len() {
                return Some(0);
            }
            let n = core::cmp::min(count, data.len() - offset);
            buf[..n].copy_from_slice(&data[offset..offset + n]);
            Some(n)
        }
        async fn write(&self, aid: &str, path: &str, offset: usize, data: &[u8]) -> usize {
            let write_len = self.write_limit.unwrap_or(data.len()).min(data.len());
            let mut files = self.files.lock();
            let file = files.entry((aid.to_string(), path.to_string())).or_default();
            if file.len() < offset + write_len {
                file.resize(offset + write_len, 0);
            }
            file[offset..offset + write_len].copy_from_slice(&data[..write_len]);
            write_len
        }
        async fn truncate(&self, aid: &str, path: &str, len: usize) {
            if self.fail_truncate {
                return;
            }
            let mut files = self.files.lock();
            let file = files.entry((aid.to_string(), path.to_string())).or_default();
            file.resize(len, 0);
        }
    }

    struct StubPlatform {
        fs: StubFilesystem,
    }
    impl Platform for StubPlatform {
        fn screen(&self) -> &dyn Screen {
            unimplemented!()
        }
        fn now(&self) -> Instant {
            Instant::from_epoch_millis(0)
        }
        fn database_repository(&self) -> &dyn DatabaseRepository {
            unimplemented!()
        }
        fn filesystem(&self) -> &dyn Filesystem {
            &self.fs
        }
        fn audio_sink(&self) -> Box<dyn AudioSink> {
            unimplemented!()
        }
        fn write_stdout(&self, _buf: &[u8]) {}
        fn write_stderr(&self, _buf: &[u8]) {}
        fn exit(&self) {}
        fn vibrate(&self, _duration_ms: u64, _intensity: u8) {}
    }

    fn setup() -> FilesystemOverlay {
        setup_with_filesystem(StubFilesystem::default())
    }

    fn setup_with_filesystem(fs: StubFilesystem) -> FilesystemOverlay {
        let platform: Arc<Box<dyn Platform>> = Arc::new(Box::new(StubPlatform { fs }));
        FilesystemOverlay::new(platform, "test-aid")
    }

    #[futures_test::test]
    async fn add_then_read_virtual() {
        let fs = setup();
        fs.add_virtual("a.bin", vec![1, 2, 3, 4]);

        let mut buf = [0u8; 4];
        assert_eq!(fs.read("a.bin", 0, 4, &mut buf).await, Some(4));
        assert_eq!(buf, [1, 2, 3, 4]);
    }

    #[futures_test::test]
    async fn size_falls_through_to_virtual() {
        let fs = setup();
        fs.add_virtual("x", vec![0; 17]);

        assert_eq!(fs.size("x").await, Some(17));
        assert_eq!(fs.size("nope").await, None);
    }

    #[futures_test::test]
    async fn exists_checks_both_layers() {
        let fs = setup();
        fs.add_virtual("x", vec![1]);

        assert!(fs.exists("x").await);
        assert!(!fs.exists("y").await);

        fs.write("written", 0, &[9]).await;
        assert!(fs.exists("written").await);
    }

    #[futures_test::test]
    async fn leading_slash_normalized() {
        let fs = setup();
        fs.add_virtual("/a/b", vec![9]);

        assert!(fs.exists("a/b").await);
        assert!(fs.exists("/a/b").await);
    }

    #[futures_test::test]
    async fn read_past_eof_virtual_returns_some_zero() {
        let fs = setup();
        fs.add_virtual("a", vec![1, 2, 3]);

        let mut buf = [0u8; 4];
        assert_eq!(fs.read("a", 10, 4, &mut buf).await, Some(0));
    }

    #[futures_test::test]
    async fn read_missing_returns_none() {
        let fs = setup();
        let mut buf = [0u8; 4];
        assert_eq!(fs.read("nope", 0, 4, &mut buf).await, None);
    }

    #[futures_test::test]
    async fn platform_write_shadows_virtual() {
        let fs = setup();
        fs.add_virtual("cfg.dat", vec![0xAA, 0xBB, 0xCC]);
        fs.write("cfg.dat", 0, &[1, 2, 3, 4]).await;

        let mut buf = [0u8; 4];
        assert_eq!(fs.read("cfg.dat", 0, 4, &mut buf).await, Some(4));
        assert_eq!(buf, [1, 2, 3, 4]);
    }

    #[futures_test::test]
    async fn first_persistent_write_materializes_virtual_prefix() {
        let fs = setup();
        fs.add_virtual("append.dat", vec![0xAA, 0xBB]);

        assert_eq!(fs.write("append.dat", 2, &[0xCC]).await, 1);

        let mut buf = [0u8; 3];
        assert_eq!(fs.read("append.dat", 0, 3, &mut buf).await, Some(3));
        assert_eq!(buf, [0xAA, 0xBB, 0xCC]);
    }

    #[futures_test::test]
    async fn first_persistent_truncate_materializes_virtual_data() {
        let fs = setup();
        fs.add_virtual("truncate.dat", vec![1, 2, 3, 4]);

        assert!(fs.truncate("truncate.dat", 2).await);

        let mut buf = [0u8; 2];
        assert_eq!(fs.read("truncate.dat", 0, 2, &mut buf).await, Some(2));
        assert_eq!(buf, [1, 2]);
    }

    #[futures_test::test]
    async fn persistent_backend_exposes_short_writes_and_failed_truncation() {
        let short_write_fs = setup_with_filesystem(StubFilesystem {
            write_limit: Some(2),
            ..Default::default()
        });
        assert_eq!(short_write_fs.write("short.dat", 0, &[1, 2, 3, 4]).await, 2);

        let failed_truncate_fs = setup_with_filesystem(StubFilesystem {
            fail_truncate: true,
            ..Default::default()
        });
        assert_eq!(failed_truncate_fs.write("truncate.dat", 0, &[1, 2, 3, 4]).await, 4);
        assert!(!failed_truncate_fs.truncate("truncate.dat", 2).await);
        assert_eq!(failed_truncate_fs.size("truncate.dat").await, Some(4));
    }
}
