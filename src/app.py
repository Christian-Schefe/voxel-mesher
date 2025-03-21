from geometry import Cube, RectQuad
from uv import unwrap_uv
from vec import Vec, vec
import pathlib


def parse_input(input_file: str):
    lines = [line for line in [line.strip() for line in input_file.split("\n")] if line]
    header = lines.pop(0).split(";")
    resolution = int(header[0])
    pivot = vec(*[float(x) for x in header[1].split(" ") if x])

    cubes = [[tuple([float(y) for y in x.strip().split(" ") if y]) for x in line.split(";")] for line in lines]
    cubes = [Cube(vec(pos[0], pos[1], pos[2]), vec(size[0], size[1], size[2])) for pos, size in cubes]

    return (cubes, resolution, pivot)


def app(input_file: str, file_name: str, output_path: str):
    (cubes, resolution, pivot) = parse_input(input_file)
    cube_quads = [quad for cube in cubes for quad in cube.get_quads()]

    filtered_quads = []
    for i in range(len(cube_quads)):
        quad = cube_quads[i]
        for j in range(len(cube_quads)):
            if i != j and cube_quads[j].contains_quad(quad):
                print(f"Quad {i} is contained by quad {j}")
                break
        else:
            filtered_quads.append(quad)

    out_folder = pathlib.Path(output_path)
    out_folder.mkdir(parents=True, exist_ok=True)

    model_path = pathlib.Path(output_path).joinpath(file_name).with_suffix(".obj")
    texture_path = model_path.with_suffix(".png")
    unwrap_uv(filtered_quads, str(texture_path))

    write_quads_to_obj(filtered_quads, file_name, resolution, pivot, str(model_path))


def write_quads_to_obj(quads: list[RectQuad], file_name: str, resolution, pivot: Vec, output_path: str):
    vertices_lines = []
    normals_lines = []
    uvs_lines = []
    faces_lines = []

    vert_index = 0

    for quad in quads:
        face_line = "f"
        vert_normal = quad.get_normal()
        vert_normal[0] *= -1
        for vert_pos, vert_uv in quad.get_vertices(reverse_direction=True):
            vert_index += 1
            pos = (vert_pos - pivot) / resolution
            pos[0] *= -1
            vertices_lines.append(f"v {pos[0]} {pos[1]} {pos[2]}")
            normals_lines.append(f"vn {vert_normal[0]} {vert_normal[1]} {vert_normal[2]}")
            uvs_lines.append(f"vt {vert_uv[0]} {vert_uv[1]}")
            face_line += f" {vert_index}/{vert_index}/{vert_index}"
        faces_lines.append(face_line)

    with open(output_path, "w") as f:
        f.write("g " + file_name + "\n")
        f.write("\n".join(vertices_lines) + "\n")
        f.write("\n".join(normals_lines) + "\n")
        f.write("\n".join(uvs_lines) + "\n")
        f.write("\n".join(faces_lines) + "\n")
