use anyhow::{Result, ensure};
use bevy_math::{IVec2, IVec3};
use std::path::PathBuf;

use crate::{geometry::Quad, obj::generate_obj_file};

pub fn app(file: &PathBuf, output: &PathBuf) -> Result<()> {
    let content_str = std::fs::read_to_string(file)?;
    let content = parse_content(content_str)?;
    let geometry = convert_to_geometry(content)?;
    println!("{:?}", geometry);
    let quads = generate_quads(&geometry);
    for quad in quads.iter() {
        println!("{:?}", quad);
    }

    let obj_file = generate_obj_file(geometry.resolution, quads);
    let output_file = output.join(file.file_stem().unwrap()).with_extension("obj");
    std::fs::write(output_file, obj_file)?;
    Ok(())
}

fn parse_content(content: String) -> Result<FileContent> {
    let lines = content
        .lines()
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>();
    let header_line = lines[0];
    let header_parts = header_line
        .split(";")
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();
    ensure!(
        header_parts.len() == 2,
        "Expected 2 parts, got {}",
        header_parts.len()
    );
    let resolution = header_parts[0].parse::<i32>()?;
    ensure!(resolution > 0, "Invalid resolution");

    let origin = parse_ivec3(header_parts[1].to_string())?;
    let mut cubes = Vec::new();
    for line in lines.iter().skip(1) {
        let parts = line
            .split(";")
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();
        ensure!(parts.len() == 2, "Expected 2 parts, got {}", parts.len());
        let corner = parse_ivec3(parts[0].to_string())?;
        let size = parse_ivec3(parts[1].to_string())?;
        ensure!(size.x > 0 && size.y > 0 && size.z > 0, "Invalid size");
        let cube = Cube { corner, size };
        cubes.push(cube);
    }
    Ok(FileContent {
        resolution,
        origin,
        cubes,
    })
}

fn parse_ivec3(str: String) -> Result<IVec3> {
    let parts = str
        .split_whitespace()
        .map(|s| s.trim().parse::<i32>())
        .collect::<Result<Vec<_>, _>>()?;
    ensure!(parts.len() == 3, "Expected 3 parts, got {}", parts.len());
    Ok(IVec3::new(parts[0], parts[1], parts[2]))
}

fn convert_to_geometry(content: FileContent) -> Result<Geometry> {
    let mut voxels = Vec::new();
    let mut min = IVec3::splat(i32::MAX);
    let mut max = IVec3::splat(i32::MIN);
    for cube in content.cubes {
        let corner = cube.corner;
        let size = cube.size;
        for x in corner.x..corner.x + size.x {
            for y in corner.y..corner.y + size.y {
                for z in corner.z..corner.z + size.z {
                    voxels.push(IVec3::new(x, y, z) - content.origin);
                }
            }
        }
        min = min.min(corner);
        max = max.max(corner + size);
    }
    ensure!(!voxels.is_empty(), "No voxels found");
    let size = max - min;
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

fn generate_quads(geometry: &Geometry) -> Vec<Quad> {
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
            IVec3::new(
                geometry.size.x - 1,
                geometry.size.y - 1,
                geometry.size.z - 1,
            ),
            IVec3::new(0, -1, 0),
            IVec3::new(0, 0, -1),
            IVec3::new(-1, 0, 0),
            IVec3::new(1, 0, 1),
        ),
        (
            geometry.size.z,
            IVec3::new(geometry.size.x - 1, 0, 0),
            IVec3::new(0, 0, 1),
            IVec3::new(-1, 0, 0),
            IVec3::new(0, 1, 0),
            IVec3::new(1, 0, 1),
        ),
        (
            geometry.size.z,
            IVec3::new(0, 0, geometry.size.z - 1),
            IVec3::new(0, 0, -1),
            IVec3::new(1, 0, 0),
            IVec3::new(0, 1, 0),
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
                let mut s1 = 1;
                while j + s1 < len && mask[i][j + s1] {
                    mask[i][j + s1] = false;
                    s1 += 1;
                }
                let mut s2 = 1;
                let mut done = false;
                while !done {
                    for k in 0..s1 {
                        if i + s2 >= mask.len() || !mask[i + s2][j + k] {
                            done = true;
                            break;
                        }
                    }
                    if !done {
                        for k in 0..s1 {
                            mask[i + s2][j + k] = false;
                        }
                        s2 += 1;
                    }
                }
                println!("{} {}, {} {}", i, j, s1, s2);

                let p1 = origin + i as i32 * dir1 + j as i32 * dir2 + geometry.min + offset;
                let p2 = p1 + dir1 * s2 as i32;
                let p3 = p2 + dir2 * s1 as i32;
                let p4 = p1 + dir2 * s1 as i32;
                quads.push(Quad::new(
                    [p1, p2, p3, p4],
                    IVec2::new(s2 as i32, s1 as i32),
                ));
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
struct Geometry {
    resolution: i32,
    min: IVec3,
    size: IVec3,
    voxels: Vec<Vec<Vec<bool>>>,
}

#[derive(Debug)]
struct FileContent {
    resolution: i32,
    origin: IVec3,
    cubes: Vec<Cube>,
}

#[derive(Debug)]
struct Cube {
    corner: IVec3,
    size: IVec3,
}
