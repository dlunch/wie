use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};
use rustjava_runtime::classes::java::{io::File as JavaFile, lang::String, util::Vector};

use wie_jvm_support::WieJvmContext;

// class org.kwis.msp.io.FileSystem
#[java_class(name = "org/kwis/msp/io/FileSystem", public, final)]
pub struct FileSystem;

#[java_methods]
impl FileSystem {
    #[method(constructor, public)]
    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.io.FileSystem::<init>({this:?})");

        jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await
    }

    #[method(public, static)]
    async fn is_file(jvm: &Jvm, _: &mut WieJvmContext, name: ClassInstanceRef<String>) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.io.FileSystem::is_file({name:?})");

        let file = jvm.new_class("java/io/File", "(Ljava/lang/String;)V", (name,)).await?;
        let is_file = jvm.invoke_virtual(&file, "java/io/File", "isFile", "()Z", ()).await?;

        Ok(is_file)
    }

    #[method(public, static)]
    async fn is_directory(jvm: &Jvm, _: &mut WieJvmContext, name: ClassInstanceRef<String>, flag: i32) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.io.FileSystem::isDirectory({name:?}, {flag:?})");

        let file = jvm.new_class("java/io/File", "(Ljava/lang/String;)V", (name,)).await?;
        let is_directory = jvm.invoke_virtual(&file, "java/io/File", "isDirectory", "()Z", ()).await?;

        Ok(is_directory)
    }

    #[method(public, static)]
    async fn exists(jvm: &Jvm, _context: &mut WieJvmContext, name: ClassInstanceRef<String>) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.io.FileSystem::exists({name:?})");

        jvm.invoke_static("org/kwis/msp/io/FileSystem", "exists", "(Ljava/lang/String;I)Z", (name, 0))
            .await
    }

    #[method(name = "exists", public, static)]
    async fn exists_with_flag(jvm: &Jvm, _context: &mut WieJvmContext, name: ClassInstanceRef<String>, flag: i32) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.io.FileSystem::exists({name:?}, {flag:?})");

        let file = jvm.new_class("java/io/File", "(Ljava/lang/String;)V", (name,)).await?;
        let exists = jvm.invoke_virtual(&file, "java/io/File", "exists", "()Z", ()).await?;

        Ok(exists)
    }

    #[method(public, static)]
    async fn mkdir(_jvm: &Jvm, _context: &mut WieJvmContext, name: ClassInstanceRef<String>, flag: i32) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.io.FileSystem::mkdir({name:?}, {flag:?})");

        Ok(())
    }

    #[method(public, static)]
    async fn available(_: &Jvm, _: &mut WieJvmContext) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.io.FileSystem::available()");

        Ok(0x1000000) // TODO temp
    }

    #[method(public, static)]
    async fn get_max_filename_length(_: &Jvm, _: &mut WieJvmContext) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.io.FileSystem::getMaxFilenameLength()");

        Ok(0)
    }

    #[method(public, static)]
    async fn list(_: &Jvm, _: &mut WieJvmContext, dirname: ClassInstanceRef<String>) -> JvmResult<ClassInstanceRef<Vector>> {
        tracing::warn!("stub org.kwis.msp.io.FileSystem::list({dirname:?})");

        Ok(ClassInstanceRef::new(None))
    }

    #[method(name = "list", public, static)]
    async fn list_with_flag(_: &Jvm, _: &mut WieJvmContext, dirname: ClassInstanceRef<String>, flag: i32) -> JvmResult<ClassInstanceRef<Vector>> {
        tracing::warn!("stub org.kwis.msp.io.FileSystem::list({dirname:?}, {flag})");

        Ok(ClassInstanceRef::new(None))
    }

    #[method(public, static)]
    async fn remove(_: &Jvm, _: &mut WieJvmContext, filename: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.io.FileSystem::remove({filename:?})");

        Ok(())
    }

    #[method(name = "remove", public, static)]
    async fn remove_with_flag(_: &Jvm, _: &mut WieJvmContext, filename: ClassInstanceRef<String>, flag: i32) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.io.FileSystem::remove({filename:?}, {flag})");

        Ok(())
    }

    #[method(name = "mkdir", public, static)]
    async fn mkdir_without_flag(_: &Jvm, _: &mut WieJvmContext, dirname: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.io.FileSystem::mkdir({dirname:?})");

        Ok(())
    }

    #[method(public, static)]
    async fn rmdir(_: &Jvm, _: &mut WieJvmContext, dirname: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.io.FileSystem::rmdir({dirname:?})");

        Ok(())
    }

    #[method(name = "rmdir", public, static)]
    async fn rmdir_with_flag(_: &Jvm, _: &mut WieJvmContext, dirname: ClassInstanceRef<String>, flag: i32) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.io.FileSystem::rmdir({dirname:?}, {flag})");

        Ok(())
    }

    #[method(public, static)]
    async fn to_c_string(_: &Jvm, _: &mut WieJvmContext, value: ClassInstanceRef<String>) -> JvmResult<ClassInstanceRef<Array<i8>>> {
        tracing::warn!("stub org.kwis.msp.io.FileSystem::toCString({value:?})");

        Ok(ClassInstanceRef::new(None))
    }

    #[method(name = "isFile", public, static)]
    async fn is_file_with_flag(_: &Jvm, _: &mut WieJvmContext, name: ClassInstanceRef<String>, flag: i32) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.io.FileSystem::isFile({name:?}, {flag})");

        Ok(false)
    }

    #[method(name = "isDirectory", public, static)]
    async fn is_directory_without_flag(jvm: &Jvm, _: &mut WieJvmContext, name: ClassInstanceRef<String>) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.io.FileSystem::isDirectory({name:?})");

        let file: ClassInstanceRef<JavaFile> = jvm.new_class("java/io/File", "(Ljava/lang/String;)V", (name,)).await?.into();
        jvm.invoke_virtual(&file, "java/io/File", "isDirectory", "()Z", ()).await
    }

    #[method(public, static)]
    async fn get_creation_time(_: &Jvm, _: &mut WieJvmContext, name: ClassInstanceRef<String>) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.io.FileSystem::getCreationTime({name:?})");

        Ok(0)
    }

    #[method(name = "getCreationTime", public, static)]
    async fn get_creation_time_with_flag(_: &Jvm, _: &mut WieJvmContext, name: ClassInstanceRef<String>, flag: i32) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.io.FileSystem::getCreationTime({name:?}, {flag})");

        Ok(0)
    }

    #[method(public, static)]
    async fn rename(_: &Jvm, _: &mut WieJvmContext, old_name: ClassInstanceRef<String>, new_name: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.io.FileSystem::rename({old_name:?}, {new_name:?})");

        Ok(())
    }

    #[method(name = "rename", public, static)]
    async fn rename_with_flag(
        _: &Jvm,
        _: &mut WieJvmContext,
        old_name: ClassInstanceRef<String>,
        new_name: ClassInstanceRef<String>,
        flag: i32,
    ) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.io.FileSystem::rename({old_name:?}, {new_name:?}, {flag})");

        Ok(())
    }
}
