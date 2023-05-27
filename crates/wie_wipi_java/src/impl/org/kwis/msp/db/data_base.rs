use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaContext, JavaMethodProto, JavaResult},
    proxy::JavaObjectProxy,
};

// class org.kwis.msp.db.DataBase
pub struct DataBase {}

impl DataBase {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![JavaMethodProto::new(
                "openDataBase",
                "(Ljava/lang/String;IZ)Lorg/kwis/msp/db/DataBase;",
                Self::open_data_base,
            )],
            fields: vec![],
        }
    }

    async fn open_data_base(_: &mut dyn JavaContext, _a0: JavaObjectProxy, _a1: u32, _a2: u32) -> JavaResult<JavaObjectProxy> {
        log::warn!("stub DataBase::openDataBase({:#x}, {}, {})", _a0.ptr_instance, _a1, _a2);

        Ok(JavaObjectProxy::new(0))
    }
}
