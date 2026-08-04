use alloc::{format, vec::Vec};
use core::mem::size_of;

use java_constants::MethodAccessFlags;
use jvm::Method;

use wie_core_arm::{Allocator, ArmCore};
use wie_util::{WieError, read_generic, write_generic};

use super::{JavaMethod, Result};

#[derive(Clone)]
pub struct JavaVtableEntry {
    pub target: u32,
    pub method: Option<JavaMethod>,
}

pub struct JavaVtable;

impl JavaVtable {
    const FIXED_ENTRY_COUNT: usize = 35;

    fn fixed_index(class_name: &str, super_class_name: Option<&str>, name: &str, descriptor: &str) -> Option<usize> {
        if matches!(class_name, "org/kwis/msp/lcdui/Jlet" | "org/kwis/msp/lcdui/JletWrapper")
            || matches!(super_class_name, Some("org/kwis/msp/lcdui/Jlet" | "org/kwis/msp/lcdui/JletWrapper"))
        {
            match (name, descriptor) {
                ("startApp", "([Ljava/lang/String;)V") => return Some(15),
                ("pauseApp", "()V") => return Some(16),
                ("resumeApp", "()V") => return Some(17),
                ("destroyApp", "(Z)V") => return Some(18),
                _ => {}
            }
        }

        match (class_name, name, descriptor) {
            ("java/lang/Object", "getClass", "()Ljava/lang/Class;") => Some(1),
            ("java/lang/Class", "getResourceAsStream", "(Ljava/lang/String;)Ljava/io/InputStream;") => Some(16),
            ("java/io/InputStream", "read", "([B)I") => Some(11),
            ("java/io/InputStream", "available", "()I") => Some(14),
            ("java/io/InputStream", "close", "()V") => Some(15),
            ("java/io/Reader", "read", "([C)I") => Some(11),
            ("java/io/Reader", "close", "()V") => Some(18),
            ("java/lang/String", "length", "()I") => Some(10),
            ("java/lang/String", "charAt", "(I)C") => Some(11),
            ("java/lang/String", "substring", "(II)Ljava/lang/String;") => Some(28),
            ("java/lang/StringBuffer", "toString", "()Ljava/lang/String;") => Some(4),
            ("java/lang/StringBuffer", "append", "(Ljava/lang/String;)Ljava/lang/StringBuffer;") => Some(18),
            ("java/lang/StringBuffer", "append", "(I)Ljava/lang/StringBuffer;") => Some(23),
            ("java/io/PrintStream", "println", "(Ljava/lang/String;)V") => Some(34),
            ("java/lang/Thread", "start", "()V") => Some(10),
            ("java/lang/Thread", "setPriority", "(I)V") => Some(14),
            ("java/util/Random", "nextInt", "()I") => Some(12),
            ("org/kwis/msp/lcdui/Card", "keyNotify", "(II)Z") => Some(25),
            ("org/kwis/msp/lcdui/Card", "paint", "(Lorg/kwis/msp/lcdui/Graphics;)V") => Some(27),
            _ => None,
        }
    }

    pub fn allocate(core: &mut ArmCore, entries: &[JavaVtableEntry]) -> Result<u32> {
        let ptr_allocation = Allocator::alloc(core, ((entries.len() + 2) * size_of::<u32>()) as u32)?;
        let ptr_vtable = ptr_allocation + size_of::<u32>() as u32;
        Self::write(core, ptr_vtable, entries)?;
        Ok(ptr_vtable)
    }

    pub fn read(core: &ArmCore, ptr_vtable: u32, known_methods: &[JavaMethod]) -> Result<Vec<JavaVtableEntry>> {
        let entry_count: u32 = read_generic(core, ptr_vtable - size_of::<u32>() as u32)?;
        (0..entry_count as usize)
            .map(|index| {
                let target = read_generic(core, ptr_vtable + ((index + 1) * size_of::<u32>()) as u32)?;
                let method = known_methods
                    .iter()
                    .find(|method| method.target().is_ok_and(|method_target| method_target == target))
                    .cloned();
                Ok(JavaVtableEntry { target, method })
            })
            .collect()
    }

    pub fn write(core: &mut ArmCore, ptr_vtable: u32, entries: &[JavaVtableEntry]) -> Result<()> {
        write_generic(core, ptr_vtable - size_of::<u32>() as u32, entries.len() as u32)?;
        write_generic(core, ptr_vtable, 0u32)?;
        for (index, entry) in entries.iter().enumerate() {
            write_generic(core, ptr_vtable + ((index + 1) * size_of::<u32>()) as u32, entry.target)?;
        }

        Ok(())
    }

    pub fn build_runtime_methods(
        class_name: &str,
        super_class_name: Option<&str>,
        parent_methods: &[JavaVtableEntry],
        declared_methods: &[JavaMethod],
    ) -> Result<Vec<JavaVtableEntry>> {
        let mut methods = parent_methods.to_vec();
        if class_name == "java/lang/Object" {
            methods.resize(Self::FIXED_ENTRY_COUNT, JavaVtableEntry { target: 0, method: None });
        }

        for method in declared_methods {
            let flags = method.access_flags();
            if flags.intersects(MethodAccessFlags::STATIC | MethodAccessFlags::PRIVATE) || method.name().starts_with('<') {
                continue;
            }

            let index = Self::fixed_index(class_name, super_class_name, &method.name(), &method.descriptor());
            if let Some(index) = index {
                if methods.len() <= index {
                    methods.resize(index + 1, JavaVtableEntry { target: 0, method: None });
                }
                let inherited = &methods[index];
                if inherited.target != 0
                    && !inherited
                        .method
                        .as_ref()
                        .is_some_and(|inherited| inherited.name() == method.name() && inherited.descriptor() == method.descriptor())
                {
                    return Err(WieError::FatalError(format!(
                        "Fixed vtable index collision for {class_name}.{}{} at index {index}",
                        method.name(),
                        method.descriptor()
                    )));
                }
                methods[index] = JavaVtableEntry {
                    target: method.target()?,
                    method: Some(method.clone()),
                };
            }
        }

        for method in declared_methods {
            let flags = method.access_flags();
            if flags.intersects(MethodAccessFlags::STATIC | MethodAccessFlags::PRIVATE) || method.name().starts_with('<') {
                continue;
            }

            if Self::fixed_index(class_name, super_class_name, &method.name(), &method.descriptor()).is_some() {
                continue;
            }

            if let Some(index) = methods.iter().position(|candidate| {
                candidate
                    .method
                    .as_ref()
                    .is_some_and(|candidate| candidate.name() == method.name() && candidate.descriptor() == method.descriptor())
            }) {
                methods[index] = JavaVtableEntry {
                    target: method.target()?,
                    method: Some(method.clone()),
                };
            } else {
                methods.push(JavaVtableEntry {
                    target: method.target()?,
                    method: Some(method.clone()),
                });
            }
        }

        Ok(methods)
    }

    pub fn build_generated_methods(
        class_name: &str,
        super_class_name: Option<&str>,
        is_jlet_subclass: bool,
        is_card_subclass: bool,
        parent_methods: &[JavaVtableEntry],
        declared_methods: &[JavaMethod],
    ) -> Result<Vec<JavaVtableEntry>> {
        let mut methods = parent_methods.to_vec();

        for method in declared_methods {
            let flags = method.access_flags();
            if flags.intersects(MethodAccessFlags::STATIC | MethodAccessFlags::PRIVATE) || method.name().starts_with('<') {
                continue;
            }

            let name = method.name();
            let descriptor = method.descriptor();
            let inherited_index = methods.iter().position(|candidate| {
                candidate
                    .method
                    .as_ref()
                    .is_some_and(|candidate| candidate.name() == name && candidate.descriptor() == descriptor)
            });
            let confirmed_index = match (is_jlet_subclass, is_card_subclass, name.as_str(), descriptor.as_str()) {
                (true, _, "startApp", "([Ljava/lang/String;)V") => Some(15),
                (true, _, "pauseApp", "()V") => Some(16),
                (true, _, "resumeApp", "()V") => Some(17),
                (true, _, "destroyApp", "(Z)V") => Some(18),
                (_, true, "keyNotify", "(II)Z") => Some(25),
                (_, true, "paint", "(Lorg/kwis/msp/lcdui/Graphics;)V") => Some(27),
                _ => None,
            };

            if let Some(index) = inherited_index {
                if confirmed_index != Some(index) {
                    return Err(WieError::FatalError(format!(
                        "Unknown generated override {class_name}.{name}{descriptor}"
                    )));
                }
                methods[index] = JavaVtableEntry {
                    target: method.target()?,
                    method: Some(method.clone()),
                };
            } else if let Some(index) = confirmed_index
                && super_class_name == Some("org/kwis/msp/lcdui/Jlet")
            {
                if methods.len() <= index {
                    methods.resize(index + 1, JavaVtableEntry { target: 0, method: None });
                }
                if methods[index].target != 0 {
                    return Err(WieError::FatalError(format!(
                        "Fixed vtable index collision for {class_name}.{name}{descriptor} at index {index}"
                    )));
                }
                methods[index] = JavaVtableEntry {
                    target: method.target()?,
                    method: Some(method.clone()),
                };
            } else if confirmed_index.is_some() {
                return Err(WieError::FatalError(format!(
                    "Unknown generated override {class_name}.{name}{descriptor}"
                )));
            }
        }

        Ok(methods)
    }
}
