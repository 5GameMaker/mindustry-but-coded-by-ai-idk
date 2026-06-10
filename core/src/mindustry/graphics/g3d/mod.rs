//! 3D graphics abstractions.

pub mod generic_mesh;
pub mod hex_mesh;
pub mod hex_mesher;
pub mod hex_sky_mesh;
pub mod mat_mesh;
pub mod mesh_builder;
pub mod multi_mesh;
pub mod noise_mesh;
pub mod planet_grid;
pub mod planet_renderer;
pub mod shader_sphere_mesh;
pub mod sun_mesh;

pub use generic_mesh::{Disposable, GenericMesh, Mat3D, PlanetParams};
pub use hex_mesh::*;
pub use hex_mesher::{
    simplex_noise3d, DefaultHexMesher, FixedColorHexMesher, G3dColor, G3dVec3, HexMesher,
};
pub use hex_sky_mesh::*;
pub use mat_mesh::MatMesh;
pub use mesh_builder::*;
pub use multi_mesh::MultiMesh;
pub use noise_mesh::*;
pub use planet_grid::*;
pub use planet_renderer::*;
pub use shader_sphere_mesh::{MeshGeometry, PlanetMesh, ShaderSphereMesh};
pub use sun_mesh::*;
