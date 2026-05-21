//! 3D graphics abstractions.

pub mod generic_mesh;
pub mod shader_sphere_mesh;

pub use generic_mesh::{Disposable, GenericMesh, Mat3D, PlanetParams};
pub use shader_sphere_mesh::{MeshGeometry, PlanetMesh, ShaderSphereMesh};
