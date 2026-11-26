---
name: bambu-studio-projects
description: Use when generating Bambu Studio .3mf project files from code (as crates/amsat-kit's bambu module does): one object per placed part, named plates with previews, per-object settings, no embedded printer profile, a deterministic zip and a read-back check.
---

# Bambu Studio projects from code

Ship print files that open in Bambu Studio (File → Open Project) with every part already placed, named and set, on any Bambu printer with a 256 mm bed. The reference is `crates/amsat-kit/src/bambu.rs` in the cubesat-classroom repository, which follows Bambu Studio's own `bbs_3mf.cpp`.

## Layout of the archive

- `[Content_Types].xml`, `_rels/.rels`, `3D/3dmodel.model`, `3D/_rels/3dmodel.model.rels`, then everything else sorted.
- **One object per placed part.** Each mesh goes in its own `3D/Objects/object_N.model`, and the main model refers to it as a component. Bambu Studio keeps one instance per object per plate, so copies of one object can't be spread across plates; repeated parts become numbered objects.
- **Placing parts on plates:**
  - Build items carry each part's translation.
  - Plates are laid out the way Bambu Studio computes them: plate stride 256 × 1.2 mm; columns = round(√count), plus 1 if √count is above that.
- **`Metadata/model_settings.config`:**
  - per object: name and extruder, plus per-object settings such as `sparse_infill_density` at 100% for flexing coupons;
  - per plate: id, name, `thumbnail_file`, and its model instances (`identify_id`).
- **Previews:** `Metadata/plate_N.png` (512 px) and `plate_N_small.png` (128 px), rendered from the meshes (an isometric z-buffer is enough).
- **No printer, process or filament profile.** Bambu Studio keeps whatever the user has selected. Say in the README which process and material to pick.

## Kits

Describe each kit as data: parts (name, mesh source, flip, solid infill, preview colour) and projects (file, title, plates of rows of part names), plus a description string. A part can reuse another's mesh under a new name, such as the same columns piece printed as both top and bottom.

## Determinism and checks

- **Fixed output:** fixed timestamps (1980-01-01), fixed deflate level, fixed number formatting (6 decimals, trailing zeros stripped), sorted entries.
- **Read back what you wrote:** unzip it, parse the model and config, and check that every object has exactly one instance, on exactly one plate, inside that plate's area.
- **Layout check:** if a row doesn't fit the 256 mm plate with an 8 mm margin, fail the build rather than ship an overlapping plate.
- **Verify the zip** with a standard tool (`unzip -t`).

## Plates people can print

- **Group by material:** for example ASA box; ASA lid and plate; TPU gasket. Say the material in each plate's name.
- **Coupons first:** a separate project of test coupons, printed before the real parts.
- **Keep plates small** enough that a failed print wastes little plastic.
