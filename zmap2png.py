import argparse
import json
import matplotlib.pyplot as plt
from pathlib import Path
from typing import Any, Tuple, Sequence

Color = Tuple[float, float, float]
Feature = Tuple[Color, Sequence[float], Sequence[float], Sequence[float]]


def split_feature(feature: Any) -> Feature:
    c = feature["color"]
    color = (c["r"] / 255.0, c["g"] / 255.0, c["b"] / 255.0)
    x = [v["x"] for v in feature["vertices"]]
    y = [v["y"] for v in feature["vertices"]]
    z = [v["z"] for v in feature["vertices"]]
    x.append(x[0])
    y.append(y[0])
    z.append(z[0])
    return (color, x, y, z)


def split_map_features(m: Any) -> Sequence[Feature]:
    return [split_feature(f) for f in m["features"]]


def plot_map2d(m: Any, png_path: Path, aspect_square: bool = False) -> None:
    features = split_map_features(m)

    fig = plt.figure(facecolor="black")
    ax = fig.add_subplot()
    ax.set_facecolor("black")
    ax.get_xaxis().set_visible(False)
    ax.get_yaxis().set_visible(False)
    ax.set_frame_on(False)

    for (color, x, y, _z) in features:
        ax.plot(x, y, color=color)

    if aspect_square:
        x_min, x_max = ax.get_xlim()
        y_min, y_max = ax.get_ylim()
        aspect = abs((x_max - x_min) / (y_max - y_min))
    else:
        aspect = "equal"
    ax.set_aspect(aspect)

    fig.tight_layout()
    fig.savefig(
        str(png_path),
        dpi=300,
        pad_inches=0,
        bbox_inches="tight",
    )


def dir_path(value: Any) -> Path:
    path = Path(value).resolve(strict=True)
    if not path.is_dir():
        raise ValueError()
    return path


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "zmap_dir",
        type=dir_path,
        help="Path to the directory with zmap JSON files",
    )
    parser.add_argument("--aspect-square", action="store_true")
    args = parser.parse_args()

    # m7-m12 are multi-player maps, and are just copies of m1
    for i in range(1, 7):
        name = f"m{i}"
        json_path = args.zmap_dir / f"{name}.json"
        png_path = args.zmap_dir / f"{name}.png"
        with json_path.open("r") as f:
            m = json.load(f)
        plot_map2d(m, png_path, args.aspect_square)


if __name__ == "__main__":
    main()
