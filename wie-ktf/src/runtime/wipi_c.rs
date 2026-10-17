use alloc::{boxed::Box, vec};

use jvm::Jvm;
use wie_backend::System;
use wie_core_arm::{ArmCore, EmulatedFunction, EmulatedFunctionParam, ResultWriter, SvcId};
use wie_util::{Result, WieError};
use wie_wipi_c::{WIPICMethodBody, WIPICResult};

use crate::runtime::SVC_CATEGORY_WIPIC;
use crate::runtime::svc_ids::{WIPICKernelMethodId, WIPICTableId};

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
    if table_id == WIPICTableId::Kernel && function_id == WIPICKernelMethodId::Reserved1 as u16 {
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

pub fn register_wipic_svc_handler(core: &mut ArmCore, system: &System, jvm: &Jvm) -> Result<()> {
    core.register_svc_handler(SVC_CATEGORY_WIPIC, handle_wipic_svc, &(system.clone(), jvm.clone()))
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, sync::Arc};
    use core::sync::atomic::{AtomicUsize, Ordering};

    use test_utils::{TestPlatform, run_jvm_test_with_system};
    use wipi_types::{
        ktf::wipic::{WIPICGraphicsInterface, WIPICInterface},
        wipic::WIPICFramebuffer,
    };

    use wie_backend::Options;
    use wie_core_arm::{Allocator, ArmCore};
    use wie_util::{Result, read_generic, write_generic};

    use super::{SVC_CATEGORY_WIPIC, WIPICKernelMethodId, WIPICTableId, register_wipic_svc_handler};

    #[test]
    fn native_screen_flush_presents_guest_framebuffer() -> Result<()> {
        let paints = Arc::new(AtomicUsize::new(0));
        let observed_paints = paints.clone();
        let platform = TestPlatform::with_paint_handler(move |image| {
            assert_eq!((image.width(), image.height()), (3, 2));
            let expected = [(255, 0, 0), (0, 255, 0)][observed_paints.fetch_add(1, Ordering::Relaxed)];
            let pixel = image.get_pixel(2, 1);
            assert_eq!((pixel.r, pixel.g, pixel.b), expected);
        });
        run_jvm_test_with_system(Box::new([]), Box::new(platform), |jvm, system| async move {
            system.platform().screen().resize(3, 2).unwrap();
            let mut core = ArmCore::new(Options {
                enable_gdbserver: false,
                aot: None,
                profile: None,
            })
            .unwrap();
            Allocator::init(&mut core).unwrap();
            let mut registers = core.save_context();
            registers.sp = Allocator::alloc(&mut core, 0x100).unwrap() + 0x100;
            core.restore_context(&registers);
            register_wipic_svc_handler(&mut core, &system, &jvm).unwrap();
            let get_interfaces = core
                .make_svc_stub(SVC_CATEGORY_WIPIC, WIPICTableId::Kernel.function_id(WIPICKernelMethodId::Reserved1))
                .unwrap();
            let interfaces: u32 = core.run_function(get_interfaces, &[]).await.unwrap();
            let interfaces: WIPICInterface = read_generic(&core, interfaces).unwrap();
            let graphics: WIPICGraphicsInterface = read_generic(&core, interfaces.graphics_interface).unwrap();
            let framebuffer: u32 = core.run_function(graphics.get_screen_framebuffer, &[0]).await.unwrap();
            let framebuffer_data: u32 = read_generic(&core, framebuffer).unwrap();
            let framebuffer: WIPICFramebuffer = read_generic(&core, framebuffer_data + 8).unwrap();
            let pixels: u32 = read_generic(&core, framebuffer.buf.0).unwrap();
            let flush: u32 = read_generic(&core, interfaces.interface_12 + 4).unwrap();
            for color in [0xf800u16, 0x07e0u16] {
                write_generic(&mut core, pixels + 8 + 10, color).unwrap();
                core.run_function::<u32>(flush, &[]).await.unwrap();
            }
            Ok(())
        })?;
        assert_eq!(paints.load(Ordering::Relaxed), 2);
        Ok(())
    }
}
