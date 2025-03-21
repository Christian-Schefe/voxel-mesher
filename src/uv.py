from geometry import FaceType, RectQuad
from vec import Vec, vec
from PIL import Image, ImageDraw
import random


def lerp(a: float, b: float, t: float) -> float:
    return a + (b - a) * t


def unwrap_uv(quads: list[RectQuad], output_path: str):
    rects: list[tuple[int, tuple[int, int], FaceType]] = [(i, quad.tex_size, quad.face_type) for i, quad in enumerate(quads)]
    size = 1
    while True:
        uvs = try_pack_rects_scanning(size, rects)
        if uvs:
            break
        size *= 2
        print(size)

    for i, quad in enumerate(quads):
        (u, v) = uvs[i]
        quad.tex_pos = (u, v)
        factor = 1 / size

        quad.origin_uv = vec(u * factor, 1 - (v + quad.tex_size[1]) * factor)
        quad.dir1_uv = vec(0, quad.tex_size[1] * factor)
        quad.dir2_uv = vec(quad.tex_size[0] * factor, 0)

        print(quad.tex_pos, quad.tex_size, quad.origin_uv, quad.dir1_uv, quad.dir2_uv)

    generate_texture(size, quads, output_path)


normal_colors: dict[FaceType, Vec] = {
    "Front": vec(0, 1, 0),
    "Back": vec(0, 0, 1),
    "Left": vec(1, 1, 0),
    "Right": vec(1, 0, 0),
    "Top": vec(1, 1, 1),
    "Bottom": vec(1, 0.65, 0),
}

face_type_index: dict[FaceType, int] = {
    "Front": 0,
    "Back": 1,
    "Left": 2,
    "Right": 3,
    "Top": 4,
    "Bottom": 5,
}


def generate_texture(size: int, quads: list[RectQuad], output_path: str):
    image = Image.new("RGB", (size, size), (0, 0, 0))
    draw = ImageDraw.Draw(image)
    for quad in quads:
        fill_color = normal_colors[quad.face_type] * (random.random() * 0.5 + 0.5) * 255
        draw.rectangle(
            [
                (quad.tex_pos[0], quad.tex_pos[1]),
                (quad.tex_pos[0] + quad.tex_size[0] - 1, quad.tex_pos[1] + quad.tex_size[1] - 1),
            ],
            fill=(int(fill_color[0]), int(fill_color[1]), int(fill_color[2])),
        )
    image.save(output_path)


def try_pack_rects_scanning(size: int, rects: list[tuple[int, tuple[int, int], FaceType]]):
    sorted_rects = sorted(rects, key=lambda x: (x[1][1] * size + x[1][0]) * 6 - face_type_index[x[2]], reverse=True)

    grid = [[True for _ in range(size)] for _ in range(size)]

    uvs: dict[int, tuple[int, int]] = {}

    for index, (w, h), _ in sorted_rects:
        for j in range(size - h + 1):
            for i in range(size - w + 1):
                if all(grid[i + x][j + y] for x in range(w) for y in range(h)):
                    for x in range(w):
                        for y in range(h):
                            grid[i + x][j + y] = False
                    uvs[index] = (i, j)
                    break
            else:
                continue
            break
        else:
            return None

    return uvs
