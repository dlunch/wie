use jvm::Jvm;

use wie_core_arm::{ArmCore, EmulatedFunction, JumpTo, ResultWriter, SvcId};
use wie_util::{Result, WieError};

use crate::runtime::{SVC_CATEGORY_JAVA_SYSTEM, svc_ids::JavaSystemSvcId};

use super::{
    JavaExceptionState,
    interface::{
        java_create_runtime_context, java_destroy_runtime_context, java_exception_matches_class, java_get_array_type, java_get_string_array_class,
        java_get_string_class, java_initialize_class, java_instantiate, java_instantiate_array, java_instantiate_multi_array,
        java_link_imported_classes, java_link_public_class, java_method_prologue, java_pending_exception, java_pop_exception_frame,
        java_push_exception_frame, java_raise_arithmetic_exception, java_raise_array_index_exception, java_raise_null_pointer_exception,
        java_register_class, java_resolve_class, java_rethrow_exception, java_safepoint, java_set_jar_path, java_start_application,
        java_store_reference_array, java_store_reference_array_unchecked, java_string_literal, java_unk0, java_unk1, java_unk2, java_unk3,
    },
};

pub fn register_java_system_svc_handler(core: &mut ArmCore, jvm: &Jvm, exception_state: JavaExceptionState, ptr_jar_path: u32) -> Result<()> {
    core.register_svc_handler(
        SVC_CATEGORY_JAVA_SYSTEM,
        handle_java_system_svc,
        &(jvm.clone(), exception_state, ptr_jar_path),
    )
}

async fn handle_java_system_svc(
    core: &mut ArmCore,
    (jvm, exception_state, ptr_jar_path): &mut (Jvm, JavaExceptionState, u32),
    id: SvcId,
) -> Result<JumpTo> {
    let (_, lr) = core.read_pc_lr()?;
    let result: Result<()> = async {
        match JavaSystemSvcId::try_from(id)? {
            JavaSystemSvcId::InterfaceUnk0 => EmulatedFunction::call(&java_unk0, core, &mut ()).await?.write(core, lr),
            JavaSystemSvcId::DestroyRuntimeContext => EmulatedFunction::call(&java_destroy_runtime_context, core, &mut ())
                .await?
                .write(core, lr),
            JavaSystemSvcId::CreateRuntimeContext => EmulatedFunction::call(&java_create_runtime_context, core, jvm).await?.write(core, lr),
            JavaSystemSvcId::LinkImportedClasses => EmulatedFunction::call(&java_link_imported_classes, core, jvm).await?.write(core, lr),
            JavaSystemSvcId::SetJarPath => EmulatedFunction::call(&java_set_jar_path, core, ptr_jar_path).await?.write(core, lr),
            JavaSystemSvcId::StartApplication => EmulatedFunction::call(&java_start_application, core, jvm).await?.write(core, lr),
            JavaSystemSvcId::RegisterClass => EmulatedFunction::call(&java_register_class, core, jvm).await?.write(core, lr),
            JavaSystemSvcId::ResolveClass => EmulatedFunction::call(&java_resolve_class, core, jvm).await?.write(core, lr),
            JavaSystemSvcId::InitializeClass => EmulatedFunction::call(&java_initialize_class, core, jvm).await?.write(core, lr),
            JavaSystemSvcId::GetArrayType => EmulatedFunction::call(&java_get_array_type, core, jvm).await?.write(core, lr),
            JavaSystemSvcId::Instantiate => EmulatedFunction::call(&java_instantiate, core, jvm).await?.write(core, lr),
            JavaSystemSvcId::InstantiateArray => EmulatedFunction::call(&java_instantiate_array, core, jvm).await?.write(core, lr),
            JavaSystemSvcId::InstantiateMultiArray => EmulatedFunction::call(&java_instantiate_multi_array, core, jvm).await?.write(core, lr),
            JavaSystemSvcId::MethodPrologue => EmulatedFunction::call(&java_method_prologue, core, &mut ()).await?.write(core, lr),
            JavaSystemSvcId::Safepoint => EmulatedFunction::call(&java_safepoint, core, &mut ()).await?.write(core, lr),
            JavaSystemSvcId::StringLiteral => EmulatedFunction::call(&java_string_literal, core, jvm).await?.write(core, lr),
            JavaSystemSvcId::PushExceptionFrame => EmulatedFunction::call(&java_push_exception_frame, core, exception_state)
                .await?
                .write(core, lr),
            JavaSystemSvcId::PopExceptionFrame => EmulatedFunction::call(&java_pop_exception_frame, core, exception_state)
                .await?
                .write(core, lr),
            JavaSystemSvcId::StoreReferenceArray => EmulatedFunction::call(&java_store_reference_array, core, jvm).await?.write(core, lr),
            JavaSystemSvcId::GetStringClass => EmulatedFunction::call(&java_get_string_class, core, jvm).await?.write(core, lr),
            JavaSystemSvcId::GetStringArrayClass => EmulatedFunction::call(&java_get_string_array_class, core, jvm).await?.write(core, lr),
            JavaSystemSvcId::PendingException => EmulatedFunction::call(&java_pending_exception, core, exception_state)
                .await?
                .write(core, lr),
            JavaSystemSvcId::StoreReferenceArrayUnchecked => EmulatedFunction::call(&java_store_reference_array_unchecked, core, &mut ())
                .await?
                .write(core, lr),
            JavaSystemSvcId::LinkPublicClass => EmulatedFunction::call(&java_link_public_class, core, jvm).await?.write(core, lr),
            JavaSystemSvcId::ExceptionMatchesClass => {
                java_exception_matches_class(core, jvm, *exception_state, core.read_param(0)?, core.read_param(1)?, core.read_param(2)?)
                    .await?
                    .write(core, lr)
            }
            JavaSystemSvcId::RethrowException => EmulatedFunction::call(&java_rethrow_exception, core, exception_state)
                .await?
                .write(core, lr),
            JavaSystemSvcId::RaiseNullPointerException => EmulatedFunction::call(&java_raise_null_pointer_exception, core, jvm)
                .await?
                .write(core, lr),
            JavaSystemSvcId::RaiseArrayIndexException => EmulatedFunction::call(&java_raise_array_index_exception, core, jvm)
                .await?
                .write(core, lr),
            JavaSystemSvcId::RaiseArithmeticException => EmulatedFunction::call(&java_raise_arithmetic_exception, core, jvm).await?.write(core, lr),
            JavaSystemSvcId::Unk1 => EmulatedFunction::call(&java_unk1, core, &mut ()).await?.write(core, lr),
            JavaSystemSvcId::Unk2 => EmulatedFunction::call(&java_unk2, core, &mut ()).await?.write(core, lr),
            JavaSystemSvcId::Unk3 => EmulatedFunction::call(&java_unk3, core, &mut ()).await?.write(core, lr),
        }
    }
    .await;

    match result {
        Ok(()) => Ok(JumpTo(lr)),
        Err(WieError::JavaException(ptr_exception)) => match exception_state.unwind(core, ptr_exception)? {
            Some(resume_address) => Ok(JumpTo(resume_address)),
            None => Err(WieError::JavaException(ptr_exception)),
        },
        Err(error) => Err(error),
    }
}
