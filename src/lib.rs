mod loader;
mod save_migration;

pub use loader::{AppMetadata, FileType, determine_file_type, extract_app_metadata, load_emulator};
pub use save_migration::save_migration_identity;
