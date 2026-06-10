//! Planet hex grid topology mirroring upstream `mindustry.graphics.g3d.PlanetGrid`.

use super::G3dVec3;

#[derive(Debug, Clone, PartialEq)]
pub struct PlanetGrid {
    pub size: i32,
    pub tiles: Vec<Ptile>,
    pub corners: Vec<Corner>,
    pub edges: Vec<Edge>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ptile {
    pub id: usize,
    pub edge_count: usize,
    pub tiles: Vec<Option<usize>>,
    pub corners: Vec<Option<usize>>,
    pub edges: Vec<Option<usize>>,
    pub v: G3dVec3,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Corner {
    pub id: usize,
    pub tiles: [Option<usize>; 3],
    pub corners: [Option<usize>; 3],
    pub edges: [Option<usize>; 3],
    pub v: G3dVec3,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Edge {
    pub id: usize,
    pub tiles: [Option<usize>; 2],
    pub corners: [Option<usize>; 2],
}

const X: f32 = -0.525731112119133606;
const Z: f32 = -0.850650808352039932;

const I_TILES: [G3dVec3; 12] = [
    G3dVec3::new(-X, 0.0, Z),
    G3dVec3::new(X, 0.0, Z),
    G3dVec3::new(-X, 0.0, -Z),
    G3dVec3::new(X, 0.0, -Z),
    G3dVec3::new(0.0, Z, X),
    G3dVec3::new(0.0, Z, -X),
    G3dVec3::new(0.0, -Z, X),
    G3dVec3::new(0.0, -Z, -X),
    G3dVec3::new(Z, X, 0.0),
    G3dVec3::new(-Z, X, 0.0),
    G3dVec3::new(Z, -X, 0.0),
    G3dVec3::new(-Z, -X, 0.0),
];

const I_TILES_P: [[usize; 5]; 12] = [
    [9, 4, 1, 6, 11],
    [4, 8, 10, 6, 0],
    [11, 7, 3, 5, 9],
    [2, 7, 10, 8, 5],
    [9, 5, 8, 1, 0],
    [2, 3, 8, 4, 9],
    [0, 1, 10, 7, 11],
    [11, 6, 10, 3, 2],
    [5, 3, 10, 1, 4],
    [2, 5, 4, 0, 11],
    [3, 7, 6, 1, 8],
    [7, 2, 9, 0, 6],
];

impl PlanetGrid {
    pub fn create(size: i32) -> Self {
        if size == 0 {
            Self::initial_grid()
        } else {
            Self::subdivided_grid(Self::create(size - 1))
        }
    }

    pub fn new(size: i32) -> Self {
        let mut tiles = Vec::with_capacity(tile_count(size));
        for id in 0..tile_count(size) {
            let edge_count = if id < 12 { 5 } else { 6 };
            tiles.push(Ptile::new(id, edge_count));
        }

        let mut corners = Vec::with_capacity(corner_count(size));
        for id in 0..corner_count(size) {
            corners.push(Corner::new(id));
        }

        let mut edges = Vec::with_capacity(edge_count(size));
        for id in 0..edge_count(size) {
            edges.push(Edge::new(id));
        }

        Self {
            size,
            tiles,
            corners,
            edges,
        }
    }

    pub fn initial_grid() -> Self {
        let mut grid = Self::new(0);

        for tile in &mut grid.tiles {
            tile.v = I_TILES[tile.id];
            for k in 0..5 {
                tile.tiles[k] = Some(I_TILES_P[tile.id][k]);
            }
        }

        for i in 0..5 {
            add_corner(i, &mut grid, 0, I_TILES_P[0][(i + 4) % 5], I_TILES_P[0][i]);
        }
        for i in 0..5 {
            add_corner(
                i + 5,
                &mut grid,
                3,
                I_TILES_P[3][(i + 4) % 5],
                I_TILES_P[3][i],
            );
        }
        add_corner(10, &mut grid, 10, 1, 8);
        add_corner(11, &mut grid, 1, 10, 6);
        add_corner(12, &mut grid, 6, 10, 7);
        add_corner(13, &mut grid, 6, 7, 11);
        add_corner(14, &mut grid, 11, 7, 2);
        add_corner(15, &mut grid, 11, 2, 9);
        add_corner(16, &mut grid, 9, 2, 5);
        add_corner(17, &mut grid, 9, 5, 4);
        add_corner(18, &mut grid, 4, 5, 8);
        add_corner(19, &mut grid, 4, 8, 1);

        connect_corner_neighbors(&mut grid);
        add_all_edges(&mut grid);
        grid
    }

    pub fn subdivided_grid(prev: Self) -> Self {
        let mut grid = Self::new(prev.size + 1);
        let prev_tiles = prev.tiles.len();
        let prev_corners = prev.corners.len();

        for i in 0..prev_tiles {
            grid.tiles[i].v = prev.tiles[i].v;
            for k in 0..grid.tiles[i].edge_count {
                grid.tiles[i].tiles[k] = Some(prev.tiles[i].corners[k].unwrap() + prev_tiles);
            }
        }

        for i in 0..prev_corners {
            let tile_id = i + prev_tiles;
            grid.tiles[tile_id].v = prev.corners[i].v;
            for k in 0..3 {
                grid.tiles[tile_id].tiles[2 * k] =
                    Some(prev.corners[i].corners[k].unwrap() + prev_tiles);
                grid.tiles[tile_id].tiles[2 * k + 1] = Some(prev.corners[i].tiles[k].unwrap());
            }
        }

        let mut next_corner = 0;
        for n in &prev.tiles {
            let tile_id = n.id;
            for k in 0..grid.tiles[tile_id].edge_count {
                let previous = grid.tiles[tile_id].tiles
                    [(k + grid.tiles[tile_id].edge_count - 1) % grid.tiles[tile_id].edge_count]
                    .unwrap();
                let next = grid.tiles[tile_id].tiles[k].unwrap();
                add_corner(next_corner, &mut grid, tile_id, previous, next);
                next_corner += 1;
            }
        }

        connect_corner_neighbors(&mut grid);
        add_all_edges(&mut grid);
        grid
    }
}

impl Ptile {
    pub fn new(id: usize, edge_count: usize) -> Self {
        Self {
            id,
            edge_count,
            tiles: vec![None; edge_count],
            corners: vec![None; edge_count],
            edges: vec![None; edge_count],
            v: G3dVec3::default(),
        }
    }
}

impl Corner {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            tiles: [None; 3],
            corners: [None; 3],
            edges: [None; 3],
            v: G3dVec3::default(),
        }
    }
}

impl Edge {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            tiles: [None; 2],
            corners: [None; 2],
        }
    }
}

pub fn tile_count(size: i32) -> usize {
    10 * 3usize.pow(size as u32) + 2
}

pub fn corner_count(size: i32) -> usize {
    20 * 3usize.pow(size as u32)
}

pub fn edge_count(size: i32) -> usize {
    30 * 3usize.pow(size as u32)
}

fn add_corner(id: usize, grid: &mut PlanetGrid, t1: usize, t2: usize, t3: usize) {
    let tiles = [t1, t2, t3];
    let v = grid.tiles[t1]
        .v
        .add(grid.tiles[t2].v)
        .add(grid.tiles[t3].v)
        .nor();
    grid.corners[id].v = v;
    for i in 0..3 {
        let tile = tiles[i];
        let neighbor = tiles[(i + 2) % 3];
        let position = pos_tile_tile(grid, tile, neighbor).unwrap();
        grid.tiles[tile].corners[position] = Some(id);
        grid.corners[id].tiles[i] = Some(tile);
    }
}

fn connect_corner_neighbors(grid: &mut PlanetGrid) {
    for corner_id in 0..grid.corners.len() {
        for k in 0..3 {
            let tile = grid.corners[corner_id].tiles[k].unwrap();
            let position = pos_tile_corner(grid, tile, corner_id).unwrap();
            let next =
                grid.tiles[tile].corners[(position + 1) % grid.tiles[tile].edge_count].unwrap();
            grid.corners[corner_id].corners[k] = Some(next);
        }
    }
}

fn add_all_edges(grid: &mut PlanetGrid) {
    let mut next_edge = 0;
    for tile_id in 0..grid.tiles.len() {
        for k in 0..grid.tiles[tile_id].edge_count {
            if grid.tiles[tile_id].edges[k].is_none() {
                let other = grid.tiles[tile_id].tiles[k].unwrap();
                add_edge(next_edge, grid, tile_id, other);
                next_edge += 1;
            }
        }
    }
}

fn add_edge(id: usize, grid: &mut PlanetGrid, t1: usize, t2: usize) {
    let tiles = [t1, t2];
    let position = pos_tile_tile(grid, t1, t2).unwrap();
    let corners = [
        grid.tiles[t1].corners[position].unwrap(),
        grid.tiles[t1].corners[(position + 1) % grid.tiles[t1].edge_count].unwrap(),
    ];

    for i in 0..2 {
        let tile = tiles[i];
        let other_tile = tiles[(i + 1) % 2];
        let tile_position = pos_tile_tile(grid, tile, other_tile).unwrap();
        grid.tiles[tile].edges[tile_position] = Some(id);
        grid.edges[id].tiles[i] = Some(tile);

        let corner = corners[i];
        let other_corner = corners[(i + 1) % 2];
        let corner_position = pos_corner_corner(grid, corner, other_corner).unwrap();
        grid.corners[corner].edges[corner_position] = Some(id);
        grid.edges[id].corners[i] = Some(corner);
    }
}

fn pos_tile_tile(grid: &PlanetGrid, tile: usize, neighbor: usize) -> Option<usize> {
    grid.tiles[tile]
        .tiles
        .iter()
        .position(|candidate| *candidate == Some(neighbor))
}

fn pos_tile_corner(grid: &PlanetGrid, tile: usize, corner: usize) -> Option<usize> {
    grid.tiles[tile]
        .corners
        .iter()
        .position(|candidate| *candidate == Some(corner))
}

fn pos_corner_corner(grid: &PlanetGrid, corner: usize, neighbor: usize) -> Option<usize> {
    grid.corners[corner]
        .corners
        .iter()
        .position(|candidate| *candidate == Some(neighbor))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn planet_grid_counts_match_java_formulas() {
        for size in 0..4 {
            let grid = PlanetGrid::create(size);

            assert_eq!(grid.tiles.len(), 10 * 3usize.pow(size as u32) + 2);
            assert_eq!(grid.corners.len(), 20 * 3usize.pow(size as u32));
            assert_eq!(grid.edges.len(), 30 * 3usize.pow(size as u32));
        }
    }

    #[test]
    fn initial_grid_uses_twelve_pentagons_and_connected_edges() {
        let grid = PlanetGrid::create(0);

        assert_eq!(grid.tiles.len(), 12);
        assert!(grid.tiles.iter().all(|tile| tile.edge_count == 5));
        assert!(grid
            .tiles
            .iter()
            .all(|tile| tile.tiles.iter().all(Option::is_some)));
        assert!(grid
            .tiles
            .iter()
            .all(|tile| tile.corners.iter().all(Option::is_some)));
        assert!(grid
            .tiles
            .iter()
            .all(|tile| tile.edges.iter().all(Option::is_some)));
    }

    #[test]
    fn subdivided_grid_keeps_original_tiles_as_pentagons_and_new_tiles_as_hexagons() {
        let grid = PlanetGrid::create(1);

        assert_eq!(grid.tiles.len(), 32);
        assert!(grid.tiles.iter().take(12).all(|tile| tile.edge_count == 5));
        assert!(grid.tiles.iter().skip(12).all(|tile| tile.edge_count == 6));
    }
}
