# MechWarrior 3 Asset Extractor

MechWarrior 3 Asset Extractor (`mech3ax`) is a cross-platform, open-source project to extract assets from certain games developed by Zipper Interactive™ to modern formats and back:

* the Recoil™ game (1999)
* the MechWarrior 3™ base game (1999)
* the MechWarrior 3 Pirate's Moon™ expansion (1999)

Zipper Interactive™ was trademark or registered trademark of Sony Computer Entertainment America LLC.

Obviously, this is an unofficial fan effort and not connected to the developers, publishers, or rightsholders. [Join us on MW3 Discord](https://discord.gg/Be53gMy), or the Recoil Discord!

[![The Annihilator 'Mech running](.github/mech_annihilator_run.gif)](https://imgur.com/a/H5pB1Vd)

## Currently supported

### MechWarrior 3

Various versions of the MechWarror 3 base game have been tested (including US versions 1.0/1.1/1.2/Gold Edition, German version 1.0, each with and without the 1.2 patch). If you are in possession of any other versions, especially the French versions, please get in touch! The expansion, Pirate's Moon, has limited support (see below).

The conversions are binary-accurate, so converting from a `*.zbd` file and then back to a `*.zbd` file produces the same file.

- Sound archives (`sounds*.zbd`) containing sound effects to ZIP archives of WAV files - note the background music is streamed from the CD and never installed
- Interpreter scripts (`interp.zbd`) to a JSON file - these small, interpreted scripts drive which files the engine loads
- All image/texture packages (`rimage.zbd`, `rmechtex*.zbd`, `rtexture*.zbd`, `texture*.zbd`) to ZIP archives of PNG files
- Reader archives (`reader*.zbd`) containing game data to ZIP archives of JSON files
- Motion data (`motion.zbd`) containing 'mech animation data to ZIP archives of JSON files - because the model data is not very well understood, applying the animations isn't perfect. Some limbs have incorrect translations; it's possible these aren't meant to be applied
- Mechlib archives (`mechlib.zbd`) containing 'mech models, and texture/material information to ZIP archives of JSON files
- All messages can be extracted from `Mech3Msg.dll` to a JSON file
- Pre-compiled animation definitions (`anim.zbd`) containing (baked) animations also present in reader archives (probably for faster loading) to ZIP archives of JSON files
- Game asset/GameZ archives (`gamez.zbd`) containing texture references, materials, meshes, and nodes for each scenario to ZIP archives of JSON files (some of this data is still rough)

Not supported (yet?):

- Savegame files
- The demo likely won't ever be supported, because it uses different versions/data structures
- Background music/ambient tracks [can be extracted from the CD](https://terranmechworks.com/mech3doc/ambient-tracks/) using e.g. [ExactAudioCopy](http://www.exactaudiocopy.de/) or other programs, so it isn't worth re-inventing this
- Similarly, video files [can be converted using `ffmpeg`](https://terranmechworks.com/mech3doc/avi-files/) to modern codecs, or played back using [VLC media player](https://www.videolan.org/vlc/)

### Pirate's Moon

`mechlib.zbd`, `anim.zbd`, and `gamez.zbd` files are not supported yet.

### Recoil

All maps (`m*.zmap`), `anim.zbd`, and `gamez.zbd` files are not supported yet.

## How to use

**You will need a copy of the game. Do not ask me for an (illegal) copy.**

It's easiest to [grab the pre-build binaries from releases](https://github.com/TerranMechworks/mech3ax/releases). Otherwise, see the [development](#development) section below on how to build from source. There are two command-line programs.

**Warning**: The output file formats aren't stable yet and may change in future - please don't build tools around the output yet.

On macOS or Linux, you can run them like this:

```bash
unzbd interp "original/zbd/interp.zbd" "interp.json"
rezbd interp "interp.json" "interp.zbd"
# the files should be the same
cmp "original/zbd/interp.zbd" "interp.zbd"
```

On Windows, you can use either the command line (`cmd.exe`) or Powershell (which I'd recommend):

```powershell
PS> unzbd.exe interp "C:\Program Files (x86)\MechWarrior 3\zbd\interp.zbd" ".\interp.json"
PS> rezbd.exe interp ".\interp.json" ".\interp.zbd"
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
* `anim` (produces a `*.zip` file)
* `gamez` (produces a `*.zip` file)

## Blender scripts

Blender 3.2.2 or higher is recommended. Blender's APIs do change, so you may need to use a version closely matching that one. It will definitely *not* work with versions below 2.80, but if you have success running it with newer versions, let me know so I can update this README.

This is a bit tricky to get running, because of the dependencies. Your install location may vary. Naturally, you can specify the absolute path. It's easier if the Blender executable can be found. For macOS and Linux, this can be achieved by an alias in your shell's profile, e.g. `.bashrc`:

```bash
alias blender="/Applications/Blender.app/Contents/MacOS/Blender"
```

For Windows/PowerShell, you can add an alias to either the current session (or the appropriate `profile.ps1`):

```powershell
New-Alias blender "C:\Program Files\Blender Foundation\Blender 2.90\blender.exe"
```

Assuming the above, and you have extracted the mechlib files and mech textures to the same directory, you can run:

```bash
blender \
    --background \
    --factory-startup \
    --python "mechlib2blend.py" \
    -- \
    "mechlib.zip" \
    --mechtex "rmechtex.zip" \
    --motion "motion.zip" \
    "supernova"
```

where `--mechtex` and `--motion` are optional. If `--mechtex` is specified, textures are extracted, applied, and packed into the `.blend` file. If `--motion` is specified, mech motions/animations are loaded and applied to the model.

Assuming the above, and you have extracted the gamez files, game textures, and mech textures to the same directory, you can run:

```bash
blender \
    --background \
    --factory-startup \
    --python "gamez2blend.py" \
    -- \
    "gamez.zip" \
    --rtexture "c1-rtexture.zip" \
    --rmechtex "rmechtex.zip"
```

where `--rtexture` and `--rmechtex` are optional.

## Changelog

### [0.5.1] - unreleased

* AnimDef field 152 is likely the status (`anim`, breaking change)
* ReaderLookup field 40 is likely the "in use" or "loaded into world" flag (`anim`, breaking change)
* AnimRef field 68 is likely a pointer (`anim`, breaking change)
* Remove ResetState structure, it's likely SeqDefInfo instead (`anim`, breaking change)
* Make `--dump-ids` the default mode for `unzbd messages` (`unzbd`, breaking change)

### [0.5.0] - 2021-12-27

* Ensure strings are ASCII (breaking change for modding)
* Fix incorrect image/alpha validation for palette images
* New FFI interface with WAV parsing support
* Figured out area partition values (breaking change)
* Split code into multiple crates for lower rebuild times
* General improvements to rebuild times
* Implement simple Windows-1252 decoder
* Implement PE (32 bit) parsing
* Bumped Rust edition to 2021

### [0.4.1] - 2021-02-03

* Allow arbitrary `.data` section/CRT initialization skip for message DLLs to support Recoil's `messages.dll` (hidden in CLI)
* Messages `--dump-ids`/JSON format changed to include the language ID - breaking change
* Better texture/image support for modding, e.g. automatically strip alpha channels, errors over panics (`rezbd`)

### [0.4.0] - 2020-12-23

* Pirate's Moon support for sounds, readers, motion, and textures
* To support Pirate's Moon textures, the `manifest.json` format has been slightly altered (`unzbd`, `rezbd`, C FFI lib) - breaking change
* Fixed off-by-one error when dumping messages (`Mech3Msg.dll`)
* Allow message table IDs to be dumped, for use with the [Mech3Msg](https://github.com/TerranMechworks/mech3msg) replacement project
* Blender script for GameZ worlds

### [0.3.2] - 2020-10-13

* Easier modding support for textures; don't require a ZIP and update image width and height automatically (`rezbd`)

### [0.3.1] - 2020-09-30

* Save memory by not pretty-printing JSON (C FFI lib)
* Write texture info manifest to callback (C FFI lib)

### [0.3.0] - 2020-09-29

* Assert and drop the last motion frame, as it's the same as the first (`motion`)
* Improvements to the Blender scripts for motion constraints and still chickenwalkers (Blender)
* Include Blender script in release archives
* Validate Empty, Lod, and Object3d node flags more strictly (`gamez`)
* Node flag 15 seems to be terrain (`gamez`) - breaking change
* Add C FFI library (`mech3ax.dll`/`libmech3ax.so`/`libmech3ax.dylib`)

### [0.2.0] - 2020-09-14

* Blender script for 'mechs (`mechlib`/Blender)
* Output all names from nodes, even if they have a fixed name (`gamez`)
* Write out `mesh_index`, and store mesh pointers separately (`mechlib`) - breaking change
* Lookup texture index from texture name instead of storing it (`gamez`)
* Calculate delta values instead of storing them in "Object Opacity From To" and "Fbfx Color From To" (`anim`)
* Unpack useful data in "Object Motion SI Script" (`anim`)

### [0.1.0] - 2020-09-12

* Initial release

## Development

[Rust](https://www.rust-lang.org/) is required, this project uses the `stable` toolchain.

This project uses [pre-commit](https://pre-commit.com/) to run `cargo fmt` when you commit (and only on the committed code, not unstaged code). If this sounds useful to you, install `pre-commit`, and put the hooks in place:

```bash
pre-commit install
```

## License

Licensed under the European Union Public Licence (EUPL) 1.2 ([LICENSE](LICENSE) or https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12).
