import filecmp
import json
import subprocess
from argparse import ArgumentParser
from pathlib import Path
from typing import List, Literal, Tuple

Build = Literal["debug", "release"]
Game = Literal["mw", "pm", "rc", "cs"]
GAME_MW: Game = "mw"
GAME_PM: Game = "pm"
GAME_RC: Game = "rc"
GAME_CS: Game = "cs"


def name_to_game(name: str) -> Game:
    if name.endswith("-pm"):
        return GAME_PM
    if name.endswith("-recoil"):
        return GAME_RC
    if name.endswith("-cs"):
        return GAME_CS
    return GAME_MW


def campaign_mission(input_zbd: Path, zbd_dir: Path) -> Tuple[str, str, List[str]]:
    zip_name = f"{input_zbd.stem}.zip"
    zbd_name = f"{input_zbd.stem}.zbd"
    rel_path = input_zbd.relative_to(zbd_dir)
    parents = []
    for parent in rel_path.parents:
        parent_name = parent.name
        if parent_name:
            zip_name = f"{parent_name}-{zip_name}"
            zbd_name = f"{parent_name}-{zbd_name}"
            parents.append(parent_name)
    return (zip_name, zbd_name, parents)


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

    def unzbd(self, command: str, game: Game, one: Path, two: Path) -> None:
        cmd = [str(self.unzbd_exe), game, command, str(one), str(two)]
        subprocess.run(cmd, check=True)

    def rezbd(self, command: str, game: Game, one: Path, two: Path) -> None:
        cmd = [str(self.rezbd_exe), game, command, str(one), str(two)]
        subprocess.run(cmd, check=True)

    def compare(self, one: Path, two: Path) -> None:
        if not filecmp.cmp(one, two, shallow=False):
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

            if game == GAME_RC:
                sounds_names = ["soundsl", "soundsm", "soundsh"]
            elif game == GAME_CS:
                sounds_names = ["soundsl", "soundsh"]
            else:
                sounds_names = ["soundsL", "soundsH"]

            for sounds in sounds_names:
                print(name, f"{sounds}.zbd", game)
                input_zbd = zbd_dir / f"{sounds}.zbd"
                zip_path = output_base / f"{sounds}.zip"
                output_zbd = output_base / f"{sounds}.zbd"

                self.unzbd("sounds", game, input_zbd, zip_path)
                self.rezbd("sounds", game, zip_path, output_zbd)
                self.compare(input_zbd, output_zbd)

    def test_interp(self) -> None:
        print("--- INTERP ---")
        for name, zbd_dir, output_base in self.versions:
            game = name_to_game(name)

            print(name, "interp.zbd", game)
            input_zbd = zbd_dir / "interp.zbd"
            zip_path = output_base / "interp.json"
            output_zbd = output_base / "interp.zbd"
            self.unzbd("interp", game, input_zbd, zip_path)
            self.rezbd("interp", game, zip_path, output_zbd)
            self.compare(input_zbd, output_zbd)

    def test_messages(self) -> None:
        print("--- MESSAGES ---")
        for name, zbd_dir, output_base in self.versions:
            game = name_to_game(name)

            if game == GAME_RC:
                msg_name = "messages"
            elif game == GAME_CS:
                msg_name = "strings"
            else:
                msg_name = "Mech3Msg"

            print(name, f"{msg_name}.dll", game)
            input_dll = zbd_dir.parent / f"{msg_name}.dll"
            output_json = output_base / f"{msg_name}.json"
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

            def _valid_messages(data: object) -> bool:
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

            if not _valid_messages(data):
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
                rel_path = input_zbd.relative_to(zbd_dir)
                campaign = rel_path.parent.name
                if not campaign:
                    zip_name = f"{input_zbd.stem}.zip"
                    zbd_name = f"{input_zbd.stem}.zbd"
                else:
                    zip_name = f"{campaign}-{input_zbd.stem}.zip"
                    zbd_name = f"{campaign}-{input_zbd.stem}.zbd"

                zip_path = output_dir / zip_name
                output_zbd = output_dir / zbd_name
                print(name, campaign, input_zbd.name, game)
                self.unzbd("textures", game, input_zbd, zip_path)
                self.rezbd("textures", game, zip_path, output_zbd)
                self.compare(input_zbd, output_zbd)

    def test_reader(self) -> None:
        print("--- READER ---")
        for name, zbd_dir, output_base in self.versions:
            game = name_to_game(name)

            if game == GAME_RC or game == GAME_CS:
                rdr_glob = "zrdr.zbd"
            else:
                rdr_glob = "reader*.zbd"

            output_dir = output_base / "reader"
            output_dir.mkdir(exist_ok=True)

            for input_zbd in sorted(zbd_dir.rglob(rdr_glob)):
                zip_name, zbd_name, parents = campaign_mission(input_zbd, zbd_dir)
                zip_path = output_dir / zip_name
                output_zbd = output_dir / zbd_name
                print(name, *parents, input_zbd.name, game)
                self.unzbd("reader", game, input_zbd, zip_path)
                self.rezbd("reader", game, zip_path, output_zbd)
                self.compare(input_zbd, output_zbd)

    def test_motion(self) -> None:
        print("--- MOTION ---")
        for name, zbd_dir, output_base in self.versions:
            game = name_to_game(name)
            if game == GAME_RC or game == GAME_CS:
                print("SKIPPING", name)
                continue

            print(name, "motion.zbd", game)

            input_zbd = zbd_dir / "motion.zbd"
            zip_path = output_base / "motion.zip"
            output_zbd = output_base / "motion.zbd"
            self.unzbd("motion", game, input_zbd, zip_path)
            self.rezbd("motion", game, zip_path, output_zbd)
            self.compare(input_zbd, output_zbd)

    def test_mechlib(self) -> None:
        print("--- MECHLIB ---")
        for name, zbd_dir, output_base in self.versions:
            game = name_to_game(name)
            if game == GAME_RC or game == GAME_CS:
                print("SKIPPING", name)
                continue

            print(name, "mechlib.zbd", game)

            input_zbd = zbd_dir / "mechlib.zbd"
            zip_path = output_base / "mechlib.zip"
            output_zbd = output_base / "mechlib.zbd"
            self.unzbd("mechlib", game, input_zbd, zip_path)
            self.rezbd("mechlib", game, zip_path, output_zbd)
            self.compare(input_zbd, output_zbd)

    def test_gamez(self) -> None:
        print("--- GAMEZ ---")
        for name, zbd_dir, output_base in self.versions:
            game = name_to_game(name)

            output_dir = output_base / "gamez"
            output_dir.mkdir(exist_ok=True)

            for input_zbd in sorted(zbd_dir.rglob("gamez.zbd")):
                zip_name, zbd_name, parents = campaign_mission(input_zbd, zbd_dir)
                zip_path = output_dir / zip_name
                output_zbd = output_dir / zbd_name
                print(name, *parents, input_zbd.name, game)
                self.unzbd("gamez", game, input_zbd, zip_path)
                self.rezbd("gamez", game, zip_path, output_zbd)
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
                zip_name, zbd_name, parents = campaign_mission(input_zbd, zbd_dir)
                zip_path = output_dir / zip_name
                output_zbd = output_dir / zbd_name
                print(name, *parents, input_zbd.name, game)
                self.unzbd("anim", game, input_zbd, zip_path)
                self.rezbd("anim", game, zip_path, output_zbd)
                self.compare(input_zbd, output_zbd)

    def test_zmap(self) -> None:
        print("--- ZMAP ---")
        for name, zbd_dir, output_base in self.versions:
            game = name_to_game(name)
            if game != GAME_RC:
                continue

            map_dir = zbd_dir.parent / "maps"
            output_dir = output_base / "zmap"
            output_dir.mkdir(exist_ok=True)

            for input_zmap in sorted(map_dir.rglob("*.zmap"), key=lambda p: int(p.stem.strip("m"))):
                json_name = f"{input_zmap.stem}.json"
                zmap_name = input_zmap.name

                json_path = output_dir / json_name
                output_zmap = output_dir / zmap_name

                print(name, input_zmap.name, game)
                self.unzbd("zmap", game, input_zmap, json_path)
                self.rezbd("zmap", game, json_path, output_zmap)
                self.compare(input_zmap, output_zmap)


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
    tester.print_miscompares()


if __name__ == "__main__":
    main()
