use alloc::{boxed::Box, format, vec};

use bytemuck::cast_slice;
use jvm::{
    ClassInstanceRef, Jvm, Result as JvmResult,
    runtime::{JavaIoInputStream, JavaLangString},
};
use jvm_class_proto::{JavaClassProto, JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::{Class, ClassLoader, String};
use wipi_types::ktf::ExeInterfaceFunctions;

use wie_backend::System;
use wie_core_arm::{Allocator, ArmCore};
use wie_util::{read_generic, write_null_terminated_string_bytes};

use crate::runtime::{init::load_native, java::jvm_support::class_definition::JavaClassDefinition};

#[derive(Clone)]
pub struct ClassLoaderContext {
    pub core: ArmCore,
    pub system: System,
}

type ClassLoaderProto = JavaClassProto<ClassLoaderContext>;

// class net.wie.KtfClassLoader
pub struct KtfClassLoader;

impl KtfClassLoader {
    pub fn as_proto() -> ClassLoaderProto {
        ClassLoaderProto {
            name: "net/wie/KtfClassLoader",
            parent_class: Some("java/net/URLClassLoader"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new(
                    "<init>",
                    "(Ljava/lang/String;Ljava/lang/String;II)V",
                    Self::init,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "findClass",
                    "(Ljava/lang/String;)Ljava/lang/Class;",
                    Self::find_class,
                    MethodAccessFlags::PROTECTED,
                ),
            ],
            fields: vec![
                JavaFieldProto::new("nativeFunctions", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new(
                    "instance",
                    "Lnet/wie/KtfClassLoader;",
                    FieldAccessFlags::PRIVATE | FieldAccessFlags::STATIC,
                ),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(
        jvm: &Jvm,
        context: &mut ClassLoaderContext,
        mut this: ClassInstanceRef<Self>,
        jar_name: ClassInstanceRef<String>,
        binary_name: ClassInstanceRef<String>,
        ptr_jvm_context: i32,
        ptr_current_jvm_thread_context: i32,
    ) -> JvmResult<()> {
        tracing::debug!(
            "net.wie.KtfClassLoader::<init>({this:?}, {jar_name:?}, {binary_name:?}, {ptr_jvm_context:#x}, {ptr_current_jvm_thread_context:#x})"
        );

        let parent: ClassInstanceRef<ClassLoader> = jvm
            .invoke_static("java/lang/ClassLoader", "getSystemClassLoader", "()Ljava/lang/ClassLoader;", ())
            .await?;

        let jar_name = JavaLangString::to_rust_string(jvm, &jar_name).await?;
        let url = JavaLangString::from_rust_string(jvm, &format!("file:{jar_name}")).await?;
        let url = jvm.new_class("java/net/URL", "(Ljava/lang/String;)V", (url,)).await?;
        let mut urls = jvm.instantiate_array("Ljava/net/URL;", 1).await?;
        jvm.store_array(&mut urls, 0, [url]).await?;

        let _: () = jvm
            .invoke_special(
                &this,
                "java/net/URLClassLoader",
                "<init>",
                "([Ljava/net/URL;Ljava/lang/ClassLoader;)V",
                (urls, parent),
            )
            .await?;

        // Native resource APIs use the system loader during initialization.
        jvm.put_static_field("java/lang/ClassLoader", "systemClassLoader", "Ljava/lang/ClassLoader;", this.clone())
            .await?;
        jvm.put_static_field("net/wie/KtfClassLoader", "instance", "Lnet/wie/KtfClassLoader;", this.clone())
            .await?;

        // load client.bin
        let name_rust = JavaLangString::to_rust_string(jvm, &binary_name).await.unwrap();
        let data_stream = jvm
            .invoke_virtual(
                &this,
                "net/wie/KtfClassLoader",
                "getResourceAsStream",
                "(Ljava/lang/String;)Ljava/io/InputStream;",
                (binary_name,),
            )
            .await
            .unwrap();
        let data = JavaIoInputStream::read_until_end(jvm, &data_stream).await.unwrap();

        // load binary
        let native_functions = load_native(
            &mut context.core,
            &mut context.system,
            jvm,
            &name_rust,
            cast_slice(&data),
            ptr_jvm_context as _,
            ptr_current_jvm_thread_context as _,
        )
        .await
        .unwrap();

        jvm.put_field(&mut this, "net/wie/KtfClassLoader", "nativeFunctions", "I", native_functions as i32)
            .await?;

        Ok(())
    }

    async fn find_class(
        jvm: &Jvm,
        context: &mut ClassLoaderContext,
        this: ClassInstanceRef<Self>,
        name: ClassInstanceRef<String>,
    ) -> JvmResult<ClassInstanceRef<Class>> {
        tracing::debug!("net.wie.KtfClassLoader::findClass({this:?}, {name:?})");

        let ptr_native_functions: i32 = jvm.get_field(&this, "net/wie/KtfClassLoader", "nativeFunctions", "I").await?;

        if ptr_native_functions == 0 {
            return Ok(None.into());
        }
        let native_functions: ExeInterfaceFunctions = read_generic(&context.core, ptr_native_functions as u32).unwrap();
        if native_functions.fn_get_class == 0 {
            return Ok(None.into());
        }

        // find from client.bin
        let name = JavaLangString::to_rust_string(jvm, &name).await?.replace('.', "/");

        let ptr_name_size = (name.len() + 1) as u32;
        let ptr_name = Allocator::alloc(&mut context.core, ptr_name_size).unwrap();
        write_null_terminated_string_bytes(&mut context.core, ptr_name, name.as_bytes()).unwrap();

        let ptr_raw = context.core.run_function(native_functions.fn_get_class, &[ptr_name]).await.unwrap();
        Allocator::free(&mut context.core, ptr_name, ptr_name_size).unwrap();

        if ptr_raw != 0 {
            let class = JavaClassDefinition::from_raw(ptr_raw, &context.core);
            jvm.register_class(Box::new(class), Some(this.into())).await?;

            Ok(jvm.resolve_class(&name).await?.java_class().into())
        } else {
            Ok(None.into())
        }
    }
}
