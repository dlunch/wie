use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
use wie_midp::classes::{
    javax::microedition::lcdui::{Command, CommandListener},
    net::wie::MIDPKeyCode,
};

// The shared MIDP event queue uses SKVM codes; Nokia callbacks use these values.
const KEY_CODES: [(&str, i32, i32); 9] = [
    ("KEY_UP_ARROW", MIDPKeyCode::UP as i32, -1),
    ("KEY_DOWN_ARROW", MIDPKeyCode::DOWN as i32, -2),
    ("KEY_LEFT_ARROW", MIDPKeyCode::LEFT as i32, -3),
    ("KEY_RIGHT_ARROW", MIDPKeyCode::RIGHT as i32, -4),
    ("KEY_SOFTKEY3", MIDPKeyCode::FIRE as i32, -5),
    ("KEY_SOFTKEY1", MIDPKeyCode::LEFT_SOFT_KEY as i32, -6),
    ("KEY_SOFTKEY2", MIDPKeyCode::RIGHT_SOFT_KEY as i32, -7),
    ("KEY_SEND", MIDPKeyCode::CALL as i32, -10),
    ("KEY_END", MIDPKeyCode::HANGUP as i32, -11),
];

// abstract class com.nokia.mid.ui.FullCanvas
pub struct FullCanvas;

impl FullCanvas {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/nokia/mid/ui/FullCanvas",
            parent_class: Some("javax/microedition/lcdui/Canvas"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<clinit>", "()V", Self::cl_init, MethodAccessFlags::STATIC),
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new(
                    "addCommand",
                    "(Ljavax/microedition/lcdui/Command;)V",
                    Self::add_command,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "setCommandListener",
                    "(Ljavax/microedition/lcdui/CommandListener;)V",
                    Self::set_command_listener,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("getGameAction", "(I)I", Self::get_game_action, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getKeyCode", "(I)I", Self::get_key_code, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("handleKeyEvent", "(II)V", Self::handle_key_event, MethodAccessFlags::PROTECTED),
            ],
            fields: KEY_CODES
                .iter()
                .map(|(name, _, _)| JavaFieldProto::new(name, "I", FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL))
                .collect(),
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::ABSTRACT,
        }
    }

    async fn cl_init(jvm: &Jvm, _: &mut WieJvmContext) -> JvmResult<()> {
        tracing::debug!("com.nokia.mid.ui.FullCanvas::<clinit>()");

        for (name, _, code) in KEY_CODES {
            jvm.put_static_field("com/nokia/mid/ui/FullCanvas", name, "I", code).await?;
        }
        Ok(())
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("com.nokia.mid.ui.FullCanvas::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "javax/microedition/lcdui/Canvas", "<init>", "()V", ()).await?;
        jvm.invoke_special(&this, "javax/microedition/lcdui/Canvas", "setFullScreenMode", "(Z)V", (true,))
            .await
    }

    async fn add_command(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, command: ClassInstanceRef<Command>) -> JvmResult<()> {
        tracing::debug!("com.nokia.mid.ui.FullCanvas::addCommand({this:?}, {command:?})");

        Err(jvm
            .exception("java/lang/IllegalStateException", "FullCanvas does not support commands")
            .await)
    }

    async fn set_command_listener(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        listener: ClassInstanceRef<CommandListener>,
    ) -> JvmResult<()> {
        tracing::debug!("com.nokia.mid.ui.FullCanvas::setCommandListener({this:?}, {listener:?})");

        Err(jvm
            .exception("java/lang/IllegalStateException", "FullCanvas does not support commands")
            .await)
    }

    async fn get_game_action(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, code: i32) -> JvmResult<i32> {
        tracing::debug!("com.nokia.mid.ui.FullCanvas::getGameAction({this:?}, {code})");

        let code = KEY_CODES
            .iter()
            .find_map(|(_, midp, nokia)| (*nokia == code).then_some(*midp))
            .unwrap_or(code);
        jvm.invoke_special(&this, "javax/microedition/lcdui/Canvas", "getGameAction", "(I)I", (code,))
            .await
    }

    async fn get_key_code(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, action: i32) -> JvmResult<i32> {
        tracing::debug!("com.nokia.mid.ui.FullCanvas::getKeyCode({this:?}, {action})");

        let code: i32 = jvm
            .invoke_special(&this, "javax/microedition/lcdui/Canvas", "getKeyCode", "(I)I", (action,))
            .await?;
        Ok(KEY_CODES
            .iter()
            .find_map(|(_, midp, nokia)| (*midp == code).then_some(*nokia))
            .unwrap_or(code))
    }

    async fn handle_key_event(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, event_type: i32, code: i32) -> JvmResult<()> {
        tracing::debug!("com.nokia.mid.ui.FullCanvas::handleKeyEvent({this:?}, {event_type}, {code})");

        let code = KEY_CODES
            .iter()
            .find_map(|(_, midp, nokia)| (*midp == code).then_some(*nokia))
            .unwrap_or(code);
        jvm.invoke_special(&this, "javax/microedition/lcdui/Canvas", "handleKeyEvent", "(II)V", (event_type, code))
            .await
    }
}
