use jvm::{ClassInstanceRef, JavaChar, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};

use wie_jvm_support::WieJvmContext;

// interface com.xce.lcdui.TextComponent
#[java_class(name = "com/xce/lcdui/TextComponent", public, interface, abstract)]
pub struct TextComponent;

#[java_methods]
impl TextComponent {
    #[method(public)]
    async fn clear(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()>;

    #[method(public)]
    async fn delete(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()>;

    #[method(public)]
    async fn get_caret_position(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32>;

    #[method(public)]
    async fn get_constraints(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32>;

    #[method(public)]
    async fn get_max_size(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32>;

    #[method(public)]
    async fn insert(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, character: JavaChar) -> JvmResult<()>;

    #[method(public)]
    async fn move_cursor(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, direction: i32) -> JvmResult<()>;

    #[method(public)]
    async fn repaint(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()>;

    #[method(name = "repaintIM", public)]
    async fn repaint_im(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()>;

    #[method(public)]
    async fn replace(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, character: JavaChar) -> JvmResult<()>;

    #[method(public)]
    async fn set_caret_position(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, position: i32) -> JvmResult<()>;

    #[method(public)]
    async fn set_caret_visible(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, visible: bool) -> JvmResult<()>;

    #[method(public)]
    async fn size(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32>;
}
