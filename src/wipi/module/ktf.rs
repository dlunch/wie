mod context;
mod runtime;

use crate::{
    core::arm::{allocator::Allocator, ArmCore},
    wipi::module::ktf::runtime::call_java_method,
};

use self::{context::Context, runtime::JavaMethodFullname};

// client.bin from jar, extracted from ktf phone
pub struct KtfWipiModule {
    core: ArmCore,
    context: Context,
    main_class_instance: u32,
}

impl KtfWipiModule {
    pub fn new(data: &[u8], filename: &str, main_class: &str) -> anyhow::Result<Self> {
        let mut core = ArmCore::new()?;
        let context = Context::new(Allocator::new(&mut core)?);

        let (base_address, bss_size) = Self::load(&mut core, data, filename)?;

        let main_class_instance = Self::init(&mut core, &context, base_address, bss_size, main_class)?;

        Ok(Self {
            core,
            context,
            main_class_instance,
        })
    }

    pub fn start(&mut self) -> anyhow::Result<()> {
        runtime::call_java_method(
            &mut self.core,
            &self.context,
            self.main_class_instance,
            &JavaMethodFullname {
                tag: 64,
                name: "startApp".into(),
                signature: "([Ljava/lang/String;)V".into(),
            },
        )?;

        Ok(())
    }

    fn init(core: &mut ArmCore, context: &Context, base_address: u32, bss_size: u32, main_class: &str) -> anyhow::Result<u32> {
        let module = runtime::init(core, context, base_address, bss_size)?;

        log::info!("Call wipi init at {:#x}", module.fn_init);
        let result = core.run_function(module.fn_init, &[])?;
        if result != 0 {
            return Err(anyhow::anyhow!("wipi init failed with code {:#x}", result));
        }

        let main_class_name = context.alloc(20)?; // TODO size fix
        core.write_raw(main_class_name, main_class.as_bytes())?;

        log::info!("Call class getter at {:#x}", module.fn_get_class);
        let main_class = core.run_function(module.fn_get_class, &[main_class_name])?;
        if main_class == 0 {
            return Err(anyhow::anyhow!("Failed to get main class"));
        }
        context.borrow_mut().allocator.free(main_class_name);

        log::info!("Got main class: {:#x}", main_class);

        let instance = runtime::instantiate_java_class(core, context, main_class)?;

        call_java_method(
            core,
            context,
            instance,
            &JavaMethodFullname {
                tag: 72,
                name: "<init>".into(),
                signature: "()V".into(),
            },
        )?;

        log::info!("Main class instance: {:#x}", instance);

        Ok(instance)
    }

    fn load(core: &mut ArmCore, data: &[u8], filename: &str) -> anyhow::Result<(u32, u32)> {
        let bss_start = filename.find("client.bin").ok_or_else(|| anyhow::anyhow!("Incorrect filename"))? + 10;
        let bss_size = filename[bss_start..].parse::<u32>()?;

        let base_address = core.load(data, data.len() + bss_size as usize)?;

        log::info!("Loaded at {:#x}, size {:#x}, bss {:#x}", base_address, data.len(), bss_size);

        Ok((base_address, bss_size))
    }
}
