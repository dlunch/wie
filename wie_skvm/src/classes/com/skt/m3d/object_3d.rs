use alloc::vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_constants::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use java_runtime::classes::java::lang::String;
use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

const MATRIX_SCALE: i32 = 1_000_000_000;

// class com.skt.m3d.Object3D
pub struct Object3D;

impl Object3D {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/skt/m3d/Object3D",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljava/lang/String;)V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "<init>",
                    "(Ljava/lang/String;[I[I[I[I[I[I[I)V",
                    Self::init_with_geometry,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("addTriangle", "(IIII)V", Self::add_triangle, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("addVertex", "(III)V", Self::add_vertex, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getMatrixRow0", "()[I", Self::get_matrix_row_0, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getMatrixRow1", "()[I", Self::get_matrix_row_1, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getMatrixRow2", "()[I", Self::get_matrix_row_2, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getName", "()Ljava/lang/String;", Self::get_name, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("rotate", "(III)V", Self::rotate, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("scale", "(III)V", Self::scale, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setName", "(Ljava/lang/String;)V", Self::set_name, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setTriangles", "([I[I[I[I)V", Self::set_triangles, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setVertices", "([I[I[I)V", Self::set_vertices, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("translate", "(III)V", Self::translate, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![JavaFieldProto::new("name", "Ljava/lang/String;", FieldAccessFlags::PRIVATE)],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, name: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::debug!("com.skt.m3d.Object3D::<init>({this:?}, {name:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
        jvm.put_field(&mut this, "name", "Ljava/lang/String;", name).await
    }

    #[allow(clippy::too_many_arguments)]
    async fn init_with_geometry(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        name: ClassInstanceRef<String>,
        vertices_x: ClassInstanceRef<Array<i32>>,
        vertices_y: ClassInstanceRef<Array<i32>>,
        vertices_z: ClassInstanceRef<Array<i32>>,
        triangles_a: ClassInstanceRef<Array<i32>>,
        triangles_b: ClassInstanceRef<Array<i32>>,
        triangles_c: ClassInstanceRef<Array<i32>>,
        triangle_colors: ClassInstanceRef<Array<i32>>,
    ) -> JvmResult<()> {
        tracing::warn!(
            "stub com.skt.m3d.Object3D::<init>({this:?}, {name:?}, {vertices_x:?}, {vertices_y:?}, {vertices_z:?}, {triangles_a:?}, {triangles_b:?}, {triangles_c:?}, {triangle_colors:?})"
        );

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
        jvm.put_field(&mut this, "name", "Ljava/lang/String;", name).await
    }

    async fn add_triangle(
        _jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        vertex_a: i32,
        vertex_b: i32,
        vertex_c: i32,
        color: i32,
    ) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m3d.Object3D::addTriangle({this:?}, {vertex_a}, {vertex_b}, {vertex_c}, {color})");
        Ok(())
    }

    async fn add_vertex(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, x: i32, y: i32, z: i32) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m3d.Object3D::addVertex({this:?}, {x}, {y}, {z})");
        Ok(())
    }

    async fn get_matrix_row_0(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Array<i32>>> {
        tracing::warn!("stub com.skt.m3d.Object3D::getMatrixRow0({this:?})");

        let mut row = jvm.instantiate_array("I", 4).await?;
        jvm.store_array(&mut row, 0, [MATRIX_SCALE, 0, 0, 0]).await?;
        Ok(row.into())
    }

    async fn get_matrix_row_1(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Array<i32>>> {
        tracing::warn!("stub com.skt.m3d.Object3D::getMatrixRow1({this:?})");

        let mut row = jvm.instantiate_array("I", 4).await?;
        jvm.store_array(&mut row, 0, [0, MATRIX_SCALE, 0, 0]).await?;
        Ok(row.into())
    }

    async fn get_matrix_row_2(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Array<i32>>> {
        tracing::warn!("stub com.skt.m3d.Object3D::getMatrixRow2({this:?})");

        let mut row = jvm.instantiate_array("I", 4).await?;
        jvm.store_array(&mut row, 0, [0, 0, MATRIX_SCALE, 0]).await?;
        Ok(row.into())
    }

    async fn get_name(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
        jvm.get_field(&this, "name", "Ljava/lang/String;").await
    }

    async fn rotate(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, x: i32, y: i32, z: i32) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m3d.Object3D::rotate({this:?}, {x}, {y}, {z})");
        Ok(())
    }

    async fn scale(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, x: i32, y: i32, z: i32) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m3d.Object3D::scale({this:?}, {x}, {y}, {z})");
        Ok(())
    }

    async fn set_name(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, name: ClassInstanceRef<String>) -> JvmResult<()> {
        jvm.put_field(&mut this, "name", "Ljava/lang/String;", name).await
    }

    async fn set_triangles(
        _jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        vertices_a: ClassInstanceRef<Array<i32>>,
        vertices_b: ClassInstanceRef<Array<i32>>,
        vertices_c: ClassInstanceRef<Array<i32>>,
        colors: ClassInstanceRef<Array<i32>>,
    ) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m3d.Object3D::setTriangles({this:?}, {vertices_a:?}, {vertices_b:?}, {vertices_c:?}, {colors:?})");
        Ok(())
    }

    async fn set_vertices(
        _jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        vertices_x: ClassInstanceRef<Array<i32>>,
        vertices_y: ClassInstanceRef<Array<i32>>,
        vertices_z: ClassInstanceRef<Array<i32>>,
    ) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m3d.Object3D::setVertices({this:?}, {vertices_x:?}, {vertices_y:?}, {vertices_z:?})");
        Ok(())
    }

    async fn translate(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, x: i32, y: i32, z: i32) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m3d.Object3D::translate({this:?}, {x}, {y}, {z})");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use java_runtime::classes::java::lang::String;
    use jvm::{Array, ClassInstanceRef, runtime::JavaLangString};
    use test_utils::run_jvm_test;

    use super::{MATRIX_SCALE, Object3D};

    #[test]
    fn name_and_identity_matrix_remain_stable_across_geometry_calls() {
        let result = run_jvm_test(Box::new([[Object3D::as_proto()].into()]), |jvm| async move {
            let initial_name: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "cube").await?.into();
            let object: ClassInstanceRef<Object3D> = jvm
                .new_class("com/skt/m3d/Object3D", "(Ljava/lang/String;)V", (initial_name.clone(),))
                .await?
                .into();

            let returned_name: ClassInstanceRef<String> = jvm.invoke_virtual(&object, "getName", "()Ljava/lang/String;", ()).await?;
            assert_eq!(
                returned_name.instance.as_ref().map(|instance| instance.identity()),
                initial_name.instance.as_ref().map(|instance| instance.identity())
            );

            let renamed: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "renamed").await?.into();
            let _: () = jvm
                .invoke_virtual(&object, "setName", "(Ljava/lang/String;)V", (renamed.clone(),))
                .await?;
            let returned_name: ClassInstanceRef<String> = jvm.invoke_virtual(&object, "getName", "()Ljava/lang/String;", ()).await?;
            assert_eq!(
                returned_name.instance.as_ref().map(|instance| instance.identity()),
                renamed.instance.as_ref().map(|instance| instance.identity())
            );

            let _: () = jvm.invoke_virtual(&object, "addVertex", "(III)V", (1, 2, 3)).await?;
            let _: () = jvm.invoke_virtual(&object, "addTriangle", "(IIII)V", (0, 1, 2, 0xff00ff)).await?;
            let _: () = jvm.invoke_virtual(&object, "rotate", "(III)V", (10, 20, 30)).await?;
            let _: () = jvm.invoke_virtual(&object, "translate", "(III)V", (40, 50, 60)).await?;
            let _: () = jvm.invoke_virtual(&object, "scale", "(III)V", (2, 3, 4)).await?;

            let row_0: ClassInstanceRef<Array<i32>> = jvm.invoke_virtual(&object, "getMatrixRow0", "()[I", ()).await?;
            let row_1: ClassInstanceRef<Array<i32>> = jvm.invoke_virtual(&object, "getMatrixRow1", "()[I", ()).await?;
            let row_2: ClassInstanceRef<Array<i32>> = jvm.invoke_virtual(&object, "getMatrixRow2", "()[I", ()).await?;
            assert_eq!(jvm.load_array::<i32>(&row_0, 0, 4).await?, [MATRIX_SCALE, 0, 0, 0]);
            assert_eq!(jvm.load_array::<i32>(&row_1, 0, 4).await?, [0, MATRIX_SCALE, 0, 0]);
            assert_eq!(jvm.load_array::<i32>(&row_2, 0, 4).await?, [0, 0, MATRIX_SCALE, 0]);

            let another_row_0: ClassInstanceRef<Array<i32>> = jvm.invoke_virtual(&object, "getMatrixRow0", "()[I", ()).await?;
            assert_ne!(
                another_row_0.instance.as_ref().map(|instance| instance.identity()),
                row_0.instance.as_ref().map(|instance| instance.identity())
            );

            Ok(())
        });

        assert!(result.is_ok(), "JVM test failed: {result:?}");
    }
}
