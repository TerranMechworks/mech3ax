# MechWarrior 3 Asset Extractor

MechWarrior 3 Asset Extractor (`mech3rs`) is a cross-platform, GPLv3 project to extract assets from the 1998 MechWarrior 3 game to modern formats and back. It has been tested on Windows, macOS, and Linux (Ubuntu).

Obviously, this is an unofficial fan effort and not connected to the developers or publishers.

[![The Annihilator 'Mech running](.github/mech_annihilator_run.gif)](https://imgur.com/a/H5pB1Vd)

## Currently supported

Various versions of the MechWarror 3 base game have been tested (including US versions 1.0/1.1/1.2/Gold Edition, German version 1.0, with and without patches). If you are in possession of any other versions, especially the French versions, please get in touch! The expansion, Pirate's Moon, is not supported.

The conversions are binary-accurate, so converting from a `*.zbd` file and then back to a `*.zbd` file produces the same file.

- Sound archives (`sounds*.zbd`)
- Interpreter scripts (`interp.zbd`)
- All texture archives (`rimage.zbd`, `rmechtex*.zbd`, `rtexture*.zbd`, `texture*.zbd`)
- Reader archives (`reader*.zbd`)
- Motion data (`motion.zbd`) can be converted binary-accurately. Because the model data is not very well understood, applying the animations isn't perfect. Some limbs have incorrect translations; it's possible these aren't meant to be applied
- 'mech models, and texture/material information (`mechlib.zbd`)
- All messages can be extracted from `Mech3Msg.dll`

Not supported (yet?):

- `gamez.zbd` files, which contain texture references, materials, meshes, and nodes for each scenario
- `anim.zbd` files, which contain pre-compiled animation definitions similar to the data in `reader*.zbd`
- The Pirate's Moon expansions
- The demo likely won't ever be supported, because it uses different versions/data structures

## How to use

**You will need a copy of the game. Do not ask me for an (illegal) copy.**

It's easiest to grab the pre-build binaries. Otherwise, see the [development](#development) section below on how to build from source. There are two command-line programs.

On macOS or Linux, you can run them like this:

```bash
unzbd interp "original/zbd/interp.zbd" "interp.json"
rezbd interp "interp.json" "interp.zbd"
# the files should be the same
cmp "original/zbd/interp.zbd" "interp.zbd"
```

On Windows, you can use either the command line (`cmd.exe`) or Powershell (which I'd recommend):

```powershell
PS> unzbd.exe interp "C:\Program Files (x86)\MechWarrior 3\zbd\interp.zbd" ".\interp.zip"
PS> rezbd.exe interp ".\interp.zip" ".\interp.zbd"
PS> comp /M "C:\Program Files (x86)\MechWarrior 3\zbd\interp.zbd" ".\interp.zbd"
Comparing C:\Program Files (x86)\MechWarrior 3\zbd\interp.zbd and .\interp.zbd...
Files compare OK
```

Provided subcommands:

* `sounds` (produces a `*.zip` file)
* `interp` (produces a `*.json` file)
* `reader` (produces a `*.zip` file)
* `messages` (produces a `*.json` file, `unzbd` only)
* `textures` (produces a `*.zip` file)
* `motion` (produces a `*.zip` file)
* `mechlib` (produces a `*.zip` file)

## Development

[Rust](https://www.rust-lang.org/) is required, this project uses `stable`.

This project uses [pre-commit](https://pre-commit.com/) to run `cargo fmt` when you commit (and only on the committed code, not unstaged code). If this sounds useful to you, install `pre-commit`, and put the hooks in place:

```bash
pre-commit install
```

## License

MechWarrior 3 Asset Extractor is GPLv3 licensed. Please see `LICENSE`.
