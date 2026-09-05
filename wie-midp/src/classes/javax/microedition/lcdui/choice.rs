use alloc::vec;

use jvm::{Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// interface javax.microedition.lcdui.Choice
pub struct Choice;

impl Choice {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/lcdui/Choice",
            parent_class: None,
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<clinit>", "()V", Self::cl_init, MethodAccessFlags::STATIC),
                JavaMethodProto::new_abstract("size", "()I", MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT),
                JavaMethodProto::new_abstract(
                    "getString",
                    "(I)Ljava/lang/String;",
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT,
                ),
                JavaMethodProto::new_abstract(
                    "getImage",
                    "(I)Ljavax/microedition/lcdui/Image;",
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT,
                ),
                JavaMethodProto::new_abstract(
                    "append",
                    "(Ljava/lang/String;Ljavax/microedition/lcdui/Image;)I",
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT,
                ),
                JavaMethodProto::new_abstract(
                    "insert",
                    "(ILjava/lang/String;Ljavax/microedition/lcdui/Image;)V",
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT,
                ),
                JavaMethodProto::new_abstract("delete", "(I)V", MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT),
                JavaMethodProto::new_abstract("deleteAll", "()V", MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT),
                JavaMethodProto::new_abstract(
                    "set",
                    "(ILjava/lang/String;Ljavax/microedition/lcdui/Image;)V",
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT,
                ),
                JavaMethodProto::new_abstract("isSelected", "(I)Z", MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT),
                JavaMethodProto::new_abstract("getSelectedIndex", "()I", MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT),
                JavaMethodProto::new_abstract("getSelectedFlags", "([Z)I", MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT),
                JavaMethodProto::new_abstract("setSelectedIndex", "(IZ)V", MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT),
                JavaMethodProto::new_abstract("setSelectedFlags", "([Z)V", MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT),
                JavaMethodProto::new_abstract("setFitPolicy", "(I)V", MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT),
                JavaMethodProto::new_abstract("getFitPolicy", "()I", MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT),
                JavaMethodProto::new_abstract(
                    "setFont",
                    "(ILjavax/microedition/lcdui/Font;)V",
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT,
                ),
                JavaMethodProto::new_abstract(
                    "getFont",
                    "(I)Ljavax/microedition/lcdui/Font;",
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT,
                ),
            ],
            fields: [
                "EXCLUSIVE",
                "MULTIPLE",
                "IMPLICIT",
                "POPUP",
                "TEXT_WRAP_DEFAULT",
                "TEXT_WRAP_ON",
                "TEXT_WRAP_OFF",
            ]
            .into_iter()
            .map(|name| JavaFieldProto::new(name, "I", FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL))
            .collect(),
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::INTERFACE | ClassAccessFlags::ABSTRACT,
        }
    }

    async fn cl_init(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Choice::<clinit>");

        for (name, value) in [
            ("EXCLUSIVE", 1),
            ("MULTIPLE", 2),
            ("IMPLICIT", 3),
            ("POPUP", 4),
            ("TEXT_WRAP_DEFAULT", 0),
            ("TEXT_WRAP_ON", 1),
            ("TEXT_WRAP_OFF", 2),
        ] {
            jvm.put_static_field("javax/microedition/lcdui/Choice", name, "I", value).await?;
        }

        Ok(())
    }
}
