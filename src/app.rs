use anyhow::{Result, anyhow, ensure};
use bevy_math::{IVec3, Vec3};
use std::path::PathBuf;

use crate::{
    geometry::{GeometryObject, convert_to_geometry, generate_quads},
    obj::{generate_mtl_file, generate_obj_file},
    texture::{apply_uv_to_quads, create_texture_file, pack_quad_texture, sort_quads},
};

pub fn app(file: &PathBuf, output: &PathBuf) -> Result<()> {
    let content_str = std::fs::read_to_string(file)?;
    let content = parse_content(content_str)?;
    let geometry = convert_to_geometry(&content)?;

    let mut quads = generate_quads(&geometry);
    sort_quads(&mut quads);

    let (tex_quads, size) = pack_quad_texture(&quads);
    apply_uv_to_quads(&mut quads, &tex_quads, size);

    let texture_path = output.join(file.file_stem().unwrap()).with_extension("png");
    let mtl_path = output.join(file.file_stem().unwrap()).with_extension("mtl");
    let obj_path = output.join(file.file_stem().unwrap()).with_extension("obj");

    let texture = create_texture_file(&tex_quads, size);
    texture.save(&texture_path)?;

    let obj_file = generate_obj_file(geometry.resolution, content.origin, quads, &mtl_path);
    std::fs::write(obj_path, obj_file)?;

    let mtl_file = generate_mtl_file(&texture_path);
    std::fs::write(mtl_path, mtl_file)?;
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

    let origin = parse_vec3(header_parts[1].to_string())?;

    let other_lines = lines.iter().skip(1);
    let other_str = other_lines
        .map(|s| s.to_string())
        .collect::<Vec<String>>()
        .join("\n");
    let mut index = 0;
    let obj = parse_geometry(&other_str.chars().collect::<Vec<_>>(), &mut index)?;
    if index != other_str.len() {
        return Err(anyhow!("Found extra characters at the end of the file"));
    }
    Ok(FileContent {
        resolution,
        origin,
        obj,
    })
}

fn parse_geometry(content: &[char], index: &mut usize) -> Result<GeometryObject> {
    while content.get(*index).is_some_and(|c| c.is_whitespace()) {
        *index += 1;
    }
    let c = content
        .get(*index)
        .ok_or(anyhow!("Unexpected end of file"))?;
    *index += 1;
    let res = match c {
        '(' => {
            let mut i = *index;
            while content.get(i).is_some_and(|c| *c != ')') {
                i += 1;
            }
            if content.get(i).is_none() {
                return Err(anyhow!("Expected ')'"));
            }
            let str = content[*index..i].iter().collect::<String>();
            *index = i + 1;
            let cube = parse_cube(&str)?;
            Ok(GeometryObject::Cube(cube))
        }
        '&' => {
            let left = Box::new(parse_geometry(content, index)?);
            let right = Box::new(parse_geometry(content, index)?);
            Ok(GeometryObject::Intersection(left, right))
        }
        '+' => {
            let left = Box::new(parse_geometry(content, index)?);
            let right = Box::new(parse_geometry(content, index)?);
            Ok(GeometryObject::Union(left, right))
        }
        '-' => {
            let left = Box::new(parse_geometry(content, index)?);
            let right = Box::new(parse_geometry(content, index)?);
            Ok(GeometryObject::Minus(left, right))
        }
        '/' => {
            let left = Box::new(parse_geometry(content, index)?);
            let right = Box::new(parse_geometry(content, index)?);
            Ok(GeometryObject::SymmetricDifference(left, right))
        }
        'w' => {
            let obj = Box::new(parse_geometry(content, index)?);
            Ok(GeometryObject::Wireframe(obj))
        }
        _ => Err(anyhow!("Unexpected character: {}", c)),
    }?;
    while content.get(*index).is_some_and(|c| c.is_whitespace()) {
        *index += 1;
    }
    Ok(res)
}

fn parse_cube(line: &str) -> Result<Cube> {
    let parts = line
        .split(";")
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();
    ensure!(parts.len() == 2, "Expected 2 parts, got {}", parts.len());
    let corner = parse_ivec3(parts[0].to_string())?;
    let size = parse_ivec3(parts[1].to_string())?;
    ensure!(size.x > 0 && size.y > 0 && size.z > 0, "Invalid size");
    Ok(Cube { corner, size })
}

fn parse_ivec3(str: String) -> Result<IVec3> {
    let parts = str
        .split_whitespace()
        .map(|s| s.trim().parse::<i32>())
        .collect::<Result<Vec<_>, _>>()?;
    ensure!(parts.len() == 3, "Expected 3 parts, got {}", parts.len());
    Ok(IVec3::new(parts[0], parts[1], parts[2]))
}

fn parse_vec3(str: String) -> Result<Vec3> {
    let parts = str
        .split_whitespace()
        .map(|s| s.trim().parse::<f32>())
        .collect::<Result<Vec<_>, _>>()?;
    ensure!(parts.len() == 3, "Expected 3 parts, got {}", parts.len());
    Ok(Vec3::new(parts[0], parts[1], parts[2]))
}
#[derive(Debug)]
pub struct Cube {
    pub corner: IVec3,
    pub size: IVec3,
}

#[derive(Debug)]
pub struct FileContent {
    pub resolution: i32,
    pub origin: Vec3,
    pub obj: GeometryObject,
}
