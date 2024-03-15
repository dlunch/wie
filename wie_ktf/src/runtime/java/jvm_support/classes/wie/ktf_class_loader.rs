use alloc::{boxed::Box, vec};

use bytemuck::cast_vec;
use dyn_clone::{clone_trait_object, DynClone};

use java_class_proto::{JavaClassProto, JavaFieldProto, JavaMethodProto};
use java_constants::FieldAccessFlags;
use java_runtime::classes::java::{
    lang::{Class, ClassLoader, String},
    net::URL,
};
use jvm::{runtime::JavaLangString, ClassInstanceRef, Jvm, Result as JvmResult};

use wie_backend::System;
use wie_core_arm::{Allocator, ArmCore};
use wie_util::write_null_terminated_string;

use crate::runtime::java::jvm_support::{class_definition::JavaClassDefinition, context_data::JavaContextData};

pub trait ClassLoaderContextBase: DynClone {
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
                JavaMethodProto::new("<init>", "(Ljava/lang/ClassLoader;)V", Self::init, Default::default()),
                JavaMethodProto::new("findClass", "(Ljava/lang/String;)Ljava/lang/Class;", Self::find_class, Default::default()),
                JavaMethodProto::new(
                    "findResource",
                    "(Ljava/lang/String;)Ljava/net/URL;",
                    Self::find_resource,
                    Default::default(),
                ),
            ],
            fields: vec![JavaFieldProto::new("classes", "[Ljava/lang/Class;", FieldAccessFlags::STATIC)],
        }
    }

    async fn init(jvm: &Jvm, _: &mut ClassLoaderContext, this: ClassInstanceRef<Self>, parent: ClassInstanceRef<ClassLoader>) -> JvmResult<()> {
        tracing::debug!("rustjava.RuntimeClassLoader::<init>({:?}, {:?})", &this, &parent);

        jvm.invoke_special(&this, "java/lang/ClassLoader", "<init>", "(Ljava/lang/ClassLoader;)V", (parent,))
            .await?;

        Ok(())
    }

    async fn find_class(
        jvm: &Jvm,
        context: &mut ClassLoaderContext,
        this: ClassInstanceRef<Self>,
        name: ClassInstanceRef<String>,
    ) -> JvmResult<ClassInstanceRef<Class>> {
        tracing::debug!("rustjava.RuntimeClassLoader::findClass({:?}, {:?})", &this, name);

        // find from client.bin

        let name = JavaLangString::to_rust_string(jvm, &name).await?;

        let core = context.core();
        let fn_get_class = JavaContextData::fn_get_class(core).unwrap();
        if fn_get_class == 0 {
            // we don't have get_class on testcases
            return Ok(None.into());
        }

        let ptr_name = Allocator::alloc(core, 50).unwrap(); // TODO size fix
        write_null_terminated_string(core, ptr_name, &name).unwrap();

        let ptr_raw = core.run_function(fn_get_class, &[ptr_name]).await.unwrap();
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
        tracing::debug!("rustjava.ClassPathClassLoader::findResource({:?}, {:?})", &this, name);

        let name = JavaLangString::to_rust_string(jvm, &name).await?;
        let id = context.system().resource().id(&name);
        if id.is_none() {
            return Ok(None.into());
        }

        let data = context.system().resource().data(id.unwrap()).to_vec();
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
