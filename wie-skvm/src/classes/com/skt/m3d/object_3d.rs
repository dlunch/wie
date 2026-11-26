use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::WieJvmContext;

const MATRIX_SCALE: i32 = 1_000_000_000;

// class com.skt.m3d.Object3D
#[java_class(name = "com/skt/m3d/Object3D", public)]
pub struct Object3D {
    #[field(private)]
    name: ClassInstanceRef<String>,
}

#[java_methods]
impl Object3D {
    #[method(constructor, public)]
    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, name: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::debug!("com.skt.m3d.Object3D::<init>({this:?}, {name:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
        jvm.put_field(&mut this, "com/skt/m3d/Object3D", "name", "Ljava/lang/String;", name).await
    }

    #[allow(clippy::too_many_arguments)]
    #[method(constructor, public)]
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
        jvm.put_field(&mut this, "com/skt/m3d/Object3D", "name", "Ljava/lang/String;", name).await
    }

    #[method(public)]
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

    #[method(public)]
    async fn add_vertex(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, x: i32, y: i32, z: i32) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m3d.Object3D::addVertex({this:?}, {x}, {y}, {z})");
        Ok(())
    }

    #[method(public)]
    async fn get_matrix_row_0(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Array<i32>>> {
        tracing::warn!("stub com.skt.m3d.Object3D::getMatrixRow0({this:?})");

        let mut row = jvm.instantiate_array("I", 4).await?;
        jvm.store_array(&mut row, 0, [MATRIX_SCALE, 0, 0, 0]).await?;
        Ok(row.into())
    }

    #[method(public)]
    async fn get_matrix_row_1(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Array<i32>>> {
        tracing::warn!("stub com.skt.m3d.Object3D::getMatrixRow1({this:?})");

        let mut row = jvm.instantiate_array("I", 4).await?;
        jvm.store_array(&mut row, 0, [0, MATRIX_SCALE, 0, 0]).await?;
        Ok(row.into())
    }

    #[method(public)]
    async fn get_matrix_row_2(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Array<i32>>> {
        tracing::warn!("stub com.skt.m3d.Object3D::getMatrixRow2({this:?})");

        let mut row = jvm.instantiate_array("I", 4).await?;
        jvm.store_array(&mut row, 0, [0, 0, MATRIX_SCALE, 0]).await?;
        Ok(row.into())
    }

    #[method(public)]
    async fn get_name(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
        jvm.get_field(&this, "com/skt/m3d/Object3D", "name", "Ljava/lang/String;").await
    }

    #[method(public)]
    async fn rotate(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, x: i32, y: i32, z: i32) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m3d.Object3D::rotate({this:?}, {x}, {y}, {z})");
        Ok(())
    }

    #[method(public)]
    async fn scale(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, x: i32, y: i32, z: i32) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m3d.Object3D::scale({this:?}, {x}, {y}, {z})");
        Ok(())
    }

    #[method(public)]
    async fn set_name(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, name: ClassInstanceRef<String>) -> JvmResult<()> {
        jvm.put_field(&mut this, "com/skt/m3d/Object3D", "name", "Ljava/lang/String;", name).await
    }

    #[method(public)]
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

    #[method(public)]
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

    #[method(public)]
    async fn translate(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, x: i32, y: i32, z: i32) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m3d.Object3D::translate({this:?}, {x}, {y}, {z})");
        Ok(())
    }
}
