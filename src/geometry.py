from dataclasses import dataclass, field
import math
from typing import Literal, Self
import numpy as np
from vec import Vec, normalize, vec

type Face = tuple[int, int, int]
type FaceType = Literal["Front", "Back", "Top", "Bottom", "Left", "Right"]


@dataclass
class RectQuad:
    origin: Vec
    dir1: Vec
    dir2: Vec
    tex_size: tuple[int, int]
    face_type: FaceType
    origin_uv: Vec = field(default_factory=lambda: vec(0, 0))
    dir1_uv: Vec = field(default_factory=lambda: vec(0, 0))
    dir2_uv: Vec = field(default_factory=lambda: vec(0, 0))
    tex_pos: tuple[int, int] = (0, 0)

    def get_normal(self) -> Vec:
        return normalize(np.cross(self.dir1, self.dir2))

    def get_vertices(self, reverse_direction=False) -> list[tuple[Vec, Vec]]:
        d1, uv1 = (self.dir1, self.dir1_uv) if not reverse_direction else (self.dir2, self.dir2_uv)
        d2, uv2 = (self.dir2, self.dir2_uv) if not reverse_direction else (self.dir1, self.dir1_uv)
        return [
            (self.origin, self.origin_uv),
            (self.origin + d1, self.origin_uv + uv1),
            (self.origin + d1 + d2, self.origin_uv + uv1 + uv2),
            (self.origin + d2, self.origin_uv + uv2),
        ]

    def get_transformation_matrix(self):
        ihat = self.dir1
        jhat = self.dir2
        khat = np.cross(ihat, jhat)
        return np.array(
            [
                [ihat[0], jhat[0], khat[0], self.origin[0]],
                [ihat[1], jhat[1], khat[1], self.origin[1]],
                [ihat[2], jhat[2], khat[2], self.origin[2]],
                [0, 0, 0, 1],
            ]
        )

    def contains_point(self, point: np.ndarray) -> bool:
        inv_mat = np.linalg.inv(self.get_transformation_matrix())
        point_local = inv_mat @ np.array([*point, 1])
        valid_x = 0 <= point_local[0] <= 1 or math.isclose(point_local[0], 0) or math.isclose(point_local[0], 1)
        valid_y = 0 <= point_local[1] <= 1 or math.isclose(point_local[1], 0) or math.isclose(point_local[1], 1)
        valid_z = math.isclose(point_local[2], 0)
        return valid_x and valid_y and valid_z

    def contains_quad(self, other: Self) -> bool:
        return (
            self.contains_point(other.origin)
            and self.contains_point(other.origin + other.dir1)
            and self.contains_point(other.origin + other.dir2)
            and self.contains_point(other.origin + other.dir1 + other.dir2)
        )


@dataclass
class Cube:
    pos: np.ndarray
    size: np.ndarray

    def get_quads(self):
        faces = self.get_faces()
        vertex_positions = self.get_vertex_positions()

        def get_face_quad(face: Face, face_type: FaceType):
            positions = [vertex_positions[i] for i in face]

            origin = positions[0]
            dir1 = positions[1] - positions[0]
            dir2 = positions[2] - positions[0]

            size = (round(np.linalg.norm(dir2)), round(np.linalg.norm(dir1)))

            return RectQuad(origin, dir1, dir2, size, face_type)

        return [get_face_quad(face, face_type) for face_type, face in faces]

    def get_vertex_positions(self) -> list[Vec]:
        x, y, z = self.pos
        w, h, d = self.size

        return [
            vec(x, y, z),  # 0
            vec(x + w, y, z),  # 1
            vec(x, y + h, z),  # 2
            vec(x + w, y + h, z),  # 3
            vec(x, y, z + d),  # 4
            vec(x + w, y, z + d),  # 5
            vec(x, y + h, z + d),  # 6
            vec(x + w, y + h, z + d),  # 7
        ]

    def get_faces(self) -> list[tuple[FaceType, Face]]:
        return [
            ("Front", (0, 2, 1)),  # Front face
            ("Back", (5, 7, 4)),  # Back face
            ("Top", (2, 6, 3)),  # Top face
            ("Bottom", (4, 0, 5)),  # Bottom face
            ("Left", (4, 6, 0)),  # Left face
            ("Right", (1, 3, 5)),  # Right face
        ]
