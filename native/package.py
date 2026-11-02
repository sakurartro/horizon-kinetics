#!/usr/bin/env python3
"""Packs mixins/kinetics into target/kinetics-<version>.hnmod (docs/api/mod-files.md in Horizon)."""
import json, pathlib, tomllib, zipfile

here = pathlib.Path(__file__).parent
bundle = here / "mixins" / "kinetics"
manifest = tomllib.loads((bundle / "horizon-mixin.toml").read_text())
compat = json.loads((here / "compatibility.json").read_text())
mod = manifest["mod"]
native = {
    "schema": 1,
    "id": mod["id"],
    "version": mod["version"],
    "game": compat["game"],
    "horizon": manifest["compatibility"]["horizon_revision"],
    "mixins": compat["horizon_mixins_revision"],
}
out = here / "target" / f"{mod['id']}-{mod['version']}.hnmod"
out.parent.mkdir(exist_ok=True)
with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as z:
    z.writestr("horizon-native.json", json.dumps(native, indent=2))
    for f in sorted(bundle.iterdir()):
        if f.suffix in (".rs", ".toml"):
            z.write(f, f"mixins/{mod['id']}/{f.name}")
print(out)
