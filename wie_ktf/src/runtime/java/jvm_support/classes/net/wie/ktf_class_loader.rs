use alloc::{boxed::Box, vec};

use bytemuck::cast_slice;
use java_class_proto::{JavaClassProto, JavaFieldProto, JavaMethodProto};
use java_constants::FieldAccessFlags;
use java_runtime::classes::java::lang::{Class, ClassLoader, String};
use jvm::{
    ClassInstanceRef, Jvm, Result as JvmResult,
    runtime::{JavaIoInputStream, JavaLangString},
};

use wie_backend::System;
use wie_core_arm::{Allocator, ArmCore};
use wie_util::write_null_terminated_string_bytes;

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
            parent_class: Some("java/lang/ClassLoader"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljava/lang/ClassLoader;Ljava/lang/String;II)V", Self::init, Default::default()),
                JavaMethodProto::new("findClass", "(Ljava/lang/String;)Ljava/lang/Class;", Self::find_class, Default::default()),
            ],
            fields: vec![
                JavaFieldProto::new("fnGetClass", "I", Default::default()),
                JavaFieldProto::new("nativeStrings", "Ljava/util/Vector;", Default::default()),
                JavaFieldProto::new("instance", "Lnet/wie/KtfClassLoader;", FieldAccessFlags::STATIC),
            ],
        }
    }

    async fn init(
        jvm: &Jvm,
        context: &mut ClassLoaderContext,
        mut this: ClassInstanceRef<Self>,
        parent: ClassInstanceRef<ClassLoader>,
        binary_name: ClassInstanceRef<String>,
        ptr_jvm_context: i32,
        ptr_jvm_exception_context: i32,
    ) -> JvmResult<()> {
        tracing::debug!("net.wie.KtfClassLoader::<init>({this:?}, {parent:?}, {binary_name:?})");

        let _: () = jvm
            .invoke_special(&this, "java/lang/ClassLoader", "<init>", "(Ljava/lang/ClassLoader;)V", (parent,))
            .await?;

        let native_strings = jvm.new_class("java/util/Vector", "()V", ()).await?;
        jvm.put_field(&mut this, "nativeStrings", "Ljava/util/Vector;", native_strings).await?;

        jvm.put_static_field("net/wie/KtfClassLoader", "instance", "Lnet/wie/KtfClassLoader;", this.clone())
            .await?;

        // load client.bin
        let name_rust = JavaLangString::to_rust_string(jvm, &binary_name).await.unwrap();
        let data_stream = jvm
            .invoke_virtual(&this, "getResourceAsStream", "(Ljava/lang/String;)Ljava/io/InputStream;", (binary_name,))
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
            ptr_jvm_exception_context as _,
        )
        .await
        .unwrap();

        jvm.put_field(&mut this, "fnGetClass", "I", native_functions.fn_get_class as i32).await?;

        Ok(())
    }

    async fn find_class(
        jvm: &Jvm,
        context: &mut ClassLoaderContext,
        this: ClassInstanceRef<Self>,
        name: ClassInstanceRef<String>,
    ) -> JvmResult<ClassInstanceRef<Class>> {
        tracing::debug!("net.wie.KtfClassLoader::findClass({:?}, {:?})", &this, name);

        let fn_get_class: i32 = jvm.get_field(&this, "fnGetClass", "I").await?;

        if fn_get_class == 0 {
            return Ok(None.into());
        }

        // find from client.bin
        let name = JavaLangString::to_rust_string(jvm, &name).await?;

        let ptr_name = Allocator::alloc(&mut context.core, 50).unwrap(); // TODO size fix
        write_null_terminated_string_bytes(&mut context.core, ptr_name, name.as_bytes()).unwrap();

        let ptr_raw = context.core.run_function(fn_get_class as _, &[ptr_name]).await.unwrap();
        Allocator::free(&mut context.core, ptr_name, 50).unwrap();

        if ptr_raw != 0 {
            let class = JavaClassDefinition::from_raw(ptr_raw, &context.core);
            jvm.register_class(Box::new(class), Some(this.into())).await?;

            Ok(jvm.resolve_class(&name).await?.java_class().into())
        } else {
            Ok(None.into())
        }
    }
}
