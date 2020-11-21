import filecmp
import subprocess
from argparse import ArgumentParser
from pathlib import Path
from typing import List, Tuple


class Tester:
    def __init__(self, base_path: Path, output_base: Path, build: str = "debug"):
        self.unzbd_exe = f"target/{build}/unzbd"
        self.rezbd_exe = f"target/{build}/rezbd"
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

    def unzbd(self, command: str, one: Path, two: Path, is_pm: bool = False) -> None:
        cmd = [self.unzbd_exe, command, str(one), str(two)]
        if is_pm:
            cmd.insert(1, "--pm")
        subprocess.run(cmd, check=True)

    def rezbd(self, command: str, one: Path, two: Path, is_pm: bool = False) -> None:
        cmd = [self.rezbd_exe, command, str(one), str(two)]
        if is_pm:
            cmd.insert(1, "--pm")
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
            is_pm = name.endswith("-pm")

            print(name, "soundsL.zbd", is_pm)
            input_zbd = zbd_dir / "soundsL.zbd"
            zip_path = output_base / "soundsL.zip"
            output_zbd = output_base / "soundsL.zbd"

            self.unzbd("sounds", input_zbd, zip_path, is_pm)
            self.rezbd("sounds", zip_path, output_zbd, is_pm)
            self.compare(input_zbd, output_zbd)

            print(name, "soundsH.zbd", is_pm)
            input_zbd = zbd_dir / "soundsH.zbd"
            zip_path = output_base / "soundsH.zip"
            output_zbd = output_base / "soundsH.zbd"

            self.unzbd("sounds", input_zbd, zip_path, is_pm)
            self.rezbd("sounds", zip_path, output_zbd, is_pm)
            self.compare(input_zbd, output_zbd)

    def test_interp(self) -> None:
        print("--- INTERP ---")
        for name, zbd_dir, output_base in self.versions:
            print(name, "interp.zbd")
            input_zbd = zbd_dir / "interp.zbd"
            zip_path = output_base / "interp.json"
            output_zbd = output_base / "interp.zbd"
            self.unzbd("interp", input_zbd, zip_path)
            self.rezbd("interp", zip_path, output_zbd)
            self.compare(input_zbd, output_zbd)

    def test_resources(self) -> None:
        print("--- RESOURCES ---")
        for name, zbd_dir, output_base in self.versions:
            print(name, "Mech3Msg.dll")
            input_dll = zbd_dir.parent / "Mech3Msg.dll"
            output_json = output_base / "Mech3Msg.json"
            self.unzbd("messages", input_dll, output_json)
            # can't convert back to a DLL

    def test_textures(self) -> None:
        print("--- TEXTURES ---")
        for name, zbd_dir, output_base in self.versions:
            output_dir = output_base / "textures"
            output_dir.mkdir(exist_ok=True)

            texture_zbds = list(zbd_dir.rglob("*tex*.zbd")) + [zbd_dir / "rimage.zbd"]
            for input_zbd in sorted(texture_zbds):
                rel_path = input_zbd.relative_to(zbd_dir)
                mission = rel_path.parent.name
                if not mission:
                    zip_name = f"{input_zbd.stem}.zip"
                    zbd_name = f"{input_zbd.stem}.zbd"
                else:
                    zip_name = f"{mission}-{input_zbd.stem}.zip"
                    zbd_name = f"{mission}-{input_zbd.stem}.zbd"

                zip_path = output_dir / zip_name
                output_zbd = output_dir / zbd_name
                print(name, mission, input_zbd.name)
                self.unzbd("textures", input_zbd, zip_path)
                self.rezbd("textures", zip_path, output_zbd)
                self.compare(input_zbd, output_zbd)

    def test_reader(self) -> None:
        print("--- READER ---")
        for name, zbd_dir, output_base in self.versions:
            output_dir = output_base / "reader"
            output_dir.mkdir(exist_ok=True)

            is_pm = name.endswith("-pm")

            for input_zbd in sorted(zbd_dir.rglob("reader*.zbd")):
                rel_path = input_zbd.relative_to(zbd_dir)
                mission = rel_path.parent.name
                if not mission:
                    zip_name = f"{input_zbd.stem}.zip"
                    zbd_name = f"{input_zbd.stem}.zbd"
                else:
                    zip_name = f"{mission}-{input_zbd.stem}.zip"
                    zbd_name = f"{mission}-{input_zbd.stem}.zbd"

                zip_path = output_dir / zip_name
                output_zbd = output_dir / zbd_name
                print(name, mission, input_zbd.name, is_pm)
                self.unzbd("reader", input_zbd, zip_path, is_pm)
                self.rezbd("reader", zip_path, output_zbd, is_pm)
                self.compare(input_zbd, output_zbd)

    def test_motion(self) -> None:
        print("--- MOTION ---")
        for name, zbd_dir, output_base in self.versions:
            is_pm = name.endswith("-pm")
            print(name, "motion.zbd", is_pm)

            input_zbd = zbd_dir / "motion.zbd"
            zip_path = output_base / "motion.zip"
            output_zbd = output_base / "motion.zbd"
            self.unzbd("motion", input_zbd, zip_path, is_pm)
            self.rezbd("motion", zip_path, output_zbd, is_pm)
            self.compare(input_zbd, output_zbd)

    def test_mechlib(self) -> None:
        print("--- MECHLIB ---")
        for name, zbd_dir, output_base in self.versions:
            print(name, "mechlib.zbd")

            if name.endswith("-pm"):
                print("SKIPPING")
                continue

            input_zbd = zbd_dir / "mechlib.zbd"
            zip_path = output_base / "mechlib.zip"
            output_zbd = output_base / "mechlib.zbd"
            self.unzbd("mechlib", input_zbd, zip_path)
            self.rezbd("mechlib", zip_path, output_zbd)
            self.compare(input_zbd, output_zbd)

    def test_gamez(self) -> None:
        print("--- GAMEZ ---")
        for name, zbd_dir, output_base in self.versions:
            if name.endswith("-pm"):
                print("SKIPPING")
                continue

            output_dir = output_base / "gamez"
            output_dir.mkdir(exist_ok=True)

            for input_zbd in sorted(zbd_dir.rglob("gamez.zbd")):
                rel_path = input_zbd.relative_to(zbd_dir)
                mission = rel_path.parent.name
                zip_name = f"{mission}-{input_zbd.stem}.zip"
                zbd_name = f"{mission}-{input_zbd.stem}.zbd"

                zip_path = output_dir / zip_name
                output_zbd = output_dir / zbd_name
                print(name, mission, input_zbd.name)
                self.unzbd("gamez", input_zbd, zip_path)
                self.rezbd("gamez", zip_path, output_zbd)
                self.compare(input_zbd, output_zbd)

    def test_anim(self) -> None:
        print("--- ANIM ---")
        for name, zbd_dir, output_base in self.versions:
            if name.endswith("-pm"):
                print("SKIPPING")
                continue

            output_dir = output_base / "anim"
            output_dir.mkdir(exist_ok=True)

            for input_zbd in sorted(zbd_dir.rglob("anim.zbd")):
                rel_path = input_zbd.relative_to(zbd_dir)
                mission = rel_path.parent.name
                zip_name = f"{mission}-{input_zbd.stem}.zip"
                zbd_name = f"{mission}-{input_zbd.stem}.zbd"

                zip_path = output_dir / zip_name
                output_zbd = output_dir / zbd_name
                print(name, mission, input_zbd.name)
                self.unzbd("anim", input_zbd, zip_path)
                self.rezbd("anim", zip_path, output_zbd)
                self.compare(input_zbd, output_zbd)


def main() -> None:
    parser = ArgumentParser()
    parser.add_argument(
        "versions_dir", type=lambda value: Path(value).resolve(strict=True)
    )
    parser.add_argument(
        "output_dir", type=lambda value: Path(value).resolve(strict=True)
    )
    parser.add_argument("--release", action="store_true")
    args = parser.parse_args()

    build = "release" if args.release else "debug"
    print("running", build)
    tester = Tester(args.versions_dir, args.output_dir, build)
    tester.test_sounds()
    tester.test_interp()
    tester.test_resources()
    tester.test_reader()
    tester.test_mechlib()
    tester.test_motion()
    tester.test_textures()
    tester.test_gamez()
    tester.test_anim()
    tester.print_miscompares()


if __name__ == "__main__":
    main()
