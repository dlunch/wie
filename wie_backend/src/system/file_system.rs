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
        self.platform.filesystem().write(&self.aid, &normalized, offset, data).await
    }

    pub async fn truncate(&self, path: &str, len: usize) {
        let Some(normalized) = normalize_guest_path(path) else {
            return;
        };
        self.platform.filesystem().truncate(&self.aid, &normalized, len).await;
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
            let mut files = self.files.lock();
            let file = files.entry((aid.to_string(), path.to_string())).or_default();
            if file.len() < offset + data.len() {
                file.resize(offset + data.len(), 0);
            }
            file[offset..offset + data.len()].copy_from_slice(data);
            data.len()
        }
        async fn truncate(&self, aid: &str, path: &str, len: usize) {
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
        let platform: Arc<Box<dyn Platform>> = Arc::new(Box::new(StubPlatform {
            fs: StubFilesystem::default(),
        }));
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
}
