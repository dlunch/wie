use alloc::{boxed::Box, vec};

use bytemuck::{cast_slice, cast_vec};
use dyn_clone::{clone_trait_object, DynClone};

use java_class_proto::{JavaClassProto, JavaFieldProto, JavaMethodProto};
use java_runtime::classes::java::{
    lang::{Class, ClassLoader, String},
    net::URL,
};
use jvm::{runtime::JavaLangString, ClassInstanceRef, Jvm, Result as JvmResult};

use wie_backend::System;
use wie_core_arm::{Allocator, ArmCore};
use wie_util::write_null_terminated_string;

use crate::runtime::{init::load_native, java::jvm_support::class_definition::JavaClassDefinition};

pub trait ClassLoaderContextBase: Sync + Send + DynClone {
    fn core(&mut self) -> &mut ArmCore;
    fn system(&self) -> &System;
}

clone_trait_object!(ClassLoaderContextBase);

type ClassLoaderProto = JavaClassProto<dyn ClassLoaderContextBase>;
type ClassLoaderContext = dyn ClassLoaderContextBase;

// class wie.KtfClassLoader
pub struct KtfClassLoader {}

impl KtfClassLoader {
    pub fn as_proto() -> ClassLoaderProto {
        ClassLoaderProto {
            parent_class: Some("java/lang/ClassLoader"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljava/lang/ClassLoader;Ljava/lang/String;II)V", Self::init, Default::default()),
                JavaMethodProto::new("findClass", "(Ljava/lang/String;)Ljava/lang/Class;", Self::find_class, Default::default()),
                JavaMethodProto::new(
                    "findResource",
                    "(Ljava/lang/String;)Ljava/net/URL;",
                    Self::find_resource,
                    Default::default(),
                ),
            ],
            fields: vec![
                JavaFieldProto::new("ptrJvmContext", "I", Default::default()),
                JavaFieldProto::new("fnGetClass", "I", Default::default()),
            ],
        }
    }

    async fn init(
        jvm: &Jvm,
        context: &mut ClassLoaderContext,
        mut this: ClassInstanceRef<Self>,
        parent: ClassInstanceRef<ClassLoader>,
        client_bin: ClassInstanceRef<String>,
        ptr_jvm_context: i32,
        ptr_jvm_exception_context: i32,
    ) -> JvmResult<()> {
        tracing::debug!(
            "wie.KtfClassLoader::<init>({:?}, {:?}, {:?}, {:?}, {:?})",
            &this,
            parent,
            client_bin,
            ptr_jvm_context,
            ptr_jvm_exception_context
        );

        jvm.invoke_special(&this, "java/lang/ClassLoader", "<init>", "(Ljava/lang/ClassLoader;)V", (parent,))
            .await?;

        // load client.bin
        let data_stream = jvm
            .invoke_virtual(
                &this,
                "getResourceAsStream",
                "(Ljava/lang/String;)Ljava/io/InputStream;",
                (client_bin.clone(),),
            )
            .await?;
        let length: i32 = jvm.invoke_virtual(&data_stream, "available", "()I", ()).await?;
        let buf = jvm.instantiate_array("B", length as _).await?;
        let _: i32 = jvm.invoke_virtual(&data_stream, "read", "([B)I", (buf.clone(),)).await?;

        let filename = JavaLangString::to_rust_string(jvm, &client_bin).await?;
        let data = jvm.load_byte_array(&buf, 0, length as _).await?;

        let fn_get_class = load_native(
            context.core(),
            &filename,
            cast_slice(&data),
            ptr_jvm_context as _,
            ptr_jvm_exception_context as _,
        )
        .await
        .unwrap() as i32;

        jvm.put_field(&mut this, "ptrJvmContext", "I", ptr_jvm_context).await?;
        jvm.put_field(&mut this, "fnGetClass", "I", fn_get_class).await?;

        Ok(())
    }

    async fn find_class(
        jvm: &Jvm,
        context: &mut ClassLoaderContext,
        this: ClassInstanceRef<Self>,
        name: ClassInstanceRef<String>,
    ) -> JvmResult<ClassInstanceRef<Class>> {
        tracing::debug!("wie.KtfClassLoader::findClass({:?}, {:?})", &this, name);

        let fn_get_class: i32 = jvm.get_field(&this, "fnGetClass", "I").await?;

        // find from client.bin
        let name = JavaLangString::to_rust_string(jvm, &name).await?;

        let core = context.core();
        let ptr_name = Allocator::alloc(core, 50).unwrap(); // TODO size fix
        write_null_terminated_string(core, ptr_name, &name).unwrap();

        let ptr_raw = core.run_function(fn_get_class as _, &[ptr_name]).await.unwrap();
        Allocator::free(core, ptr_name).unwrap();

        if ptr_raw != 0 {
            let class = JavaClassDefinition::from_raw(ptr_raw, core);
            jvm.register_class(Box::new(class), Some(this.into())).await?;

            Ok(jvm.resolve_class(&name).await?.java_class(jvm).await?.into())
        } else {
            Ok(None.into())
        }
    }

    // TODO use classpathloader's jar loading
    async fn find_resource(
        jvm: &Jvm,
        context: &mut ClassLoaderContext,
        this: ClassInstanceRef<Self>,
        name: ClassInstanceRef<String>,
    ) -> JvmResult<ClassInstanceRef<URL>> {
        tracing::debug!("wie.KtfClassLoader::findResource({:?}, {:?})", &this, name);

        let name = JavaLangString::to_rust_string(jvm, &name).await?;

        let data = {
            let filesystem = context.system().filesystem();
            let data = filesystem.read(&name).map(|x| x.to_vec()); // TODO exception
            if data.is_none() {
                return Ok(None.into());
            }
            data.unwrap()
        };

        let mut data_array = jvm.instantiate_array("B", data.len()).await?;
        jvm.store_byte_array(&mut data_array, 0, cast_vec(data)).await?;

        let protocol = JavaLangString::from_rust_string(jvm, "bytes").await?;
        let host = JavaLangString::from_rust_string(jvm, "").await?;
        let port = 0;
        let file = JavaLangString::from_rust_string(jvm, &name).await?;
        let handler = jvm.new_class("rustjava/ByteArrayURLHandler", "([B)V", (data_array,)).await?;

        let url = jvm
            .new_class(
                "java/net/URL",
                "(Ljava/lang/String;Ljava/lang/String;ILjava/lang/String;Ljava/net/URLStreamHandler;)V",
                (protocol, host, port, file, handler),
            )
            .await?;

        Ok(url.into())
    }
}
