use alloc::{boxed::Box, collections::BTreeMap, string::String, vec::Vec};

use jvm::{runtime::JavaLangString, ClassInstance};
use wie_backend::{App, Event, System};
use wie_core_arm::{Allocator, ArmCore};

use crate::runtime::KtfJvmSupport;

pub const IMAGE_BASE: u32 = 0x100000;

pub struct KtfApp {
    core: ArmCore,
    system: System,
    jar_filename: String,
    main_class_name: Option<String>,
}

impl KtfApp {
    pub fn new(jar_filename: String, files: BTreeMap<String, Vec<u8>>, main_class_name: Option<String>, system: System) -> anyhow::Result<Self> {
        let mut core = ArmCore::new()?;

        system.filesystem().mount_zip(files.get(&jar_filename).unwrap()); // TODO implement resource loading to load from java classloader

        for (path, data) in files {
            let path = path.trim_start_matches("P/");
            system.filesystem().add(path, data.clone());
        }

        Allocator::init(&mut core)?;

        Ok(Self {
            core,
            system,
            jar_filename,
            main_class_name,
        })
    }

    #[tracing::instrument(name = "start", skip_all)]
    async fn do_start(core: &mut ArmCore, system: &mut System, jar_filename: String, main_class_name: Option<String>) -> anyhow::Result<()> {
        let (jvm, class_loader) = KtfJvmSupport::init(core, system, Some(&jar_filename)).await?;

        let main_class_name = if let Some(x) = main_class_name {
            x
        } else {
            anyhow::bail!("Main class not found");
        };

        let main_class_name = main_class_name.replace('.', "/");

        let main_class_name_java = JavaLangString::from_rust_string(&jvm, &main_class_name).await?;
        let _main_class: Box<dyn ClassInstance> = jvm
            .invoke_virtual(
                &class_loader,
                "loadClass",
                "(Ljava/lang/String;)Ljava/lang/Class;",
                (main_class_name_java,),
            )
            .await?;
        // TODO can't we use java/lang/Class above?
        let main_class = jvm.new_class(&main_class_name, "()V", []).await?;

        tracing::debug!("Main class instance: {:?}", &main_class);

        let arg = jvm.instantiate_array("Ljava/lang/String;", 0).await?;
        let _: () = jvm
            .invoke_virtual(&main_class, "startApp", "([Ljava/lang/String;)V", [arg.into()])
            .await?;

        Ok(())
    }
}

impl App for KtfApp {
    fn start(&mut self) -> anyhow::Result<()> {
        let mut core = self.core.clone();
        let mut system = self.system.clone();
        let jar_filename = self.jar_filename.clone();
        let main_class_name = self.main_class_name.clone();

        self.system
            .clone()
            .spawn(move || async move { Self::do_start(&mut core, &mut system, jar_filename, main_class_name).await });

        Ok(())
    }

    fn on_event(&mut self, event: Event) {
        self.system.event_queue().push(event)
    }

    fn tick(&mut self) -> anyhow::Result<()> {
        self.system
            .tick()
            .map_err(|x| anyhow::anyhow!("{}\n{}", x, self.core.dump_reg_stack(IMAGE_BASE)))
    }
}
