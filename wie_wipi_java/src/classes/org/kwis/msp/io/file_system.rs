use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_constants::MethodAccessFlags;
use java_runtime::classes::java::{io::File as JavaFile, lang::String, util::Vector};
use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class org.kwis.msp.io.FileSystem
pub struct FileSystem;

impl FileSystem {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/io/FileSystem",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, Default::default()),
                JavaMethodProto::new("isFile", "(Ljava/lang/String;)Z", Self::is_file, MethodAccessFlags::STATIC),
                JavaMethodProto::new("isDirectory", "(Ljava/lang/String;I)Z", Self::is_directory, MethodAccessFlags::STATIC),
                JavaMethodProto::new("exists", "(Ljava/lang/String;)Z", Self::exists, MethodAccessFlags::STATIC),
                JavaMethodProto::new("exists", "(Ljava/lang/String;I)Z", Self::exists_with_flag, MethodAccessFlags::STATIC),
                JavaMethodProto::new("mkdir", "(Ljava/lang/String;I)V", Self::mkdir, MethodAccessFlags::STATIC),
                JavaMethodProto::new("available", "()I", Self::available, MethodAccessFlags::STATIC),
                JavaMethodProto::new("getMaxFilenameLength", "()I", Self::get_max_filename_length, MethodAccessFlags::STATIC),
                JavaMethodProto::new("list", "(Ljava/lang/String;)Ljava/util/Vector;", Self::list, MethodAccessFlags::STATIC),
                JavaMethodProto::new(
                    "list",
                    "(Ljava/lang/String;I)Ljava/util/Vector;",
                    Self::list_with_flag,
                    MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new("remove", "(Ljava/lang/String;)V", Self::remove, MethodAccessFlags::STATIC),
                JavaMethodProto::new("remove", "(Ljava/lang/String;I)V", Self::remove_with_flag, MethodAccessFlags::STATIC),
                JavaMethodProto::new("mkdir", "(Ljava/lang/String;)V", Self::mkdir_without_flag, MethodAccessFlags::STATIC),
                JavaMethodProto::new("rmdir", "(Ljava/lang/String;)V", Self::rmdir, MethodAccessFlags::STATIC),
                JavaMethodProto::new("rmdir", "(Ljava/lang/String;I)V", Self::rmdir_with_flag, MethodAccessFlags::STATIC),
                JavaMethodProto::new("toCString", "(Ljava/lang/String;)[B", Self::to_c_string, MethodAccessFlags::STATIC),
                JavaMethodProto::new("isFile", "(Ljava/lang/String;I)Z", Self::is_file_with_flag, MethodAccessFlags::STATIC),
                JavaMethodProto::new(
                    "isDirectory",
                    "(Ljava/lang/String;)Z",
                    Self::is_directory_without_flag,
                    MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "getCreationTime",
                    "(Ljava/lang/String;)I",
                    Self::get_creation_time,
                    MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "getCreationTime",
                    "(Ljava/lang/String;I)I",
                    Self::get_creation_time_with_flag,
                    MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "rename",
                    "(Ljava/lang/String;Ljava/lang/String;)V",
                    Self::rename,
                    MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "rename",
                    "(Ljava/lang/String;Ljava/lang/String;I)V",
                    Self::rename_with_flag,
                    MethodAccessFlags::STATIC,
                ),
            ],
            fields: vec![],
            access_flags: Default::default(),
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.io.FileSystem::<init>({this:?})");

        jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await
    }

    async fn is_file(jvm: &Jvm, _: &mut WieJvmContext, name: ClassInstanceRef<String>) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.io.FileSystem::is_file({name:?})");

        let file = jvm.new_class("java/io/File", "(Ljava/lang/String;)V", (name,)).await?;
        let is_file = jvm.invoke_virtual(&file, "isFile", "()Z", ()).await?;

        Ok(is_file)
    }

    async fn is_directory(jvm: &Jvm, _: &mut WieJvmContext, name: ClassInstanceRef<String>, flag: i32) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.io.FileSystem::isDirectory({name:?}, {flag:?})");

        let file = jvm.new_class("java/io/File", "(Ljava/lang/String;)V", (name,)).await?;
        let is_directory = jvm.invoke_virtual(&file, "isDirectory", "()Z", ()).await?;

        Ok(is_directory)
    }

    async fn exists(jvm: &Jvm, _context: &mut WieJvmContext, name: ClassInstanceRef<String>) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.io.FileSystem::exists({name:?})");

        jvm.invoke_static("org/kwis/msp/io/FileSystem", "exists", "(Ljava/lang/String;I)Z", (name, 0))
            .await
    }

    async fn exists_with_flag(jvm: &Jvm, _context: &mut WieJvmContext, name: ClassInstanceRef<String>, flag: i32) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.io.FileSystem::exists({name:?}, {flag:?})");

        let file = jvm.new_class("java/io/File", "(Ljava/lang/String;)V", (name,)).await?;
        let exists = jvm.invoke_virtual(&file, "exists", "()Z", ()).await?;

        Ok(exists)
    }

    async fn mkdir(_jvm: &Jvm, _context: &mut WieJvmContext, name: ClassInstanceRef<String>, flag: i32) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.io.FileSystem::mkdir({name:?}, {flag:?})");

        Ok(())
    }

    async fn available(_: &Jvm, _: &mut WieJvmContext) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.io.FileSystem::available()");

        Ok(0x1000000) // TODO temp
    }

    async fn get_max_filename_length(_: &Jvm, _: &mut WieJvmContext) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.io.FileSystem::getMaxFilenameLength()");

        Ok(0)
    }

    async fn list(_: &Jvm, _: &mut WieJvmContext, dirname: ClassInstanceRef<String>) -> JvmResult<ClassInstanceRef<Vector>> {
        tracing::warn!("stub org.kwis.msp.io.FileSystem::list({dirname:?})");

        Ok(ClassInstanceRef::new(None))
    }

    async fn list_with_flag(_: &Jvm, _: &mut WieJvmContext, dirname: ClassInstanceRef<String>, flag: i32) -> JvmResult<ClassInstanceRef<Vector>> {
        tracing::warn!("stub org.kwis.msp.io.FileSystem::list({dirname:?}, {flag})");

        Ok(ClassInstanceRef::new(None))
    }

    async fn remove(_: &Jvm, _: &mut WieJvmContext, filename: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.io.FileSystem::remove({filename:?})");

        Ok(())
    }

    async fn remove_with_flag(_: &Jvm, _: &mut WieJvmContext, filename: ClassInstanceRef<String>, flag: i32) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.io.FileSystem::remove({filename:?}, {flag})");

        Ok(())
    }

    async fn mkdir_without_flag(_: &Jvm, _: &mut WieJvmContext, dirname: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.io.FileSystem::mkdir({dirname:?})");

        Ok(())
    }

    async fn rmdir(_: &Jvm, _: &mut WieJvmContext, dirname: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.io.FileSystem::rmdir({dirname:?})");

        Ok(())
    }

    async fn rmdir_with_flag(_: &Jvm, _: &mut WieJvmContext, dirname: ClassInstanceRef<String>, flag: i32) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.io.FileSystem::rmdir({dirname:?}, {flag})");

        Ok(())
    }

    async fn to_c_string(_: &Jvm, _: &mut WieJvmContext, value: ClassInstanceRef<String>) -> JvmResult<ClassInstanceRef<Array<i8>>> {
        tracing::warn!("stub org.kwis.msp.io.FileSystem::toCString({value:?})");

        Ok(ClassInstanceRef::new(None))
    }

    async fn is_file_with_flag(_: &Jvm, _: &mut WieJvmContext, name: ClassInstanceRef<String>, flag: i32) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.io.FileSystem::isFile({name:?}, {flag})");

        Ok(false)
    }

    async fn is_directory_without_flag(jvm: &Jvm, _: &mut WieJvmContext, name: ClassInstanceRef<String>) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.io.FileSystem::isDirectory({name:?})");

        let file: ClassInstanceRef<JavaFile> = jvm.new_class("java/io/File", "(Ljava/lang/String;)V", (name,)).await?.into();
        jvm.invoke_virtual(&file, "isDirectory", "()Z", ()).await
    }

    async fn get_creation_time(_: &Jvm, _: &mut WieJvmContext, name: ClassInstanceRef<String>) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.io.FileSystem::getCreationTime({name:?})");

        Ok(0)
    }

    async fn get_creation_time_with_flag(_: &Jvm, _: &mut WieJvmContext, name: ClassInstanceRef<String>, flag: i32) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.io.FileSystem::getCreationTime({name:?}, {flag})");

        Ok(0)
    }

    async fn rename(_: &Jvm, _: &mut WieJvmContext, old_name: ClassInstanceRef<String>, new_name: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.io.FileSystem::rename({old_name:?}, {new_name:?})");

        Ok(())
    }

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

#[cfg(test)]
mod test {
    use alloc::boxed::Box;

    use java_runtime::classes::java::{lang::String, util::Vector};
    use jvm::{Array, ClassInstanceRef, runtime::JavaLangString};
    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::get_protos;

    use super::FileSystem;

    #[test]
    fn test_filesystem_overloads_and_neutral_stubs() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let _: ClassInstanceRef<FileSystem> = jvm.new_class("org/kwis/msp/io/FileSystem", "()V", ()).await?.into();
            let name: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "missing").await?.into();
            let new_name: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "renamed").await?.into();

            let max_length: i32 = jvm.invoke_static("org/kwis/msp/io/FileSystem", "getMaxFilenameLength", "()I", ()).await?;
            assert_eq!(max_length, 0);

            let listed: ClassInstanceRef<Vector> = jvm
                .invoke_static(
                    "org/kwis/msp/io/FileSystem",
                    "list",
                    "(Ljava/lang/String;)Ljava/util/Vector;",
                    (name.clone(),),
                )
                .await?;
            let listed_with_flag: ClassInstanceRef<Vector> = jvm
                .invoke_static(
                    "org/kwis/msp/io/FileSystem",
                    "list",
                    "(Ljava/lang/String;I)Ljava/util/Vector;",
                    (name.clone(), 1),
                )
                .await?;
            assert!(listed.is_null());
            assert!(listed_with_flag.is_null());

            let c_string: ClassInstanceRef<Array<i8>> = jvm
                .invoke_static("org/kwis/msp/io/FileSystem", "toCString", "(Ljava/lang/String;)[B", (name.clone(),))
                .await?;
            assert!(c_string.is_null());

            let is_file: bool = jvm
                .invoke_static("org/kwis/msp/io/FileSystem", "isFile", "(Ljava/lang/String;I)Z", (name.clone(), 1))
                .await?;
            let is_directory: bool = jvm
                .invoke_static("org/kwis/msp/io/FileSystem", "isDirectory", "(Ljava/lang/String;)Z", (name.clone(),))
                .await?;
            let creation_time: i32 = jvm
                .invoke_static("org/kwis/msp/io/FileSystem", "getCreationTime", "(Ljava/lang/String;)I", (name.clone(),))
                .await?;
            let creation_time_with_flag: i32 = jvm
                .invoke_static(
                    "org/kwis/msp/io/FileSystem",
                    "getCreationTime",
                    "(Ljava/lang/String;I)I",
                    (name.clone(), 1),
                )
                .await?;
            assert!(!is_file);
            assert!(!is_directory);
            assert_eq!(creation_time, 0);
            assert_eq!(creation_time_with_flag, 0);

            let _: () = jvm
                .invoke_static("org/kwis/msp/io/FileSystem", "remove", "(Ljava/lang/String;)V", (name.clone(),))
                .await?;
            let _: () = jvm
                .invoke_static("org/kwis/msp/io/FileSystem", "remove", "(Ljava/lang/String;I)V", (name.clone(), 1))
                .await?;
            let _: () = jvm
                .invoke_static("org/kwis/msp/io/FileSystem", "mkdir", "(Ljava/lang/String;)V", (name.clone(),))
                .await?;
            let _: () = jvm
                .invoke_static("org/kwis/msp/io/FileSystem", "rmdir", "(Ljava/lang/String;)V", (name.clone(),))
                .await?;
            let _: () = jvm
                .invoke_static("org/kwis/msp/io/FileSystem", "rmdir", "(Ljava/lang/String;I)V", (name.clone(), 1))
                .await?;
            let _: () = jvm
                .invoke_static(
                    "org/kwis/msp/io/FileSystem",
                    "rename",
                    "(Ljava/lang/String;Ljava/lang/String;)V",
                    (name.clone(), new_name.clone()),
                )
                .await?;
            let _: () = jvm
                .invoke_static(
                    "org/kwis/msp/io/FileSystem",
                    "rename",
                    "(Ljava/lang/String;Ljava/lang/String;I)V",
                    (name, new_name, 1),
                )
                .await?;

            Ok(())
        })
    }
}
