use alloc::{boxed::Box, vec};

use java_class_proto::{JavaClassProto, JavaFieldProto, JavaMethodProto};
use java_runtime::classes::java::lang::{Class, ClassLoader, String};
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};

use wie_core_arm::{Allocator, ArmCore};
use wie_util::write_null_terminated_string_bytes;

use crate::runtime::java::jvm_support::class_definition::JavaClassDefinition;

#[derive(Clone)]
pub struct ClassLoaderContext {
    pub core: ArmCore,
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
                JavaMethodProto::new("<init>", "(Ljava/lang/ClassLoader;I)V", Self::init, Default::default()),
                JavaMethodProto::new("findClass", "(Ljava/lang/String;)Ljava/lang/Class;", Self::find_class, Default::default()),
            ],
            fields: vec![
                JavaFieldProto::new("fnGetClass", "I", Default::default()),
                JavaFieldProto::new("nativeStrings", "Ljava/util/Vector;", Default::default()),
            ],
        }
    }

    async fn init(
        jvm: &Jvm,
        _context: &mut ClassLoaderContext,
        mut this: ClassInstanceRef<Self>,
        parent: ClassInstanceRef<ClassLoader>,
        fn_get_class: i32,
    ) -> JvmResult<()> {
        tracing::debug!("net.wie.KtfClassLoader::<init>({this:?}, {parent:?}, {fn_get_class:?})");

        let _: () = jvm
            .invoke_special(&this, "java/lang/ClassLoader", "<init>", "(Ljava/lang/ClassLoader;)V", (parent,))
            .await?;

        jvm.put_field(&mut this, "fnGetClass", "I", fn_get_class).await?;

        let native_strings = jvm.new_class("java/util/Vector", "()V", ()).await?;
        jvm.put_field(&mut this, "nativeStrings", "Ljava/util/Vector;", native_strings).await?;

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
