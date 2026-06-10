//! Hex mesh sampler contract mirroring upstream `mindustry.graphics.g3d.HexMesher`.

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct G3dVec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl G3dVec3 {
    pub const Y: Self = Self::new(0.0, 1.0, 0.0);

    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }

    pub fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }

    pub fn scl(self, scale: f32) -> Self {
        Self::new(self.x * scale, self.y * scale, self.z * scale)
    }

    pub fn len(self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn nor(self) -> Self {
        self.scl(1.0 / self.len())
    }

    pub fn crs(self, other: Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    pub fn rotate_y_degrees(self, degrees: f32) -> Self {
        let radians = degrees.to_radians();
        let cos = radians.cos();
        let sin = radians.sin();
        Self::new(
            self.x * cos + self.z * sin,
            self.y,
            -self.x * sin + self.z * cos,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct G3dColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl G3dColor {
    pub const CLEAR: Self = Self::new(0.0, 0.0, 0.0, 0.0);
    pub const WHITE: Self = Self::new(1.0, 1.0, 1.0, 1.0);

    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self::new(r, g, b, 1.0)
    }

    pub fn mul(self, scale: f32) -> Self {
        Self::new(
            self.r * scale,
            self.g * scale,
            self.b * scale,
            self.a * scale,
        )
    }
}

impl Default for G3dColor {
    fn default() -> Self {
        Self::CLEAR
    }
}

/** Defines color and height for a planet mesh. */
pub trait HexMesher {
    fn get_height(&self, _position: G3dVec3) -> f32 {
        0.0
    }

    fn get_color(&self, _position: G3dVec3, _out: &mut G3dColor) {}

    fn get_emissive_color(&self, _position: G3dVec3, _out: &mut G3dColor) {}

    fn is_emissive(&self) -> bool {
        false
    }

    fn skip(&self, _position: G3dVec3) -> bool {
        false
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct DefaultHexMesher;

impl HexMesher for DefaultHexMesher {}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FixedColorHexMesher {
    pub color: G3dColor,
}

impl FixedColorHexMesher {
    pub const fn new(color: G3dColor) -> Self {
        Self { color }
    }
}

impl HexMesher for FixedColorHexMesher {
    fn get_color(&self, _position: G3dVec3, out: &mut G3dColor) {
        *out = self.color;
    }
}

pub fn simplex_noise3d(
    seed: i32,
    octaves: impl Into<f64>,
    persistence: impl Into<f64>,
    scale: impl Into<f64>,
    x: f32,
    y: f32,
    z: f32,
) -> f32 {
    let octaves = octaves.into() as i32;
    let persistence = persistence.into();
    let scale = scale.into();
    let mut total = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0 / scale;
    let mut max = 0.0;

    for octave in 0..octaves {
        let value = raw_noise(
            seed + octave,
            x as f64 * frequency,
            y as f64 * frequency,
            z as f64 * frequency,
        );
        total += value * amplitude;
        max += amplitude;
        amplitude *= persistence;
        frequency *= 2.0;
    }

    (total / max) as f32
}

fn raw_noise(seed: i32, x: f64, y: f64, z: f64) -> f64 {
    let value = (x * 12.9898 + y * 78.233 + z * 37.719 + seed as f64 * 19.19).sin() * 43758.5453;
    value - value.floor()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_mesher_defaults_match_java_interface_defaults() {
        let mesher = DefaultHexMesher;
        let position = G3dVec3::new(1.0, 2.0, 3.0);
        let mut color = G3dColor::rgb(0.2, 0.3, 0.4);
        let mut emissive = G3dColor::WHITE;

        assert_eq!(mesher.get_height(position), 0.0);
        mesher.get_color(position, &mut color);
        mesher.get_emissive_color(position, &mut emissive);

        assert_eq!(color, G3dColor::rgb(0.2, 0.3, 0.4));
        assert_eq!(emissive, G3dColor::WHITE);
        assert!(!mesher.is_emissive());
        assert!(!mesher.skip(position));
    }

    #[test]
    fn fixed_color_mesher_overwrites_color_like_anonymous_java_mesher() {
        let mesher = FixedColorHexMesher::new(G3dColor::rgb(0.7, 0.2, 0.1));
        let mut color = G3dColor::WHITE;

        mesher.get_color(G3dVec3::Y, &mut color);

        assert_eq!(color, G3dColor::rgb(0.7, 0.2, 0.1));
    }

    #[test]
    fn g3d_vec3_rotate_y_uses_degrees_like_arc_vec3_rotate() {
        let rotated = G3dVec3::new(1.0, 0.0, 0.0).rotate_y_degrees(90.0);

        assert!(rotated.x.abs() < 0.0001);
        assert!((rotated.z + 1.0).abs() < 0.0001);
    }
}
