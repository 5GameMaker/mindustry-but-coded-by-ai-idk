//! Cubemap skybox mesh model mirroring upstream `mindustry.graphics.CubemapMesh`.

use super::g3d::{Disposable, Mat3D};

pub const CUBEMAP_MESH_VERTICES: [f32; 108] = [
    -1.0, 1.0, -1.0, -1.0, -1.0, -1.0, 1.0, -1.0, -1.0, 1.0, -1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 1.0,
    -1.0, -1.0, -1.0, 1.0, -1.0, -1.0, -1.0, -1.0, 1.0, -1.0, -1.0, 1.0, -1.0, -1.0, 1.0, 1.0,
    -1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, -1.0,
    1.0, -1.0, -1.0, -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, -1.0, 1.0,
    -1.0, -1.0, 1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, -1.0, 1.0, 1.0,
    -1.0, 1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 1.0, -1.0, -1.0, -1.0,
    -1.0, 1.0, 1.0, -1.0, 1.0,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CubemapTextureFilter {
    Linear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CubemapRenderPrimitive {
    Triangles,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CubemapRenderPlan {
    pub cubemap: String,
    pub projection: Mat3D,
    pub shader: String,
    pub uniform_name: String,
    pub uniform_slot: i32,
    pub primitive: CubemapRenderPrimitive,
    pub vertex_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CubemapMesh {
    pub cubemap: String,
    pub filter: CubemapTextureFilter,
    pub disposed: bool,
}

impl CubemapMesh {
    pub fn new(cubemap: impl Into<String>) -> Self {
        Self {
            cubemap: cubemap.into(),
            filter: CubemapTextureFilter::Linear,
            disposed: false,
        }
    }

    pub fn set_cubemap(&mut self, cubemap: impl Into<String>) {
        self.cubemap = cubemap.into();
    }

    pub fn render(&self, projection: &Mat3D) -> CubemapRenderPlan {
        CubemapRenderPlan {
            cubemap: self.cubemap.clone(),
            projection: *projection,
            shader: "shaders/cubemap".into(),
            uniform_name: "u_cubemap".into(),
            uniform_slot: 0,
            primitive: CubemapRenderPrimitive::Triangles,
            vertex_count: CUBEMAP_MESH_VERTICES.len() / 3,
        }
    }
}

impl Disposable for CubemapMesh {
    fn dispose(&mut self) {
        self.disposed = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cubemap_mesh_uses_upstream_cube_vertices() {
        assert_eq!(CUBEMAP_MESH_VERTICES.len(), 36 * 3);
        assert_eq!(
            &CUBEMAP_MESH_VERTICES[..6],
            &[-1.0, 1.0, -1.0, -1.0, -1.0, -1.0]
        );
        assert_eq!(
            &CUBEMAP_MESH_VERTICES[102..],
            &[-1.0, -1.0, 1.0, 1.0, -1.0, 1.0]
        );
    }

    #[test]
    fn cubemap_mesh_constructor_sets_linear_filter() {
        let mesh = CubemapMesh::new("cubemaps/stars/");

        assert_eq!(mesh.cubemap, "cubemaps/stars/");
        assert_eq!(mesh.filter, CubemapTextureFilter::Linear);
    }

    #[test]
    fn cubemap_mesh_render_plan_binds_cubemap_shader_uniform_zero() {
        let mut mesh = CubemapMesh::new("cubemaps/stars/");
        mesh.set_cubemap("cubemaps/clouds/");

        let plan = mesh.render(&Mat3D::translation(1.0, 2.0, 3.0));

        assert_eq!(plan.cubemap, "cubemaps/clouds/");
        assert_eq!(plan.projection, Mat3D::translation(1.0, 2.0, 3.0));
        assert_eq!(plan.shader, "shaders/cubemap");
        assert_eq!(plan.uniform_name, "u_cubemap");
        assert_eq!(plan.uniform_slot, 0);
        assert_eq!(plan.primitive, CubemapRenderPrimitive::Triangles);
        assert_eq!(plan.vertex_count, 36);
    }

    #[test]
    fn cubemap_mesh_disposes_cubemap_resource() {
        let mut mesh = CubemapMesh::new("cubemaps/stars/");

        mesh.dispose();

        assert!(mesh.disposed);
    }
}
