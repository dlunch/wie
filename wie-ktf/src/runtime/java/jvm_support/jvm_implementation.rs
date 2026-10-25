use alloc::{boxed::Box, collections::BTreeMap, format, sync::Arc, vec::Vec};
use core::{
    ops::{Deref, DerefMut},
    pin::Pin,
};

use jvm::{ClassDefinition, ClassInstance, ExternalRoots, Jvm, Result as JvmResult};
use jvm_class_proto::JavaClassProto;
use spin::Mutex;

use wie_core_arm::{ArmCore, EmulatedFunctionParam, ResultWriter, SvcId};
use wie_jvm_support::JvmImplementation;
use wie_util::{WieError, read_generic};
use wipi_types::ktf::WipiExe;

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
    fn roots(&self, _objects: &[&dyn ClassInstance]) -> anyhow::Result<Vec<usize>> {
        let mut core = self.core.clone();
        let context: KtfJvmSupportContext = read_generic(&core, SUPPORT_CONTEXT_BASE)?;
        let mut roots = Vec::new();
        if context.ptr_wipi_exe == 0 {
            return Ok(roots);
        }
        let executable: WipiExe = read_generic(&core, context.ptr_wipi_exe)?;
        if executable.fn_visit_gc_roots == 0 {
            return Ok(roots);
        }

        tracing::debug!("ktf.visit_gc_roots()");
        core.run_function_sync::<()>(executable.fn_visit_gc_roots, &[], |core, category| {
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
        Ok(roots)
    }
}
