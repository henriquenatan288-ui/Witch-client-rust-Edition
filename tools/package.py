#!/usr/bin/env python3
"""Create a deterministic, source-only Witch Client .hnmod for Horizon."""
from __future__ import annotations

import hashlib
import json
import tomllib
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MOD_ID = "witch"


def main() -> None:
    compatibility = json.loads((ROOT / "compatibility.json").read_text(encoding="utf-8"))
    mixin_path = ROOT / "horizon-mixin.toml"
    mixin = tomllib.loads(mixin_path.read_text(encoding="utf-8"))
    mod = mixin["mod"]

    if mod["id"] != MOD_ID:
        raise SystemExit("horizon-mixin.toml has an unexpected mod id")
    if mixin["compatibility"]["horizon_revision"] != compatibility["horizon_revision"]:
        raise SystemExit("Horizon pin mismatch between compatibility.json and horizon-mixin.toml")

    manifest = {
        "schema": 1,
        "id": MOD_ID,
        "version": mod["version"],
        "game": compatibility["game"],
        "horizon": compatibility["horizon_revision"],
        "mixins": compatibility["horizon_mixins_revision"],
        "hooks": compatibility["hooks"],
    }
    files: dict[str, bytes] = {
        "horizon-native.json": (json.dumps(manifest, indent=2, sort_keys=True) + "\n").encode("utf-8"),
        f"mixins/{MOD_ID}/horizon-mixin.toml": mixin_path.read_bytes(),
    }

    # Ship only Rust/TOML sources and supported crate data files.
    for crate_name in ("engine", "witchcore"):
        crate_root = ROOT / crate_name
        if not crate_root.is_dir():
            raise SystemExit(f"missing crate directory: {crate_name}")
        for path in sorted(crate_root.rglob("*")):
            if not path.is_file():
                continue
            relative = path.relative_to(ROOT)
            parts = relative.parts
            is_source = path.suffix in {".rs", ".toml"}
            is_data = len(parts) >= 2 and (
                parts[0] == "data" or (len(parts) >= 3 and parts[1] == "data")
            )
            if is_source or is_data:
                archive_path = f"mixins/{MOD_ID}/{relative.as_posix()}"
                files[archive_path] = path.read_bytes()

    required = [
        f"mixins/{MOD_ID}/engine/Cargo.toml",
        f"mixins/{MOD_ID}/engine/src/lib.rs",
        f"mixins/{MOD_ID}/engine/data/frame.rgba",
        f"mixins/{MOD_ID}/witchcore/Cargo.toml",
        f"mixins/{MOD_ID}/witchcore/src/lib.rs",
    ]
    missing = [name for name in required if name not in files]
    if missing:
        raise SystemExit("package is missing required files: " + ", ".join(missing))

    out_dir = ROOT / "target"
    out_dir.mkdir(parents=True, exist_ok=True)
    out_file = out_dir / f"{MOD_ID}-{mod['version']}.hnmod"
    with zipfile.ZipFile(out_file, "w", compression=zipfile.ZIP_DEFLATED) as archive:
        for name, content in sorted(files.items()):
            info = zipfile.ZipInfo(name, (2026, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o100644 << 16
            archive.writestr(info, content)

    with zipfile.ZipFile(out_file, "r") as archive:
        bad_file = archive.testzip()
        if bad_file is not None:
            raise SystemExit(f"invalid ZIP member: {bad_file}")
        actual = set(archive.namelist())
        if actual != set(files):
            raise SystemExit("archive contents differ from the package manifest")

    digest = hashlib.sha256(out_file.read_bytes()).hexdigest()
    print(json.dumps({
        "file": str(out_file.relative_to(ROOT)),
        "size": out_file.stat().st_size,
        "sha256": digest,
        "entries": len(files),
        "horizon": manifest["horizon"],
        "hooks": manifest["hooks"],
    }, indent=2))


if __name__ == "__main__":
    main()
