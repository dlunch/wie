use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_constants::MethodAccessFlags;
use java_runtime::classes::java::lang::String;
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class com.skt.m.MathFP
pub struct MathFP;

impl MathFP {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/skt/m/MathFP",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new(
                "parseFPString",
                "(Ljava/lang/String;)J",
                Self::parse_fp_string,
                MethodAccessFlags::STATIC,
            )],
            fields: vec![],
            access_flags: Default::default(),
        }
    }

    async fn parse_fp_string(jvm: &Jvm, _context: &mut WieJvmContext, s: ClassInstanceRef<String>) -> JvmResult<i64> {
        tracing::debug!("com.skt.m.MathFP::parseFPString({:?})", &s);

        let s = JavaLangString::to_rust_string(jvm, &s).await?;

        Ok(Self::f64_to_mathfp_i64(s.parse().unwrap()))
    }

    fn f64_to_mathfp_i64(f: f64) -> i64 {
        (f * 1000000000.0) as _
    }
}
