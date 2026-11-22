#!/usr/bin/env python3
"""Add the launcher pack to an existing CraftKit application."""

from __future__ import annotations

import argparse
import json
import shutil
from pathlib import Path

from launcher_icon import write_icon

ROOT = Path(__file__).resolve().parent.parent


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("app_dir", type=Path)
    parser.add_argument("app_id")
    parser.add_argument("name")
    parser.add_argument("tagline")
    parser.add_argument("--owner", default="emilio-n8n")
    parser.add_argument("--publisher", default="emilio-n8n")
    parser.add_argument("--color", default="#e8573f")
    parser.add_argument("--bundle-id", default=None)
    args = parser.parse_args()

    app_dir = args.app_dir.resolve()
    if not (app_dir / "Cargo.toml").is_file():
        raise SystemExit(f"not a CraftKit app: {app_dir}")

    owner_slug = "".join(character for character in args.owner.lower() if character.isalnum())
    bundle_id = args.bundle_id or f"io.github.{owner_slug}.{args.app_id}"

    manifest = {
        "schema": 1,
        "id": args.app_id,
        "name": args.name,
        "tagline": args.tagline,
        "repo": f"{args.owner}/{args.app_id}",
        "publisher": args.publisher,
        "upstream": None,
        "bundle_id": bundle_id,
        "msi_publisher": args.publisher,
        "apple_team": None,
        "website": f"https://github.com/{args.owner}/{args.app_id}",
        "color": args.color,
        "icon": f"{args.app_id}-128.png",
    }
    (app_dir / "craft-app.json").write_text(
        json.dumps(manifest, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )
    write_icon(app_dir / "assets" / f"{args.app_id}-128.png", args.color)

    tools = app_dir / "tools"
    tools.mkdir(parents=True, exist_ok=True)
    shutil.copy2(ROOT / "tools" / "launcher_check.py", tools / "launcher_check.py")

    replacements = [
        ("__APP__", args.name),
        ("__SLUG__", args.app_id),
        ("__OWNER__", args.owner),
        ("__TAGLINE__", args.tagline),
        ("__PUBLISHER__", args.publisher),
        ("__BUNDLE_ID__", bundle_id),
        ("__COLOR__", args.color),
    ]
    launcher = (ROOT / "templates" / "app" / "LAUNCHER.md.tmpl").read_text(encoding="utf-8")
    for old, new in replacements:
        launcher = launcher.replace(old, new)
    (app_dir / "LAUNCHER.md").write_text(launcher, encoding="utf-8")

    workflows = app_dir / ".github" / "workflows"
    workflows.mkdir(parents=True, exist_ok=True)
    for template_name in ("ci.yml", "release.yml"):
        text = (ROOT / "templates" / "app" / template_name).read_text(encoding="utf-8")
        for old, new in replacements:
            text = text.replace(old, new)
        (workflows / template_name).write_text(text, encoding="utf-8")

    print(f"launcher pack applied to {args.name} ({app_dir})")


if __name__ == "__main__":
    main()
