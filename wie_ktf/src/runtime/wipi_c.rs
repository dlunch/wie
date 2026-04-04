use alloc::{boxed::Box, vec};

use jvm::Jvm;
use wie_backend::System;
use wie_core_arm::{ArmCore, EmulatedFunction, EmulatedFunctionParam, ResultWriter, SvcId};
use wie_util::{Result, WieError};
use wie_wipi_c::{WIPICMethodBody, WIPICResult};

use crate::runtime::SVC_CATEGORY_WIPIC;
use crate::runtime::svc_ids::WIPICTableId;

mod context;
pub mod interface;
mod method_table;

use context::KtfWIPICContext;

struct WIPICMethodResult {
    result: WIPICResult,
}

impl ResultWriter<WIPICMethodResult> for WIPICMethodResult {
    fn write(self, core: &mut ArmCore, next_pc: u32) -> Result<()> {
        core.write_return_value(&self.result.results)?;
        core.set_next_pc(next_pc)?;

        Ok(())
    }
}

struct CMethodProxy {
    context: KtfWIPICContext,
    body: WIPICMethodBody,
}

#[async_trait::async_trait]
impl EmulatedFunction<(), WIPICMethodResult, ()> for CMethodProxy {
    async fn call(&self, core: &mut ArmCore, _: &mut ()) -> Result<WIPICMethodResult> {
        let a0 = u32::get(core, 0);
        let a1 = u32::get(core, 1);
        let a2 = u32::get(core, 2);
        let a3 = u32::get(core, 3);
        let a4 = u32::get(core, 4);
        let a5 = u32::get(core, 5);
        let a6 = u32::get(core, 6);
        let a7 = u32::get(core, 7);
        let a8 = u32::get(core, 8);

        let result = self
            .body
            .call(&mut self.context.clone(), vec![a0, a1, a2, a3, a4, a5, a6, a7, a8].into_boxed_slice())
            .await?;

        Ok(WIPICMethodResult { result })
    }
}

async fn handle_wipic_svc(core: &mut ArmCore, (system, jvm): &mut (System, Jvm), id: SvcId) -> Result<()> {
    let table_id = WIPICTableId::try_from(id.0 >> 16)?;
    let function_id = id.0 as u16;
    let (_, lr) = core.read_pc_lr()?;
    if table_id == WIPICTableId::Kernel && function_id == 33 {
        return interface::get_wipic_interfaces(core, &mut KtfWIPICContext::new(core.clone(), system.clone(), jvm.clone()))
            .await?
            .write(core, lr);
    }

    let body = method_table::get_method_body(table_id, function_id)
        .ok_or_else(|| WieError::FatalError(alloc::format!("Unknown KTF WIPIC SVC id {:#x}", id.0)))?;

    EmulatedFunction::call(
        &CMethodProxy {
            context: KtfWIPICContext::new(core.clone(), system.clone(), jvm.clone()),
            body,
        },
        core,
        &mut (),
    )
    .await?
    .write(core, lr)
}

pub(crate) fn register_wipic_svc_handler(core: &mut ArmCore, system: &System, jvm: &Jvm) -> Result<()> {
    core.register_svc_handler(SVC_CATEGORY_WIPIC, handle_wipic_svc, &(system.clone(), jvm.clone()))
}
