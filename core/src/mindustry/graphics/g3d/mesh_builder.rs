//! Mesh construction plans mirroring upstream `mindustry.graphics.g3d.MeshBuilder`.

use super::{FixedColorHexMesher, G3dColor, G3dVec3, HexMesher, PlanetGrid};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeshBuildKind {
    Icosphere,
    PlanetGrid,
    Hex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MeshVertexAttributes {
    pub normal: bool,
    pub color: bool,
    pub emissive: bool,
    pub packed_normal: bool,
}

impl MeshVertexAttributes {
    pub const fn new(normal: bool, color: bool, emissive: bool, packed_normal: bool) -> Self {
        Self {
            normal,
            color,
            emissive,
            packed_normal,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MeshVertex {
    pub position: G3dVec3,
    pub normal: G3dVec3,
    pub color: G3dColor,
    pub emissive: Option<G3dColor>,
}

impl MeshVertex {
    pub const fn new(
        position: G3dVec3,
        normal: G3dVec3,
        color: G3dColor,
        emissive: Option<G3dColor>,
    ) -> Self {
        Self {
            position,
            normal,
            color,
            emissive,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeshBuildPlan {
    pub kind: MeshBuildKind,
    pub divisions: i32,
    pub radius: f32,
    pub intensity: f32,
    pub indexed: bool,
    pub attributes: MeshVertexAttributes,
    pub vertex_capacity: usize,
    pub index_capacity: usize,
    pub vertices: Vec<MeshVertex>,
    pub indices: Vec<u16>,
}

impl MeshBuildPlan {
    fn new(
        kind: MeshBuildKind,
        divisions: i32,
        radius: f32,
        intensity: f32,
        indexed: bool,
        attributes: MeshVertexAttributes,
        vertex_capacity: usize,
        index_capacity: usize,
    ) -> Self {
        Self {
            kind,
            divisions,
            radius,
            intensity,
            indexed,
            attributes,
            vertex_capacity,
            index_capacity,
            vertices: Vec::with_capacity(vertex_capacity),
            indices: Vec::with_capacity(index_capacity),
        }
    }
}

pub struct MeshBuilder;

impl MeshBuilder {
    pub fn build_icosphere(divisions: i32, radius: f32) -> MeshBuildPlan {
        let faces = 20 * 4usize.pow(divisions as u32);
        let vertices = 10 * 4usize.pow(divisions as u32) + 2;
        MeshBuildPlan::new(
            MeshBuildKind::Icosphere,
            divisions,
            radius,
            0.0,
            true,
            MeshVertexAttributes::new(false, false, false, false),
            vertices,
            faces * 3,
        )
    }

    pub fn build_planet_grid(grid: &PlanetGrid, color: G3dColor, scale: f32) -> MeshBuildPlan {
        let mut plan = MeshBuildPlan::new(
            MeshBuildKind::PlanetGrid,
            grid.size,
            scale,
            0.0,
            false,
            MeshVertexAttributes::new(false, true, false, false),
            grid.tiles.len() * 12,
            0,
        );

        for tile in &grid.tiles {
            for i in 0..tile.corners.len() {
                let v1 = grid.corners[tile.corners[i].unwrap()].v.scl(scale);
                let v2 = grid.corners[tile.corners[(i + 1) % tile.corners.len()].unwrap()]
                    .v
                    .scl(scale);
                plan.vertices
                    .push(MeshVertex::new(v1, G3dVec3::default(), color, None));
                plan.vertices
                    .push(MeshVertex::new(v2, G3dVec3::default(), color, None));
            }
        }

        plan
    }

    pub fn build_hex_color(color: G3dColor, divisions: i32, radius: f32) -> MeshBuildPlan {
        Self::build_hex(&FixedColorHexMesher::new(color), divisions, radius, 0.0)
    }

    pub fn build_hex<M: HexMesher>(
        mesher: &M,
        divisions: i32,
        radius: f32,
        intensity: f32,
    ) -> MeshBuildPlan {
        let grid = PlanetGrid::create(divisions);
        let emit = mesher.is_emissive();
        let indexed = grid.tiles.len() * 6 < 65535;
        let vertex_capacity = if indexed {
            grid.tiles.len() * 6
        } else {
            grid.tiles.len() * 12
        };
        let index_capacity = if indexed { grid.tiles.len() * 4 * 3 } else { 0 };
        let mut plan = MeshBuildPlan::new(
            MeshBuildKind::Hex,
            divisions,
            radius,
            intensity,
            indexed,
            MeshVertexAttributes::new(true, true, emit, false),
            vertex_capacity,
            index_capacity,
        );

        let heights: Vec<f32> = grid
            .corners
            .iter()
            .map(|corner| (1.0 + mesher.get_height(corner.v) * intensity) * radius)
            .collect();
        let mut position = 0u16;

        for tile in &grid.tiles {
            if mesher.skip(tile.v) {
                continue;
            }

            let corners: Vec<usize> = tile.corners.iter().map(|corner| corner.unwrap()).collect();
            let normal = normal(
                grid.corners[corners[0]].v.scl(heights[corners[0]]),
                grid.corners[corners[2]].v.scl(heights[corners[2]]),
                grid.corners[corners[4]].v.scl(heights[corners[4]]),
            );

            let mut color = G3dColor::WHITE;
            mesher.get_color(tile.v, &mut color);

            let emissive = if emit {
                let mut out = G3dColor::CLEAR;
                mesher.get_emissive_color(tile.v, &mut out);
                Some(out)
            } else {
                None
            };

            if indexed {
                for &corner in &corners {
                    push_vertex(&mut plan, &grid, &heights, corner, normal, color, emissive);
                }

                plan.indices
                    .extend_from_slice(&[position, position + 1, position + 2]);
                plan.indices
                    .extend_from_slice(&[position, position + 2, position + 3]);
                plan.indices
                    .extend_from_slice(&[position, position + 3, position + 4]);
                if corners.len() > 5 {
                    plan.indices
                        .extend_from_slice(&[position, position + 4, position + 5]);
                }

                position += corners.len() as u16;
            } else {
                push_triangle(
                    &mut plan, &grid, &heights, corners[0], corners[1], corners[2], normal, color,
                    emissive,
                );
                push_triangle(
                    &mut plan, &grid, &heights, corners[0], corners[2], corners[3], normal, color,
                    emissive,
                );
                push_triangle(
                    &mut plan, &grid, &heights, corners[0], corners[3], corners[4], normal, color,
                    emissive,
                );
                if corners.len() > 5 {
                    push_triangle(
                        &mut plan, &grid, &heights, corners[0], corners[4], corners[5], normal,
                        color, emissive,
                    );
                }
            }
        }

        plan
    }

    pub fn pack_normals(x: f32, y: f32, z: f32) -> f32 {
        let xs = if x < -1.0 / 512.0 { 1 } else { 0 };
        let ys = if y < -1.0 / 512.0 { 1 } else { 0 };
        let zs = if z < -1.0 / 512.0 { 1 } else { 0 };

        let vi = zs << 29
            | (((z * 511.0 + (zs << 9) as f32) as i32 & 511) << 20)
            | ys << 19
            | (((y * 511.0 + (ys << 9) as f32) as i32 & 511) << 10)
            | xs << 9
            | ((x * 511.0 + (xs << 9) as f32) as i32 & 511);
        f32::from_bits(vi as u32)
    }
}

pub fn icosphere_vertex_count(divisions: i32) -> usize {
    10 * 4usize.pow(divisions as u32) + 2
}

pub fn icosphere_index_count(divisions: i32) -> usize {
    20 * 4usize.pow(divisions as u32) * 3
}

fn push_triangle(
    plan: &mut MeshBuildPlan,
    grid: &PlanetGrid,
    heights: &[f32],
    a: usize,
    b: usize,
    c: usize,
    normal: G3dVec3,
    color: G3dColor,
    emissive: Option<G3dColor>,
) {
    push_vertex(plan, grid, heights, a, normal, color, emissive);
    push_vertex(plan, grid, heights, b, normal, color, emissive);
    push_vertex(plan, grid, heights, c, normal, color, emissive);
}

fn push_vertex(
    plan: &mut MeshBuildPlan,
    grid: &PlanetGrid,
    heights: &[f32],
    corner: usize,
    normal: G3dVec3,
    color: G3dColor,
    emissive: Option<G3dColor>,
) {
    let height = heights[corner];
    plan.vertices.push(MeshVertex::new(
        grid.corners[corner].v.scl(height),
        normal,
        color,
        emissive,
    ));
}

fn normal(v1: G3dVec3, v2: G3dVec3, v3: G3dVec3) -> G3dVec3 {
    v2.sub(v1).crs(v3.sub(v1)).nor()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::graphics::g3d::{tile_count, DefaultHexMesher};

    #[derive(Debug, Clone, Copy)]
    struct RaisedMesher;

    impl HexMesher for RaisedMesher {
        fn get_height(&self, _position: G3dVec3) -> f32 {
            2.0
        }

        fn get_color(&self, _position: G3dVec3, out: &mut G3dColor) {
            *out = G3dColor::rgb(0.1, 0.2, 0.3);
        }

        fn is_emissive(&self) -> bool {
            true
        }

        fn get_emissive_color(&self, _position: G3dVec3, out: &mut G3dColor) {
            *out = G3dColor::rgb(0.9, 0.8, 0.7);
        }
    }

    #[test]
    fn mesh_builder_icosphere_counts_match_subdivision_formula() {
        let plan = MeshBuilder::build_icosphere(3, 2.5);

        assert_eq!(plan.kind, MeshBuildKind::Icosphere);
        assert_eq!(plan.vertex_capacity, 10 * 4usize.pow(3) + 2);
        assert_eq!(plan.index_capacity, 20 * 4usize.pow(3) * 3);
        assert_eq!(plan.radius, 2.5);
        assert_eq!(
            plan.attributes,
            MeshVertexAttributes::new(false, false, false, false)
        );
    }

    #[test]
    fn mesh_builder_hex_for_initial_grid_uses_indexed_pentagon_fans() {
        let plan = MeshBuilder::build_hex(&DefaultHexMesher, 0, 2.0, 0.2);

        assert_eq!(plan.kind, MeshBuildKind::Hex);
        assert!(plan.indexed);
        assert_eq!(plan.vertex_capacity, tile_count(0) * 6);
        assert_eq!(plan.index_capacity, tile_count(0) * 4 * 3);
        assert_eq!(plan.vertices.len(), tile_count(0) * 5);
        assert_eq!(plan.indices.len(), tile_count(0) * 9);
        assert_eq!(plan.vertices[0].color, G3dColor::WHITE);
        assert_eq!(plan.vertices[0].emissive, None);
    }

    #[test]
    fn mesh_builder_hex_applies_height_intensity_color_and_emissive() {
        let plan = MeshBuilder::build_hex(&RaisedMesher, 0, 3.0, 0.5);

        assert!((plan.vertices[0].position.len() - 6.0).abs() < 0.0001);
        assert_eq!(plan.vertices[0].color, G3dColor::rgb(0.1, 0.2, 0.3));
        assert_eq!(
            plan.vertices[0].emissive,
            Some(G3dColor::rgb(0.9, 0.8, 0.7))
        );
        assert!(plan.attributes.emissive);
    }

    #[test]
    fn mesh_builder_planet_grid_emits_line_vertices_for_each_tile_edge() {
        let grid = PlanetGrid::create(0);
        let plan = MeshBuilder::build_planet_grid(&grid, G3dColor::rgb(0.4, 0.5, 0.6), 2.0);

        assert_eq!(plan.kind, MeshBuildKind::PlanetGrid);
        assert_eq!(plan.vertices.len(), grid.tiles.len() * 5 * 2);
        assert_eq!(plan.indices.len(), 0);
        assert_eq!(plan.vertices[0].color, G3dColor::rgb(0.4, 0.5, 0.6));
        assert!((plan.vertices[0].position.len() - 2.0).abs() < 0.0001);
    }

    #[test]
    fn mesh_builder_pack_normals_matches_java_bit_layout_for_positive_axis() {
        assert_eq!(MeshBuilder::pack_normals(1.0, 0.0, 0.0).to_bits(), 511);
        assert_eq!(
            MeshBuilder::pack_normals(0.0, 1.0, 0.0).to_bits(),
            511 << 10
        );
        assert_eq!(
            MeshBuilder::pack_normals(0.0, 0.0, 1.0).to_bits(),
            511 << 20
        );
    }
}
