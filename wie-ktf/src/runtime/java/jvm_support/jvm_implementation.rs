use alloc::{
    boxed::Box,
    collections::{BTreeMap, BTreeSet},
    format,
    sync::Arc,
    vec::Vec,
};
use core::{
    mem::size_of,
    ops::{Deref, DerefMut},
    pin::Pin,
};

use jvm::{ClassDefinition, ClassInstance, ExternalRoots, Jvm, Result as JvmResult};
use jvm_class_proto::JavaClassProto;
use spin::Mutex;
use wipi_types::ktf::java::JavaClassInstance as RawJavaClassInstance;

use wie_core_arm::{ArmCore, EmulatedFunctionParam, ResultWriter, SvcId};
use wie_jvm_support::JvmImplementation;
use wie_util::{WieError, read_generic};

use crate::runtime::{
    SVC_CATEGORY_INIT,
    java::{JavaSvcFunctions, register_java_svc_handler},
    svc_ids::InitSvcId,
};

use super::{JavaArrayClassDefinition, JavaClassDefinition, KtfJvmSupportContext, SUPPORT_CONTEXT_BASE};

#[derive(Clone)]
pub struct KtfJvmImplementation {
    core: ArmCore,
    java_functions: JavaSvcFunctions,
}

impl KtfJvmImplementation {
    pub fn new(core: &mut ArmCore) -> Self {
        let java_functions = Arc::new(Mutex::new(BTreeMap::new()));
        register_java_svc_handler(core, &java_functions).unwrap();

        Self {
            core: core.clone(),
            java_functions,
        }
    }

    pub fn java_functions(&self) -> JavaSvcFunctions {
        self.java_functions.clone()
    }
}

impl JvmImplementation for KtfJvmImplementation {
    fn external_roots(&self) -> Option<Box<dyn ExternalRoots>> {
        Some(Box::new(self.clone()))
    }

    fn define_class_rust<'a, C, Context>(
        &'a self,
        jvm: &'a Jvm,
        proto: JavaClassProto<C>,
        context: Context,
    ) -> Pin<Box<dyn Future<Output = JvmResult<Box<dyn ClassDefinition>>> + Send + 'a>>
    where
        C: ?Sized + 'static + Send,
        Context: Sync + Send + DerefMut + Deref<Target = C> + Clone + 'static,
    {
        Box::pin(async move {
            Ok(Box::new(
                JavaClassDefinition::new(&mut self.core.clone(), jvm, proto, context, self.java_functions.clone())
                    .await
                    .unwrap(),
            ) as _)
        })
    }

    async fn define_array_class(&self, jvm: &Jvm, element_type_name: &str) -> JvmResult<Box<dyn ClassDefinition>> {
        let class_name = format!("[{element_type_name}");
        let class = JavaArrayClassDefinition::new(&mut self.core.clone(), jvm, &class_name).await.unwrap();

        Ok(Box::new(class) as Box<_>)
    }
}

impl ExternalRoots for KtfJvmImplementation {
    fn roots(&self, objects: &[&dyn ClassInstance]) -> anyhow::Result<Vec<usize>> {
        let mut core = self.core.clone();
        let context: KtfJvmSupportContext = read_generic(&core, SUPPORT_CONTEXT_BASE)?;
        let mut roots = Vec::new();
        if context.ptr_visit_gc_root == 0 {
            return Ok(roots);
        }

        tracing::debug!("ktf.visit_gc_roots()");
        core.run_function_sync::<()>(context.ptr_visit_gc_root, &[], |core, category| {
            let id = SvcId::get(core, 0);
            if category != SVC_CATEGORY_INIT || id.0 != InitSvcId::VisitGcRoot as u32 {
                return Err(WieError::FatalError(format!(
                    "Unexpected SVC during root visitation: {category}:{:#x}",
                    id.0
                )));
            }
            let root = core.read_param(0)?;
            tracing::debug!("ktf.visit_gc_root({root:#x})");
            if root != 0 {
                roots.push(root as usize);
            }
            let (_, lr) = core.read_pc_lr()?;
            ().write(core, lr)
        })?;
        let words = core.stack_words()?.into_iter().collect::<BTreeSet<_>>();
        for object in objects {
            let identity = object.identity() as u32;
            let raw: RawJavaClassInstance = read_generic(&core, identity)?;
            let field_size = if let Some(array) = object.as_array_instance() {
                JavaArrayClassDefinition::from_raw(raw.ptr_class, &core).element_size()? * array.length() + 4
            } else {
                JavaClassDefinition::from_raw(raw.ptr_class, &core).field_size()?
            };
            if words
                .range(identity..identity + size_of::<RawJavaClassInstance>() as u32)
                .next()
                .is_some()
                || words.range(raw.ptr_fields..raw.ptr_fields + field_size as u32 + 4).next().is_some()
            {
                roots.push(object.identity());
            }
        }
        Ok(roots)
    }
}
