use alloc::{boxed::Box, vec};

use dyn_clone::{clone_trait_object, DynClone};

use java_class_proto::{JavaClassProto, JavaFieldProto, JavaMethodProto, JavaResult};
use java_constants::FieldAccessFlags;
use java_runtime::classes::java::lang::{Class, ClassLoader, String};
use jvm::{ClassInstanceRef, Jvm};

use wie_common::util::write_null_terminated_string;
use wie_core_arm::{Allocator, ArmCore};

use crate::runtime::java::jvm::{class::JavaClass, context_data::JavaContextData};

pub trait ClassLoaderContextBase: DynClone {
    fn core(&mut self) -> &mut ArmCore;
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
            ],
            fields: vec![JavaFieldProto::new("classes", "[Ljava/lang/Class;", FieldAccessFlags::STATIC)],
        }
    }

    async fn init(jvm: &Jvm, _: &mut ClassLoaderContext, this: ClassInstanceRef<Self>, parent: ClassInstanceRef<ClassLoader>) -> JavaResult<()> {
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
    ) -> JavaResult<ClassInstanceRef<Class>> {
        tracing::debug!("rustjava.RuntimeClassLoader::findClass({:?}, {:?})", &this, name);

        // find from client.bin

        let name = String::to_rust_string(jvm, &name)?;

        let core = context.core();
        let fn_get_class = JavaContextData::fn_get_class(core)?;
        if fn_get_class == 0 {
            // we don't have get_class on testcases
            return Ok(None.into());
        }

        let ptr_name = Allocator::alloc(core, 50)?; // TODO size fix
        write_null_terminated_string(core, ptr_name, &name)?;

        let ptr_raw = core.run_function(fn_get_class, &[ptr_name]).await?;
        Allocator::free(core, ptr_name)?;

        if ptr_raw != 0 {
            let class = JavaClass::from_raw(ptr_raw, core);

            return Class::from_rust_class(jvm, Box::new(class)).await;
        }

        Ok(None.into())
    }
}
