import json
import subprocess
from argparse import ArgumentParser
from pathlib import Path
from typing import List, Literal, Tuple, Optional

Build = Literal["debug", "release"]
Game = Literal["mw", "pm", "rc", "cs"]
GAME_MW: Game = "mw"
GAME_PM: Game = "pm"
GAME_RC: Game = "rc"
GAME_CS: Game = "cs"


BUFSIZE = 8 * 1024


def cmp(one: Path, two: Path) -> bool:
    with one.open("rb") as fp1, two.open("rb") as fp2:
        while True:
            buf1 = fp1.read(BUFSIZE)
            buf2 = fp2.read(BUFSIZE)
            if buf1 != buf2:
                return False
            if not buf1:
                return True


def cmp_fuzzy(one: Path, two: Path, limit: int) -> bool:
    diffs = 0
    with one.open("rb") as fp1, two.open("rb") as fp2:
        while True:
            buf1 = fp1.read(BUFSIZE)
            buf2 = fp2.read(BUFSIZE)
            if buf1 != buf2:
                if len(buf1) != len(buf2):
                    return False
                diff = sum(b1 != b2 for (b1, b2) in zip(buf1, buf2))
                diffs += diff
                if diffs > limit:
                    return False
            if not buf1:
                return True


def name_to_game(name: str) -> Game:
    if name.endswith("-pm"):
        return GAME_PM
    if name.endswith("-recoil"):
        return GAME_RC
    if name.endswith("-cs"):
        return GAME_CS
    return GAME_MW


def campaign_mission(input_zbd: Path, zbd_dir: Path) -> Tuple[str, List[str]]:
    base_name = input_zbd.stem
    rel_path = input_zbd.relative_to(zbd_dir)
    parents = []
    for parent in rel_path.parents:
        parent_name = parent.name
        if parent_name:
            base_name = f"{parent_name}-{base_name}"
            parents.append(parent_name)
    return (base_name, parents)


def valid_messages(data: object) -> bool:
    if not isinstance(data, dict):
        print("Data is not a dict:", repr(data))
        return False

    try:
        language_id = data["language_id"]
        entries = data["entries"]
    except KeyError as e:
        print("Key missing:", e)
        return False

    if not isinstance(language_id, int):
        print("Language is not an int:", repr(language_id))
        return False

    if not isinstance(entries, list):
        print("Entries is not a list:", repr(language_id))
        return False

    if len(entries) < 30:
        print("Too few entries:", len(entries))
        return False

    return True


class Tester:
    def __init__(self, base_path: Path, output_base: Path, target_dir: Path):
        self.unzbd_exe = target_dir / "unzbd"
        self.rezbd_exe = target_dir / "rezbd"
        self.miscompares: List[Tuple[Path, Path]] = []
        self.base_path = base_path
        output_base.mkdir(exist_ok=True)

        self.versions = sorted(
            (
                (path.name, path / "zbd", output_base / path.name)
                for path in self.base_path.iterdir()
                if path.is_dir() and path.name != "demo"
            ),
            key=lambda value: value[0],
            reverse=True,
        )

        for _, _, output_dir in self.versions:
            output_dir.mkdir(exist_ok=True)

    def unzbd(self, command: str, game: Game, one: Path, two: Path, log: Path) -> None:
        env = {"RUST_LOG": "trace"}
        cmd = [str(self.unzbd_exe), game, command, str(one), str(two)]
        try:
            with log.open("wb") as f:
                subprocess.run(cmd, check=True, env=env, stderr=f)
        except subprocess.CalledProcessError:
            print(" ".join(cmd))
            raise

    def rezbd(self, command: str, game: Game, one: Path, two: Path, log: Path) -> None:
        env = {"RUST_LOG": "trace"}
        cmd = [str(self.rezbd_exe), game, command, str(one), str(two)]
        try:
            with log.open("wb") as f:
                subprocess.run(cmd, check=True, env=env, stderr=f)
        except subprocess.CalledProcessError:
            print(" ".join(cmd))
            raise

    def compare(self, one: Path, two: Path, limit: Optional[int] = None) -> None:
        if limit is None:
            res = cmp(one, two)
        else:
            res = cmp_fuzzy(one, two, limit)
        if not res:
            print("*** MISMATCH ***", one, two)
            self.miscompares.append((one, two))

    def print_miscompares(self) -> None:
        if self.miscompares:
            print("--- MISMATCH ---")
            for one, two in self.miscompares:
                print(one, two)
        else:
            print("--- ALL OK ---")

    def test_sounds(self) -> None:
        print("--- SOUNDS ---")
        for name, zbd_dir, output_base in self.versions:
            game = name_to_game(name)

            output_dir = output_base / "sounds"
            output_dir.mkdir(exist_ok=True)

            if game == GAME_RC:
                sounds_names = ["soundsl", "soundsm", "soundsh"]
            elif game == GAME_CS:
                sounds_names = ["soundsl", "soundsh"]
            else:
                sounds_names = ["soundsL", "soundsH"]

            for sounds in sounds_names:
                input_zbd = zbd_dir / f"{sounds}.zbd"
                zip_path = output_dir / f"{sounds}.zip"
                output_zbd = output_dir / f"{sounds}.zbd"
                read_log = output_dir / f"{sounds}-read.log"
                write_log = output_dir / f"{sounds}-write.log"

                print(game, name, sounds)

                self.unzbd("sounds", game, input_zbd, zip_path, read_log)
                self.rezbd("sounds", game, zip_path, output_zbd, write_log)
                self.compare(input_zbd, output_zbd)

    def test_interp(self) -> None:
        print("--- INTERP ---")
        for name, zbd_dir, output_base in self.versions:
            game = name_to_game(name)

            output_dir = output_base / "interp"
            output_dir.mkdir(exist_ok=True)

            input_zbd = zbd_dir / "interp.zbd"
            zip_path = output_dir / "interp.json"
            output_zbd = output_dir / "interp.zbd"
            read_log = output_dir / "interp-read.log"
            write_log = output_dir / "interp-write.log"

            print(game, name, "interp")

            self.unzbd("interp", game, input_zbd, zip_path, read_log)
            self.rezbd("interp", game, zip_path, output_zbd, write_log)
            self.compare(input_zbd, output_zbd)

    def test_messages(self) -> None:
        print("--- MESSAGES ---")
        for name, zbd_dir, output_base in self.versions:
            game = name_to_game(name)

            output_dir = output_base / "messages"
            output_dir.mkdir(exist_ok=True)

            if game == GAME_RC:
                msg_name = "messages"
            elif game == GAME_CS:
                msg_name = "strings"
            else:
                msg_name = "Mech3Msg"

            input_dll = zbd_dir.parent / f"{msg_name}.dll"
            output_json = output_dir / f"{msg_name}.json"

            print(game, name, f"{msg_name}")

            cmd = [
                str(self.unzbd_exe),
                game,
                "messages",
                str(input_dll),
                str(output_json),
            ]
            subprocess.run(cmd, check=True)
            # can't convert back to a DLL
            with output_json.open("r") as f:
                data = json.load(f)

            if not valid_messages(data):
                print("*** MISMATCH ***", input_dll, output_json)
                self.miscompares.append((input_dll, output_json))

    def test_textures(self) -> None:
        print("--- TEXTURES ---")
        for name, zbd_dir, output_base in self.versions:
            game = name_to_game(name)

            output_dir = output_base / "textures"
            output_dir.mkdir(exist_ok=True)

            texture_zbds = list(zbd_dir.rglob("*tex*.zbd"))
            if game == GAME_RC:
                texture_zbds += [zbd_dir / "image.zbd"]
            else:
                texture_zbds += [zbd_dir / "rimage.zbd"]

            for input_zbd in sorted(texture_zbds):
                base_name, parents = campaign_mission(input_zbd, zbd_dir)

                zip_path = output_dir / f"{base_name}.zip"
                output_zbd = output_dir / f"{base_name}.zbd"
                read_log = output_dir / f"{base_name}-log.log"
                write_log = output_dir / f"{base_name}-write.log"

                print(game, name, *parents, input_zbd.name)

                self.unzbd("textures", game, input_zbd, zip_path, read_log)
                self.rezbd("textures", game, zip_path, output_zbd, write_log)
                self.compare(input_zbd, output_zbd)

    def test_reader(self) -> None:
        print("--- READER ---")
        for name, zbd_dir, output_base in self.versions:
            game = name_to_game(name)

            output_dir = output_base / "reader"
            output_dir.mkdir(exist_ok=True)

            if game == GAME_RC or game == GAME_CS:
                rdr_glob = "zrdr.zbd"
            else:
                rdr_glob = "reader*.zbd"

            for input_zbd in sorted(zbd_dir.rglob(rdr_glob)):
                base_name, parents = campaign_mission(input_zbd, zbd_dir)

                zip_path = output_dir / f"{base_name}.zip"
                output_zbd = output_dir / f"{base_name}.zbd"
                read_log = output_dir / f"{base_name}-log.log"
                write_log = output_dir / f"{base_name}-write.log"

                print(game, name, *parents, input_zbd.name)

                self.unzbd("reader", game, input_zbd, zip_path, read_log)
                self.rezbd("reader", game, zip_path, output_zbd, write_log)
                self.compare(input_zbd, output_zbd)

    def test_motion(self) -> None:
        print("--- MOTION ---")
        for name, zbd_dir, output_base in self.versions:
            game = name_to_game(name)

            if game != GAME_MW and game != GAME_PM:
                continue

            output_dir = output_base / "motion"
            output_dir.mkdir(exist_ok=True)

            input_zbd = zbd_dir / "motion.zbd"
            zip_path = output_dir / "motion.zip"
            output_zbd = output_dir / "motion.zbd"
            read_log = output_dir / "motion-read.log"
            write_log = output_dir / "motion-write.log"

            print(game, name, "motion")

            self.unzbd("motion", game, input_zbd, zip_path, read_log)
            self.rezbd("motion", game, zip_path, output_zbd, write_log)
            self.compare(input_zbd, output_zbd)

    def test_mechlib(self) -> None:
        print("--- MECHLIB ---")
        for name, zbd_dir, output_base in self.versions:
            game = name_to_game(name)

            if game != GAME_MW and game != GAME_PM:
                continue

            output_dir = output_base / "mechlib"
            output_dir.mkdir(exist_ok=True)

            input_zbd = zbd_dir / "mechlib.zbd"
            zip_path = output_dir / "mechlib.zip"
            output_zbd = output_dir / "mechlib.zbd"
            read_log = output_dir / "mechlib-read.log"
            write_log = output_dir / "mechlib-write.log"

            print(game, name, "mechlib")

            self.unzbd("mechlib", game, input_zbd, zip_path, read_log)
            self.rezbd("mechlib", game, zip_path, output_zbd, write_log)
            self.compare(input_zbd, output_zbd)

    def test_gamez(self) -> None:
        print("--- GAMEZ ---")
        for name, zbd_dir, output_base in self.versions:
            game = name_to_game(name)

            output_dir = output_base / "gamez"
            output_dir.mkdir(exist_ok=True)

            for input_zbd in sorted(zbd_dir.rglob("gamez.zbd")):
                base_name, parents = campaign_mission(input_zbd, zbd_dir)

                if game == GAME_RC and (parents == ["m6"] or parents == ["m9"]):
                    continue

                zip_path = output_dir / f"{base_name}.zip"
                output_zbd = output_dir / f"{base_name}.zbd"
                read_log = output_dir / f"{base_name}-read.log"
                write_log = output_dir / f"{base_name}-write.log"

                print(game, name, *parents, input_zbd.name)

                self.unzbd("gamez", game, input_zbd, zip_path, read_log)
                self.rezbd("gamez", game, zip_path, output_zbd, write_log)
                self.compare(input_zbd, output_zbd)

    def test_anim(self) -> None:
        print("--- ANIM ---")
        for name, zbd_dir, output_base in self.versions:
            game = name_to_game(name)

            if game != GAME_MW:
                print("SKIPPING", name)
                continue

            output_dir = output_base / "anim"
            output_dir.mkdir(exist_ok=True)

            for input_zbd in sorted(zbd_dir.rglob("anim.zbd")):
                base_name, parents = campaign_mission(input_zbd, zbd_dir)

                zip_path = output_dir / f"{base_name}.zip"
                output_zbd = output_dir / f"{base_name}.zbd"
                read_log = output_dir / f"{base_name}-read.log"
                write_log = output_dir / f"{base_name}-write.log"

                print(game, name, *parents, input_zbd.name)

                self.unzbd("anim", game, input_zbd, zip_path, read_log)
                self.rezbd("anim", game, zip_path, output_zbd, write_log)
                self.compare(input_zbd, output_zbd)

    def test_zmap(self) -> None:
        print("--- ZMAP ---")
        for name, zbd_dir, output_base in self.versions:
            game = name_to_game(name)

            if game != GAME_RC:
                continue

            # maps are not in the zbd dir
            map_dir = zbd_dir.parent / "maps"

            output_dir = output_base / "zmap"
            output_dir.mkdir(exist_ok=True)

            for input_zmap in sorted(
                map_dir.rglob("*.zmap"), key=lambda p: int(p.stem.strip("m"))
            ):
                base_name = input_zmap.stem

                json_path = output_dir / f"{base_name}.json"
                output_zmap = output_dir / f"{base_name}.zmap"
                read_log = output_dir / f"{base_name}-read.log"
                write_log = output_dir / f"{base_name}-write.log"

                print(game, name, base_name)

                self.unzbd("zmap", game, input_zmap, json_path, read_log)
                self.rezbd("zmap", game, json_path, output_zmap, write_log)
                self.compare(input_zmap, output_zmap)

    def test_planes(self) -> None:
        print("--- PLANES ---")
        for name, zbd_dir, output_base in self.versions:
            game = name_to_game(name)

            if game != GAME_CS:
                continue

            output_dir = output_base / "planes"
            output_dir.mkdir(exist_ok=True)

            input_zbd = zbd_dir / "planes.zbd"
            zip_path = output_dir / "planes.zip"
            output_zbd = output_dir / "planes.zbd"
            read_log = output_dir / "planes-read.log"
            write_log = output_dir / "planes-write.log"

            print(game, name, "planes")

            self.unzbd("gamez", game, input_zbd, zip_path, read_log)
            self.rezbd("gamez", game, zip_path, output_zbd, write_log)
            # planes has textures with duplicate names. this would actually
            # be surprisingly hard to fix, and with very little upside. so
            # instead, we'll simply ignore mismatches up to a certain limit,
            # since planes uses similar code to gamez, so we have coverage.
            self.compare(input_zbd, output_zbd, limit=100)


def main() -> None:
    parser = ArgumentParser()
    parser.add_argument(
        "versions_dir", type=lambda value: Path(value).resolve(strict=True)
    )
    parser.add_argument(
        "output_dir", type=lambda value: Path(value).resolve(strict=True)
    )
    parser.add_argument("--release", action="store_true")
    parser.add_argument("--target-dir", default="target")
    args = parser.parse_args()

    build: Build = "release" if args.release else "debug"
    target_dir = Path(args.target_dir).resolve(strict=True) / build
    print("running", build, target_dir)
    tester = Tester(args.versions_dir, args.output_dir, target_dir)
    tester.test_sounds()
    tester.test_interp()
    tester.test_messages()
    tester.test_reader()
    tester.test_textures()
    tester.test_mechlib()
    tester.test_motion()
    tester.test_gamez()
    tester.test_anim()
    tester.test_zmap()
    tester.test_planes()
    tester.print_miscompares()


if __name__ == "__main__":
    main()
