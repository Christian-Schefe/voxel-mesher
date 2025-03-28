use std::collections::HashSet;

use bevy_math::{IVec3, Vec2, Vec3};

use crate::app::{Cube, FileContent};
use anyhow::{Result, ensure};

#[derive(Debug)]
pub struct Quad {
    pub vertices: [IVec3; 4],
    pub normal: Vec3,
    pub uvs: [Vec2; 4],
    pub tex_size: (usize, usize),
}

impl Quad {
    pub fn new(vertices: [IVec3; 4], tex_size: (usize, usize)) -> Self {
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

pub fn convert_to_geometry(content: &FileContent) -> Result<Geometry> {
    let voxels: Vec<_> = content.obj.get_voxels().into_iter().collect();
    ensure!(!voxels.is_empty(), "No voxels found");

    let min = voxels.iter().fold(IVec3::MAX, |acc, &v| IVec3::min(acc, v));
    let max = voxels.iter().fold(IVec3::MIN, |acc, &v| IVec3::max(acc, v));
    let size = max - min + IVec3::splat(1);

    let mut grid = vec![vec![vec![false; size.z as usize]; size.y as usize]; size.x as usize];
    for voxel in voxels.iter() {
        let p = voxel - min;
        grid[p.x as usize][p.y as usize][p.z as usize] = true;
    }

    Ok(Geometry {
        resolution: content.resolution,
        min,
        size,
        voxels: grid,
    })
}

pub fn generate_quads(geometry: &Geometry) -> Vec<Quad> {
    let mut quads = Vec::new();
    let slices = vec![
        (
            geometry.size.x,
            IVec3::new(0, 0, 0),
            IVec3::new(1, 0, 0),
            IVec3::new(0, 1, 0),
            IVec3::new(0, 0, 1),
            IVec3::new(1, 0, 0),
        ),
        (
            geometry.size.x,
            IVec3::new(geometry.size.x - 1, 0, geometry.size.z - 1),
            IVec3::new(-1, 0, 0),
            IVec3::new(0, 1, 0),
            IVec3::new(0, 0, -1),
            IVec3::new(0, 0, 1),
        ),
        (
            geometry.size.y,
            IVec3::new(0, 0, 0),
            IVec3::new(0, 1, 0),
            IVec3::new(0, 0, 1),
            IVec3::new(1, 0, 0),
            IVec3::new(0, 1, 0),
        ),
        (
            geometry.size.y,
            IVec3::new(0, geometry.size.y - 1, geometry.size.z - 1),
            IVec3::new(0, -1, 0),
            IVec3::new(0, 0, -1),
            IVec3::new(1, 0, 0),
            IVec3::new(0, 0, 1),
        ),
        (
            geometry.size.z,
            IVec3::new(geometry.size.x - 1, 0, 0),
            IVec3::new(0, 0, 1),
            IVec3::new(0, 1, 0),
            IVec3::new(-1, 0, 0),
            IVec3::new(1, 0, 1),
        ),
        (
            geometry.size.z,
            IVec3::new(0, 0, geometry.size.z - 1),
            IVec3::new(0, 0, -1),
            IVec3::new(0, 1, 0),
            IVec3::new(1, 0, 0),
            IVec3::new(0, 0, 0),
        ),
    ];
    for (steps, origin, normal, dir1, dir2, offset) in slices {
        for n in 0..steps {
            let pos = origin + n * normal;
            println!("{:?} {:?} {:?} {:?}", pos, dir1, dir2, normal);
            let quads_slice = generate_quads_slice(&geometry, pos, dir1, dir2, normal, offset);
            for quad in quads_slice.iter() {
                println!("q {:?}", quad.vertices);
            }
            quads.extend(quads_slice);
        }
    }
    quads
}

fn generate_quads_slice(
    geometry: &Geometry,
    origin: IVec3,
    dir1: IVec3,
    dir2: IVec3,
    normal: IVec3,
    offset: IVec3,
) -> Vec<Quad> {
    let mut mask = get_slice_mask(geometry, origin, dir1, dir2, normal);
    let mut quads = Vec::new();

    for i in 0..mask.len() {
        let len = mask[i].len();
        for j in 0..len {
            if mask[i][j] {
                let mut s2 = 1;
                while j + s2 < len && mask[i][j + s2] {
                    mask[i][j + s2] = false;
                    s2 += 1;
                }
                let mut s1 = 1;
                let mut done = false;
                while !done {
                    for k in 0..s2 {
                        if i + s1 >= mask.len() || !mask[i + s1][j + k] {
                            done = true;
                            break;
                        }
                    }
                    if !done {
                        for k in 0..s2 {
                            mask[i + s1][j + k] = false;
                        }
                        s1 += 1;
                    }
                }
                println!("{} {}, {} {}", i, j, s2, s1);

                let p1 = origin + i as i32 * dir1 + j as i32 * dir2 + geometry.min + offset;
                let p2 = p1 + dir1 * s1 as i32;
                let p3 = p2 + dir2 * s2 as i32;
                let p4 = p1 + dir2 * s2 as i32;
                quads.push(Quad::new([p1, p2, p3, p4], (s2, s1)));
            }
        }
    }

    quads
}

fn get_slice_mask(
    geometry: &Geometry,
    origin: IVec3,
    dir1: IVec3,
    dir2: IVec3,
    normal: IVec3,
) -> Vec<Vec<bool>> {
    let size1 = geometry.size.dot(IVec3::abs(dir1));
    let size2 = geometry.size.dot(IVec3::abs(dir2));

    let mut mask = vec![vec![false; size2 as usize]; size1 as usize];

    let is_out_of_bounds = |pos: IVec3| -> bool {
        pos.x < 0
            || pos.y < 0
            || pos.z < 0
            || pos.x >= geometry.size.x
            || pos.y >= geometry.size.y
            || pos.z >= geometry.size.z
    };

    for i in 0..size1 {
        for j in 0..size2 {
            let pos = origin + i * dir1 + j * dir2;
            let other_pos = pos + normal;
            let has_block = geometry.voxels[pos.x as usize][pos.y as usize][pos.z as usize];
            let other_has_no_block = is_out_of_bounds(other_pos)
                || !geometry.voxels[other_pos.x as usize][other_pos.y as usize]
                    [other_pos.z as usize];
            let has_face = has_block && other_has_no_block;
            mask[i as usize][j as usize] = has_face;
        }
    }

    mask
}

#[derive(Debug)]
pub struct Geometry {
    pub resolution: i32,
    pub min: IVec3,
    pub size: IVec3,
    pub voxels: Vec<Vec<Vec<bool>>>,
}

#[derive(Debug)]
pub enum GeometryObject {
    Cube(Cube),
    Intersection(Box<GeometryObject>, Box<GeometryObject>),
    Union(Box<GeometryObject>, Box<GeometryObject>),
    Minus(Box<GeometryObject>, Box<GeometryObject>),
    SymmetricDifference(Box<GeometryObject>, Box<GeometryObject>),
    Wireframe(Box<GeometryObject>),
}

impl GeometryObject {
    pub fn get_voxels(&self) -> HashSet<IVec3> {
        match self {
            GeometryObject::Cube(cube) => {
                let mut voxels: HashSet<_> = HashSet::new();
                for x in 0..cube.size.x {
                    for y in 0..cube.size.y {
                        for z in 0..cube.size.z {
                            voxels.insert(cube.corner + IVec3::new(x, y, z));
                        }
                    }
                }
                voxels
            }
            GeometryObject::Intersection(geometry_object, geometry_object1) => {
                let voxels = geometry_object.get_voxels();
                let voxels1 = geometry_object1.get_voxels();
                voxels.intersection(&voxels1).cloned().collect()
            }
            GeometryObject::Union(geometry_object, geometry_object1) => {
                let voxels = geometry_object.get_voxels();
                let voxels1 = geometry_object1.get_voxels();
                voxels.union(&voxels1).cloned().collect()
            }
            GeometryObject::Minus(geometry_object, geometry_object1) => {
                let voxels = geometry_object.get_voxels();
                let voxels1 = geometry_object1.get_voxels();
                voxels.difference(&voxels1).cloned().collect()
            }
            GeometryObject::SymmetricDifference(geometry_object, geometry_object1) => {
                let voxels = geometry_object.get_voxels();
                let voxels1 = geometry_object1.get_voxels();
                voxels.symmetric_difference(&voxels1).cloned().collect()
            }
            GeometryObject::Wireframe(geometry_object) => {
                let voxels = geometry_object.get_voxels();
                let mut wireframe_voxels = HashSet::new();
                for voxel in voxels.iter() {
                    let mut empty_count = 0;
                    for x in -1..=1i32 {
                        for y in -1..=1i32 {
                            for z in -1..=1i32 {
                                let neighbor = *voxel + IVec3::new(x, y, z);
                                if !voxels.contains(&neighbor) {
                                    empty_count += 1;
                                }
                            }
                        }
                    }
                    if empty_count > 9 || empty_count == 3 {
                        wireframe_voxels.insert(*voxel);
                    }
                }
                wireframe_voxels
            }
        }
    }
}
