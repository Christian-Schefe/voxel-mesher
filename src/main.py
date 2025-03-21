import argparse
from app import app
import pathlib


def read_file(path: str) -> str:
    with open(path, "r") as f:
        return f.read()


def main():
    parser = argparse.ArgumentParser(description="Voxel Mesher CLI")
    parser.add_argument("--input", type=str, required=True, help="Path to the input file")
    parser.add_argument("--output", type=str, required=True, help="Path to the output file")

    args = parser.parse_args()

    for file_path in glob_files(args.input):
        print(f"Processing {file_path}")
        input_file = read_file(file_path)
        input_file_name = pathlib.Path(file_path).with_suffix("").name

        app(input_file, input_file_name, args.output)


def glob_files(path: str) -> list[str]:
    return [str(x) for x in pathlib.Path().glob(path)]


if __name__ == "__main__":
    main()
