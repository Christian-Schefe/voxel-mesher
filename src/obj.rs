use std::path::PathBuf;

use bevy_math::Vec3;

use crate::geometry::Quad;

pub fn generate_obj_file(
    resolution: i32,
    origin: Vec3,
    quads: Vec<Quad>,
    mtl_file_name: &PathBuf,
) -> String {
    let mut vert_index = 1;

    let mut vertex_lines = Vec::new();
    let mut normal_lines = Vec::new();
    let mut uv_lines = Vec::new();
    let mut face_lines = Vec::new();

    for quad in quads {
        let mut face_line = "f".to_string();
        for i in [0, 3, 2, 1] {
            let vertex = (quad.vertices[i].as_vec3() - origin) / resolution as f32;
            let normal = quad.normal;
            let uvs = quad.uvs[i];

            let vertex_line = format!("v {} {} {}", -vertex.x, vertex.y, vertex.z);
            vertex_lines.push(vertex_line);

            let normal_line = format!("vn {} {} {}", -normal.x, normal.y, normal.z);
            normal_lines.push(normal_line);

            let uv_line = format!("vt {} {}", uvs.x, uvs.y);
            uv_lines.push(uv_line);

            face_line.push_str(&format!(" {}/{}/{}", vert_index, vert_index, vert_index));
            vert_index += 1;
        }
        face_lines.push(face_line);
    }

    let mut obj_lines = Vec::new();
    obj_lines.push(format!(
        "mtllib {}",
        mtl_file_name.file_name().unwrap().to_str().unwrap()
    ));
    obj_lines.push("usemtl material".to_string());
    obj_lines.push("o object".to_string());
    obj_lines.push("# Vertices".to_string());
    obj_lines.append(&mut vertex_lines);
    obj_lines.push("# Normals".to_string());
    obj_lines.append(&mut normal_lines);
    obj_lines.push("# UVs".to_string());
    obj_lines.append(&mut uv_lines);
    obj_lines.push("# Faces".to_string());
    obj_lines.append(&mut face_lines);

    obj_lines.join("\n")
}

pub fn generate_mtl_file(texture_file_name: &PathBuf) -> String {
    format!(
        r#"newmtl material
Ka 0.2 0.2 0.2
Kd 0.8 0.8 0.8
Ks 1.0 1.0 1.0
Ns 200
map_Kd {}
"#,
        texture_file_name.file_name().unwrap().to_str().unwrap()
    )
    .to_string()
}
