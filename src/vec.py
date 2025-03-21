import numpy as np

type Vec = np.ndarray


def vec(*args):
    return np.array(args)


def normalize(vec: Vec) -> Vec:
    norm = np.linalg.norm(vec)
    return vec / norm if norm != 0 else vec
