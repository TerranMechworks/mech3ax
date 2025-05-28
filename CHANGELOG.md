# Changelog

## Unreleased

* GameZ RC only: Refer to meshes as models. Almost all model fields understood. (`gamez`, RC)
* Move fields from AnimFileName to AnimDef; AnimFileName is now only a list of the animation definition files in the archive.
* Rename AnimPtr to AnimFileName
* ObjectRef and NodeRef fields
* SeqDef fields
* Save SI scripts separately

## [0.6.1] - 2024-11-28

* Update to Rust 1.83.0
* Better logging
* Change/unify date-time stuff
* Support `u64` in exchange protocol
* ZArchive field `garbage` is sometimes a comment and timestamp (zarchive, breaking change)
* PM/CS GameZ field `unk08` is a timestamp (`gamez`, breaking change)
* Remove duplicate reset state ptr (`anim`, breaking change)
* Fix `z_mid` issue for MW3 US v1.0/v1.1
* Materials field 32/`specular` is actually soil (`gamez`, breaking change)

## [0.6.0] - 2023-02-10

* Horribly dirty hacks for Recoil M6 and M9 (`gamez`)
* Allow Recoil light nodes in other positions (`gamez`)
* Blender scripts removed
* Update to Rust 1.76.0

## [0.6.0-rc4] - 2023-07-15

* API type updates (codegen)
* C# partial classes support (codegen)
* Map Rust module/C# namespace structure to directories (codegen)
* Deduplicate GameZ texture names for Crimson Skies, because `planes.zbd` has some duplicates
* Add zmap reading and writing functions (`lib`)
* Rename reader JSON reading and writing functions (`lib`, breaking change)
* Split C# struct information from Rust type information (codegen)
* Update to Rust 1.71.0

## [0.6.0-rc3] - 2023-05-30

* Serialize C# enums as enums, not classes (codegen)
* Support Recoil GameZ nodes (`lib`/`unzbd`/`rezbd`, breaking change)
* Support Pirate's Moon GameZ nodes (`lib`/`unzbd`/`rezbd`, breaking change)
* Support Crimson Skies GameZ nodes (`lib`/`unzbd`/`rezbd`, breaking change)
* Validate material indices, which renamed "texture index" to the more accurate "material index" (breaking change)

## [0.6.0-rc2] - 2022-11-29

* Implement custom data exchange format to replace JSON (`lib`)
* Use custom data exchange format for C# (`lib`, breaking change)
* Update C# code generation for data exchange format (gen)
* Remove old v1 `anim` API (`lib`)

## [0.6.0-rc1] - 2022-11-20

Big features:

* Introduce API types crates, to clarify external structures
* Implement Rust structures to C# structures code generation
* Updated CLI and library to support multiple games
* Support files from Recoil, Pirate's Moon, and Crimson Skies

Detailed changes:

* Support Recoil zmaps (`unzbd`/`rezbd`)
* Support Crimson Skies' textures (`unzbd`/`rezbd`)
* Support Crimson Skies' `strings.dll` (`unzbd`)
* Detailed logging for many operations
* Display helpers for displaying the raw C data structures
* Update unzbd/rezbd CLI to support multiple games (`unzbd`/`rezbd`, breaking change)
* Removed `--dump-ids` flag (`unzbd`, breaking change)
* `AnimDef` contains reset state seq def - thanks Skyfaller (`anim`, breaking change)
* Update Blender scripts to match breaking API changes
* Support writing anim.zbd (`lib`)
* Split `PrereqObject` into that and `PrereqParent` (`anim`, breaking change)
* Add event start struct for `Event` (`anim`, breaking change)
* Add condition structures for `If`/`ElseIf` (`anim`, breaking change)
* Add scale struct for `ObjectMotion` (`anim`, breaking change)
* Add xyz rotation struct for `ObjectMotion` (`anim`, breaking change)
* Add translation struct for `ObjectMotion` (`anim`, breaking change)
* Add opacity value/state struct for `ObjectOpacityFromTo` (`anim`, breaking change)
* Add bounce sequence struct for `ObjectMotion` (`anim`, breaking change)
* Add struct for `ActivationPrereq::Animation` variant (`anim`, breaking change)
* Convert `PufferState` cycle textures from tuple to struct (`anim`, breaking change)
* Convert `CallAnimationParameters` and `ForwardRotation` enum fields to external structs (`anim`, breaking change)
* Many changes to public anim structures (`anim`, breaking change)
* Add reader raw/passthrough functions to mech3ax-lib (`lib`)
* Remove old mech3ax-lib v1 API except anim (`lib`, breaking change)
* Convert `Vec3` tuple to structure (breaking change)
* Convert `Matrix` tuple to structure (breaking change)
* Convert `Block` tuple to `BoundingBox` structure (`mechlib`/`gamez`, breaking change)
* Add `MessageEntry` structure instead of tuple (`messages`, breaking change)
* Serialize global palette data as base64 (`image`, breaking change)
* Add variant structures for `TexturePalette` (`image`, breaking change)
* Add `MotionPart` structure between `Motion` and `MotionFrame` (`motion`, breaking change)
* Convert remaining `Vec4` uses to `Quaternion` (breaking change)
* Destructure resolution tuple (`gamez`, breaking change)
* Convert one `Vec4` use to `Rgba` (`anim`, breaking change)
* Convert some `Vec3` uses to `Color` (breaking change)
* Convert remaining `Vec2` uses to `UvCoord` (`mechlib`/`gamez`, breaking change)
* Convert most `Vec2` uses to `Range` (breaking change)
* Convert `Area` into a named structure (`gamez`, breaking change)
* Convert `AreaPartition` into a named structure (`gamez`, breaking change)
* Introduce API types crates, to clarify external structures
* Fix possible undefined behaviour (UB) in `read_struct`
* Implement JSON to ZRD conversion (`rezbd`)
* AnimDef field 152 is likely the status (`anim`, breaking change)
* ReaderLookup field 40 is likely the "in use" or "loaded into world" flag (`anim`, breaking change)
* AnimRef field 68 is likely a pointer (`anim`, breaking change)
* Remove ResetState structure, it's likely SeqDefInfo instead (`anim`, breaking change)
* Make `--dump-ids` the default mode for `unzbd messages` (`unzbd`, breaking change)

## [0.5.0] - 2021-12-27

* Ensure strings are ASCII (breaking change for modding)
* Fix incorrect image/alpha validation for palette images
* New FFI interface with WAV parsing support
* Figured out area partition values (breaking change)
* Split code into multiple crates for lower rebuild times
* General improvements to rebuild times
* Implement simple Windows-1252 decoder
* Implement PE (32 bit) parsing
* Bumped Rust edition to 2021

## [0.4.1] - 2021-02-03

* Allow arbitrary `.data` section/CRT initialization skip for message DLLs to support Recoil's `messages.dll` (hidden in CLI)
* Messages `--dump-ids`/JSON format changed to include the language ID - breaking change
* Better texture/image support for modding, e.g. automatically strip alpha channels, errors over panics (`rezbd`)

## [0.4.0] - 2020-12-23

* Pirate's Moon support for sounds, readers, motion, and textures
* To support Pirate's Moon textures, the `manifest.json` format has been slightly altered (`unzbd`, `rezbd`, C FFI lib) - breaking change
* Fixed off-by-one error when dumping messages (`Mech3Msg.dll`)
* Allow message table IDs to be dumped, for use with the [Mech3Msg](https://github.com/TerranMechworks/mech3msg) replacement project
* Blender script for GameZ worlds

## [0.3.2] - 2020-10-13

* Easier modding support for textures; don't require a ZIP and update image width and height automatically (`rezbd`)

## [0.3.1] - 2020-09-30

* Save memory by not pretty-printing JSON (C FFI lib)
* Write texture info manifest to callback (C FFI lib)

## [0.3.0] - 2020-09-29

* Assert and drop the last motion frame, as it's the same as the first (`motion`)
* Improvements to the Blender scripts for motion constraints and still chickenwalkers (Blender)
* Include Blender script in release archives
* Validate Empty, Lod, and Object3d node flags more strictly (`gamez`)
* Node flag 15 seems to be terrain (`gamez`) - breaking change
* Add C FFI library (`mech3ax.dll`/`libmech3ax.so`/`libmech3ax.dylib`)

## [0.2.0] - 2020-09-14

* Blender script for 'mechs (`mechlib`/Blender)
* Output all names from nodes, even if they have a fixed name (`gamez`)
* Write out `mesh_index`, and store mesh pointers separately (`mechlib`) - breaking change
* Lookup texture index from texture name instead of storing it (`gamez`)
* Calculate delta values instead of storing them in "Object Opacity From To" and "Fbfx Color From To" (`anim`)
* Unpack useful data in "Object Motion SI Script" (`anim`)

## [0.1.0] - 2020-09-12

* Initial release
