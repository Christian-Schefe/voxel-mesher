use bevy_math::{IVec2, IVec3, Vec2, Vec3};

#[derive(Debug)]
pub struct Quad {
    pub vertices: [IVec3; 4],
    pub normal: Vec3,
    pub uvs: [Vec2; 4],
    pub tex_size: IVec2,
}

impl Quad {
    pub fn new(vertices: [IVec3; 4], tex_size: IVec2) -> Self {
        let normal = Self::get_normal(vertices);
        let uvs = [
            Vec2::new(0.0, 0.0),
            Vec2::new(0.0, 1.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(1.0, 0.0),
        ];
        Self {
            vertices,
            normal,
            uvs,
            tex_size,
        }
    }

    fn get_normal(vertices: [IVec3; 4]) -> Vec3 {
        let normal = IVec3::cross(vertices[1] - vertices[0], vertices[2] - vertices[0]).as_vec3();
        normal / normal.length()
    }
}
