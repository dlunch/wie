use alloc::{
    boxed::Box,
    string::{String as RustString, ToString},
    vec,
};
use core::mem::size_of;

use java_class_proto::{JavaClassProto, JavaFieldProto, JavaMethodProto};
use java_constants::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use java_runtime::classes::java::{
    lang::{Class, ClassLoader, String},
    util::Vector,
};
use jvm::{ClassInstance, ClassInstanceRef, JavaError, Jvm, Result as JvmResult, runtime::JavaLangString};
use wipi_types::lgt::java::{LgtJavaClass as RawJavaClass, LgtJavaClassDescriptor as RawJavaClassDescriptor};

use wie_core_arm::ArmCore;
use wie_util::{Result, read_generic, read_null_terminated_string_bytes};

use crate::runtime::java::jvm_support::LgtJvmSupport;

type ClassLoaderProto = JavaClassProto<ArmCore>;

// class net.wie.LgtClassLoader
pub struct LgtClassLoader;

impl LgtClassLoader {
    pub fn as_proto() -> ClassLoaderProto {
        ClassLoaderProto {
            name: "net/wie/LgtClassLoader",
            parent_class: Some("java/lang/ClassLoader"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljava/lang/ClassLoader;I)V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "findClass",
                    "(Ljava/lang/String;)Ljava/lang/Class;",
                    Self::find_class,
                    MethodAccessFlags::PROTECTED,
                ),
            ],
            fields: vec![
                JavaFieldProto::new("generatedClasses", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("nativeStrings", "Ljava/util/Vector;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new(
                    "instance",
                    "Lnet/wie/LgtClassLoader;",
                    FieldAccessFlags::PRIVATE | FieldAccessFlags::STATIC,
                ),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(
        jvm: &Jvm,
        _: &mut ArmCore,
        mut this: ClassInstanceRef<Self>,
        parent: ClassInstanceRef<ClassLoader>,
        generated_classes: i32,
    ) -> JvmResult<()> {
        let _: () = jvm
            .invoke_special(&this, "java/lang/ClassLoader", "<init>", "(Ljava/lang/ClassLoader;)V", (parent,))
            .await?;
        jvm.put_field(&mut this, "generatedClasses", "I", generated_classes).await?;
        let native_strings: ClassInstanceRef<Vector> = jvm.new_class("java/util/Vector", "()V", ()).await?.into();
        jvm.put_field(&mut this, "nativeStrings", "Ljava/util/Vector;", native_strings).await?;
        jvm.put_static_field("net/wie/LgtClassLoader", "instance", "Lnet/wie/LgtClassLoader;", this)
            .await
    }

    fn find_raw_class(core: &ArmCore, generated_classes: u32, name: &str) -> Result<Option<u32>> {
        let last_bucket: u32 = read_generic(core, generated_classes)?;
        for bucket in 0..=last_bucket {
            let mut ptr_class: u32 = read_generic(core, generated_classes + size_of::<u32>() as u32 + bucket * size_of::<u32>() as u32)?;
            while ptr_class != 0 {
                let raw: RawJavaClass = read_generic(core, ptr_class)?;
                let descriptor: RawJavaClassDescriptor = read_generic(core, raw.ptr_descriptor)?;
                let class_name = RustString::from_utf8(read_null_terminated_string_bytes(core, descriptor.ptr_name)?)
                    .map_err(|error| wie_util::WieError::FatalError(alloc::format!("Invalid LGT class name: {error}")))?;
                if class_name == name {
                    return Ok(Some(ptr_class));
                }
                ptr_class = descriptor.ptr_next_class;
            }
        }
        Ok(None)
    }

    async fn find_class(
        jvm: &Jvm,
        core: &mut ArmCore,
        this: ClassInstanceRef<Self>,
        name: ClassInstanceRef<String>,
    ) -> JvmResult<ClassInstanceRef<Class>> {
        let name = JavaLangString::to_rust_string(jvm, &name).await?.replace('.', "/");
        let generated_classes: i32 = jvm.get_field(&this, "generatedClasses", "I").await?;
        let ptr_class = match Self::find_raw_class(core, generated_classes as u32, &name) {
            Ok(Some(ptr_class)) => ptr_class,
            Ok(None) => return Ok(None.into()),
            Err(error) => return Err(jvm.exception("net/wie/WieError", &error.to_string()).await),
        };

        let loader: Box<dyn ClassInstance> = this.clone().into();
        match LgtJvmSupport::register_generated_class(core, jvm, ptr_class, generated_classes as u32, loader).await {
            Ok(class) => Ok(class.into()),
            Err(wie_util::WieError::JavaException(ptr_exception)) => {
                Err(JavaError::JavaException(LgtJvmSupport::class_instance_from_raw(core, ptr_exception)))
            }
            Err(error) => Err(jvm.exception("net/wie/WieError", &error.to_string()).await),
        }
    }
}

#[cfg(test)]
mod tests {
    use core::mem::{offset_of, size_of};

    use wipi_types::lgt::java::{LgtJavaClass as RawJavaClass, LgtJavaClassDescriptor as RawJavaClassDescriptor};

    use wie_core_arm::{Allocator, ArmCore};
    use wie_util::{ByteWrite, Result, write_generic, write_null_terminated_string_bytes};

    use super::LgtClassLoader;

    #[test]
    fn generated_class_lookup_includes_last_bucket() -> Result<()> {
        let mut core = ArmCore::new(false, None)?;
        Allocator::init(&mut core)?;

        let ptr_name = Allocator::alloc(&mut core, 5)?;
        write_null_terminated_string_bytes(&mut core, ptr_name, b"Last")?;
        let ptr_descriptor = Allocator::alloc(&mut core, size_of::<RawJavaClassDescriptor>() as u32)?;
        core.write_bytes(ptr_descriptor, &[0; size_of::<RawJavaClassDescriptor>()])?;
        write_generic(&mut core, ptr_descriptor + offset_of!(RawJavaClassDescriptor, ptr_name) as u32, ptr_name)?;
        let ptr_class = Allocator::alloc(&mut core, size_of::<RawJavaClass>() as u32)?;
        core.write_bytes(ptr_class, &[0; size_of::<RawJavaClass>()])?;
        write_generic(&mut core, ptr_class + offset_of!(RawJavaClass, ptr_descriptor) as u32, ptr_descriptor)?;

        let generated_classes = Allocator::alloc(&mut core, 4 + 14 * 4)?;
        core.write_bytes(generated_classes, &[0; 4 + 14 * 4])?;
        write_generic(&mut core, generated_classes, 13u32)?;
        write_generic(&mut core, generated_classes + 4 + 13 * 4, ptr_class)?;

        assert_eq!(LgtClassLoader::find_raw_class(&core, generated_classes, "Last")?, Some(ptr_class));
        Ok(())
    }
}
