#!/usr/bin/env python3
"""Builds a resource pack with Create's shaft and cogwheel models, from a Create jar you downloaded yourself.

    python3 tools/create_look.py ~/Downloads/create-1.21.1-6.0.10.jar

Create exists for Minecraft 1.21.1 (NeoForge) at most; its block models and PNG textures still load in 26.3.
The pack holds Create's own models (create:block/shaft, cogwheel, large_cogwheel) and only the textures they use.
Horizon lays a mod's files over a resource pack's, so the pack can't replace Kinetics' models: the Kinetics
native mod draws Create's models when they exist and its own otherwise.
Create's assets are All Rights Reserved: the pack is for your own game, never share or commit it.
Default output: the Horizon game folder's resourcepacks/kinetics-create-look.zip.
"""
import argparse, json, pathlib, re, sys, zipfile

# Create models the Kinetics mod draws first (runtime.rs LOOK_MODELS), all built along the y axis.
MODELS = ["shaft", "cogwheel", "large_cogwheel"]
PACK_FORMAT = [97, 1]  # Minecraft 26.3 (horizon-core version.rs RESOURCE_PACK_FORMAT)


def ref(name):
    """'create:block/x' or 'block/x' -> (namespace, path)."""
    ns, _, path = name.rpartition(":")
    return (ns or "minecraft"), path


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("jar", type=pathlib.Path, help="create-1.21.1-*.jar")
    ap.add_argument("--out", type=pathlib.Path,
                    default=pathlib.Path.home() / ".local/share/horizon/resourcepacks/kinetics-create-look.zip")
    a = ap.parse_args()

    try:
        jar = zipfile.ZipFile(a.jar)
    except (OSError, zipfile.BadZipFile) as e:
        sys.exit(f"{a.jar}: not a jar ({e})")
    names = set(jar.namelist())
    toml = jar.read("META-INF/neoforge.mods.toml").decode() if "META-INF/neoforge.mods.toml" in names else ""
    if not re.search(r'modId\s*=\s*"create"', toml):
        sys.exit(f"{a.jar}: not Create for NeoForge (no modId \"create\" in META-INF/neoforge.mods.toml)")
    if "1.21.1" not in a.jar.name:
        print(f"warning: {a.jar.name} doesn't say 1.21.1; other versions' models may differ", file=sys.stderr)

    files = {}  # path in the pack -> bytes

    def model(name):
        ns, path = ref(name)
        if ns != "create":
            return  # vanilla's own (block/block): the game has it
        entry = f"assets/create/models/{path}.json"
        if entry in files:
            return
        if entry not in names:
            sys.exit(f"{a.jar.name} has no {entry}: another Create version?")
        data = jar.read(entry)
        files[entry] = data
        m = json.loads(data)
        if "loader" in m:
            print(f"warning: {entry} uses NeoForge loader {m['loader']!r}, which Horizon may not draw", file=sys.stderr)
        for tex in m.get("textures", {}).values():
            if isinstance(tex, str) and not tex.startswith("#"):
                texture(tex)
        if "parent" in m:
            model(m["parent"])

    def texture(name):
        ns, path = ref(name)
        if ns != "create":
            return
        for ext in (".png", ".png.mcmeta"):
            entry = f"assets/create/textures/{path}{ext}"
            if entry in names:
                files[entry] = jar.read(entry)
        if f"assets/create/textures/{path}.png" not in names:
            print(f"warning: {a.jar.name} has no texture {name}", file=sys.stderr)

    MODELS.extend(["creative_motor/block", "creative_motor/block_vertical", "shaft_half"])  # motor.rs LOOK_MODELS
    for name in MODELS:
        model(f"create:block/{name}")
    files["pack.mcmeta"] = json.dumps({"pack": {
        "description": "Kinetics with Create's look (from your own Create jar; don't share)",
        "min_format": PACK_FORMAT, "max_format": PACK_FORMAT}}, indent=2).encode()

    a.out.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(a.out, "w", zipfile.ZIP_DEFLATED) as z:
        for path in sorted(files):
            z.writestr(path, files[path])
    print(f"{a.out}: {len(files)} files. Turn it on in Options > Resource Packs.")


if __name__ == "__main__":
    main()
