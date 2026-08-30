use alloc::boxed::Box;

use crate::{audio_sink::AudioSink, database::DatabaseRepository, screen::Screen, time::Instant};

pub trait Platform: Send + Sync {
    fn screen(&self) -> &dyn Screen;
    fn now(&self) -> Instant;
    fn database_repository(&self) -> &dyn DatabaseRepository;
    fn filesystem(&self) -> &dyn Filesystem;
    fn audio_sink(&self) -> Box<dyn AudioSink>;
    fn write_stdout(&self, buf: &[u8]);
    fn write_stderr(&self, buf: &[u8]);
    fn exit(&self);
    fn vibrate(&self, duration_ms: u64, intensity: u8);
}

/// Platform filesystem abstraction. Every method is scoped by `aid`;
/// implementations MUST NOT cross aid boundaries.
#[async_trait::async_trait]
pub trait Filesystem: Send + Sync {
    async fn exists(&self, aid: &str, path: &str) -> bool;

    async fn size(&self, aid: &str, path: &str) -> Option<usize>;

    /// Read up to `count` bytes starting at `offset` into `buf[..count]`.
    ///
    /// - File missing → `None`.
    /// - `offset >= size` (read past EOF) → `Some(0)`.
    /// - Otherwise → `Some(n)` where `0 < n <= count`. Short reads allowed
    ///   at end of file.
    /// - Caller guarantees `buf.len() >= count`. Implementations only write
    ///   to `buf[..n]`.
    async fn read(&self, aid: &str, path: &str, offset: usize, count: usize, buf: &mut [u8]) -> Option<usize>;

    /// Write `data` starting at `offset`.
    ///
    /// - Creates the file (and any missing intermediate directories) if it
    ///   does not yet exist. A zero-length `data` is a valid way to
    ///   materialize an empty file.
    /// - If `offset + data.len() > current_size` the implementation MUST
    ///   automatically extend the file, zero-filling the gap.
    /// - Returns the number of bytes actually written. On success this
    ///   equals `data.len()`.
    /// - On failure (path rejected, disk full, permission denied, etc.)
    ///   MUST return `0` and log via `tracing::warn!` or `tracing::error!`.
    ///   Silent `0` returns are forbidden.
    async fn write(&self, aid: &str, path: &str, offset: usize, data: &[u8]) -> usize;

    /// Truncate the file to exactly `len` bytes. Creates the file if
    /// missing.
    /// - `len > current_size` → zero-fill extend.
    /// - `len < current_size` → tail bytes dropped.
    async fn truncate(&self, aid: &str, path: &str, len: usize);
}
