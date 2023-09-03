use alloc::string::String;

use futures::{future::LocalBoxFuture, FutureExt};

use wie_backend::Backend;
use wie_base::{util::ByteWrite, Core, Module};
use wie_core_arm::{Allocator, ArmCore};
use wie_wipi_java::{JavaContext, JavaObjectProxy};

use crate::runtime::KtfJavaContext;

// client.bin from jar, extracted from ktf phone
pub struct KtfWipiModule {
    core: ArmCore,
    entry: u32,
    render: u32,
    base_address: u32,
    bss_size: u32,
    ptr_main_class_name: u32,
}

impl KtfWipiModule {
    pub fn new(filename: &str, main_class_name: &str, backend: &Backend) -> anyhow::Result<Self> {
        let mut core = ArmCore::new()?;

        Allocator::init(&mut core)?;

        let resource = backend.resource();
        let data = resource.data(resource.id(filename).ok_or(anyhow::anyhow!("Resource not found"))?);

        let (base_address, bss_size) = Self::load(&mut core, data, filename)?;

        let ptr_main_class_name = Allocator::alloc(&mut core, 20)?; // TODO size fix
        core.write_bytes(ptr_main_class_name, main_class_name.as_bytes())?;

        let entry = core.register_function(Self::do_start, backend)?;
        let render = core.register_function(Self::do_render, backend)?;

        Ok(Self {
            core,
            entry,
            render,
            base_address,
            bss_size,
            ptr_main_class_name,
        })
    }

    async fn do_start(core: &mut ArmCore, backend: &mut Backend, base_address: u32, bss_size: u32, main_class_name: String) -> anyhow::Result<()> {
        let ptr_main_class = Self::init(core, backend, base_address, bss_size, &main_class_name).await?;

        let mut java_context = KtfJavaContext::new(core, backend);

        let instance = java_context.instantiate_from_ptr_class(ptr_main_class)?;
        java_context.call_method(&instance, "<init>", "()V", &[]).await?;

        log::debug!("Main class instance: {:#x}", instance.ptr_instance);

        let arg = java_context.instantiate_array("Ljava/lang/String;", 0)?;
        java_context
            .call_method(&instance, "startApp", "([Ljava/lang/String;)V", &[arg.ptr_instance])
            .await?;

        Ok(())
    }

    async fn do_render(core: &mut ArmCore, backend: &mut Backend) -> anyhow::Result<()> {
        let screen_canvas = backend.screen_canvas();

        let mut java_context = KtfJavaContext::new(core, backend);

        let display = java_context.get_static_field("org/kwis/msp/lcdui/Display", "display")?;
        if display == 0 {
            return Ok(());
        }

        let card = java_context.get_field(&JavaObjectProxy::new(display), "card")?;
        if card == 0 {
            return Ok(());
        }

        let graphics = java_context.instantiate("Lorg/kwis/msp/lcdui/Graphics;")?;
        java_context.call_method(&graphics, "<init>", "(I)V", &[screen_canvas]).await?;

        java_context
            .call_method(
                &JavaObjectProxy::new(card),
                "paint",
                "(Lorg/kwis/msp/lcdui/Graphics;)V",
                &[graphics.ptr_instance],
            )
            .await?;

        java_context.destroy(graphics)?;

        Ok(())
    }

    async fn init(core: &mut ArmCore, backend: &Backend, base_address: u32, bss_size: u32, main_class_name: &str) -> anyhow::Result<u32> {
        let module = crate::runtime::init(core, backend, base_address, bss_size).await?;

        log::debug!("Call wipi init at {:#x}", module.fn_init);
        let result = core.run_function::<u32>(module.fn_init, &[]).await?;
        if result != 0 {
            return Err(anyhow::anyhow!("wipi init failed with code {:#x}", result));
        }

        let ptr_main_class_name = Allocator::alloc(core, 20)?; // TODO size fix
        core.write_bytes(ptr_main_class_name, main_class_name.as_bytes())?;

        log::debug!("Call class getter at {:#x}", module.fn_get_class);
        let ptr_main_class = core.run_function(module.fn_get_class, &[ptr_main_class_name]).await?;
        if ptr_main_class == 0 {
            return Err(anyhow::anyhow!("Failed to get main class"));
        }
        Allocator::free(core, ptr_main_class_name)?;

        log::debug!("Got main class: {:#x}", ptr_main_class);

        Ok(ptr_main_class)
    }

    fn load(core: &mut ArmCore, data: &[u8], filename: &str) -> anyhow::Result<(u32, u32)> {
        let bss_start = filename.find("client.bin").ok_or_else(|| anyhow::anyhow!("Incorrect filename"))? + 10;
        let bss_size = filename[bss_start..].parse::<u32>()?;

        let base_address = core.load(data, data.len() + bss_size as usize)?;

        log::debug!("Loaded at {:#x}, size {:#x}, bss {:#x}", base_address, data.len(), bss_size);

        Ok((base_address, bss_size))
    }
}

impl Module for KtfWipiModule {
    fn core_mut(&mut self) -> &mut dyn Core {
        &mut self.core
    }

    fn start(&mut self) -> LocalBoxFuture<'static, anyhow::Result<()>> {
        self.core
            .run_function::<()>(self.entry, &[self.base_address, self.bss_size, self.ptr_main_class_name])
            .boxed_local()
    }

    fn render(&mut self) -> LocalBoxFuture<'static, anyhow::Result<()>> {
        self.core.run_function::<()>(self.render, &[]).boxed_local()
    }
}
