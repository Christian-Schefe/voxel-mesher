use crate::geometry::Quad;

pub fn generate_obj_file(resolution: i32, quads: Vec<Quad>) -> String {
    let mut vert_index = 1;

    let mut vertex_lines = Vec::new();
    let mut normal_lines = Vec::new();
    let mut uv_lines = Vec::new();
    let mut face_lines = Vec::new();

    for quad in quads {
        let mut face_line = "f".to_string();
        for i in 0..4 {
            let vertex = quad.vertices[i].as_vec3() / resolution as f32;
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
