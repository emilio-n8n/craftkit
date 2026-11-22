#!/usr/bin/env python3
"""Generate a standalone CraftKit application from the reference app."""

from __future__ import annotations

import argparse
import json
import re
import shutil
import subprocess
import sys
from pathlib import Path

from launcher_icon import write_icon

CRAFTKIT_REPO = "https://github.com/emilio-n8n/craftkit"
DEFAULT_TAG = "v0.1.0"
SLUG_PATTERN = re.compile(r"^[a-z][a-z0-9-]*$")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Create a new CraftKit application in a directory."
    )
    parser.add_argument("slug", help="lowercase kebab-case application id")
    parser.add_argument("name", help='display name, for example "MusicCraft"')
    parser.add_argument("owner", nargs="?", default="emilio-n8n", help="GitHub owner")
    parser.add_argument("target", nargs="?", help="target directory (default: $HOME/<slug>)")
    parser.add_argument(
        "--tagline",
        default="Native Rust desktop application",
        help="one-line description used in the README and About window",
    )
    parser.add_argument(
        "--craftkit-tag",
        default=DEFAULT_TAG,
        help=f"CraftKit git tag to pin (default: {DEFAULT_TAG})",
    )
    parser.add_argument(
        "--publisher",
        help="publisher shown in craft-app.json (default: owner)",
    )
    parser.add_argument(
        "--bundle-id",
        help="reverse-DNS bundle id (default: io.github.<owner>.<slug>)",
    )
    parser.add_argument(
        "--color",
        default="#e8573f",
        help="brand colour for the icon and launcher card (default: #e8573f)",
    )
    return parser.parse_args()


def fail(message: str) -> None:
    print(f"error: {message}", file=sys.stderr)
    raise SystemExit(1)


def replace_text(path: Path, replacements: list[tuple[str, str]]) -> None:
    text = path.read_text(encoding="utf-8")
    for old, new in replacements:
        text = text.replace(old, new)
    path.write_text(text, encoding="utf-8")


def main() -> None:
    args = parse_args()
    if not SLUG_PATTERN.match(args.slug):
        fail("slug must start with a letter and contain only lowercase letters, digits and hyphens")
    if not args.name.strip():
        fail("display name must not be empty")

    root = Path(__file__).resolve().parent.parent
    demo = root / "apps" / "craftkit-demo"
    templates = root / "templates" / "app"
    if not demo.is_dir() or not templates.is_dir():
        fail("run the generator from a complete CraftKit checkout")

    target = (
        Path(args.target).expanduser()
        if args.target
        else Path.home() / args.slug
    )
    target = target.resolve()
    if target.exists():
        entries = [entry for entry in target.iterdir() if entry.name != ".git"]
        if entries:
            fail(f"target directory is not empty: {target}")
    else:
        target.mkdir(parents=True)

    for item in demo.iterdir():
        if item.name == "target":
            continue
        destination = target / item.name
        if item.is_dir():
            shutil.copytree(item, destination)
        else:
            shutil.copy2(item, destination)

    for name in ("LICENSE-MIT", "LICENSE-APACHE", "rustfmt.toml", ".gitignore"):
        shutil.copy2(root / name, target / name)

    mit = target / "LICENSE-MIT"
    mit.write_text(
        mit.read_text(encoding="utf-8").replace(
            "CraftKit contributors", f"{args.name} contributors"
        ),
        encoding="utf-8",
    )

    publisher = args.publisher or args.owner
    owner_slug = re.sub(r"[^a-z0-9]", "", args.owner.lower()) or "app"
    bundle_id = args.bundle_id or f"io.github.{owner_slug}.{args.slug}"
    common = [
        ("__APP__", args.name),
        ("__SLUG__", args.slug),
        ("__OWNER__", args.owner),
        ("__TAGLINE__", args.tagline),
        ("__CRAFTKIT_TAG__", args.craftkit_tag),
        ("__PUBLISHER__", publisher),
        ("__BUNDLE_ID__", bundle_id),
        ("__COLOR__", args.color),
    ]
    for template_name, output_name in (
        ("README.md.tmpl", "README.md"),
        ("AGENTS.md.tmpl", "AGENTS.md"),
        ("THIRD_PARTY.md.tmpl", "THIRD_PARTY.md"),
        ("CHANGELOG.md.tmpl", "CHANGELOG.md"),
        ("LAUNCHER.md.tmpl", "LAUNCHER.md"),
        ("decisions.md.tmpl", "docs/decisions.md"),
    ):
        source = templates / template_name
        destination = target / output_name
        destination.parent.mkdir(parents=True, exist_ok=True)
        text = source.read_text(encoding="utf-8")
        for old, new in common:
            text = text.replace(old, new)
        destination.write_text(text, encoding="utf-8")

    manifest_data = {
        "schema": 1,
        "id": args.slug,
        "name": args.name,
        "tagline": args.tagline,
        "repo": f"{args.owner}/{args.slug}",
        "publisher": publisher,
        "upstream": None,
        "bundle_id": bundle_id,
        "msi_publisher": publisher,
        "apple_team": None,
        "website": f"https://github.com/{args.owner}/{args.slug}",
        "color": args.color,
        "icon": f"{args.slug}-128.png",
    }
    (target / "craft-app.json").write_text(
        json.dumps(manifest_data, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )
    write_icon(target / "assets" / f"{args.slug}-128.png", args.color)

    tools = target / "tools"
    tools.mkdir(parents=True, exist_ok=True)
    shutil.copy2(root / "tools" / "launcher_check.py", tools / "launcher_check.py")

    workflows = target / ".github" / "workflows"
    workflows.mkdir(parents=True, exist_ok=True)
    for template_name in ("ci.yml", "release.yml"):
        text = (templates / template_name).read_text(encoding="utf-8")
        for old, new in common:
            text = text.replace(old, new)
        (workflows / template_name).write_text(text, encoding="utf-8")

    manifest = target / "Cargo.toml"
    text = manifest.read_text(encoding="utf-8")
    text = re.sub(r'^name = "craftkit-demo"', f'name = "{args.slug}"', text, flags=re.MULTILINE)
    text = re.sub(
        r'^description = ".*"',
        f'description = "{args.name} — native Rust desktop application"',
        text,
        flags=re.MULTILINE,
    )
    text = text.replace("rust-version.workspace = true", 'rust-version = "1.95"')
    text = text.replace("version.workspace = true", 'version = "0.1.0"')
    text = text.replace("edition.workspace = true", 'edition = "2024"')
    text = text.replace("license.workspace = true", 'license = "MIT OR Apache-2.0"')
    text = text.replace(
        "repository.workspace = true",
        f'repository = "https://github.com/{args.owner}/{args.slug}"',
    )
    text = text.replace(
        "authors.workspace = true",
        f'authors = ["{args.name} contributors"]',
    )
    for crate in ("core", "io", "jobs", "ui"):
        text = text.replace(
            f"craftkit-{crate}.workspace = true",
            f'craftkit-{crate} = {{ git = "{CRAFTKIT_REPO}", tag = "{args.craftkit_tag}" }}',
        )
    text = text.replace(
        "serde.workspace = true",
        'serde = { version = "1", features = ["derive"] }',
    )
    manifest.write_text(text, encoding="utf-8")

    source_replacements = [
        ("craftkit-demo", args.slug),
        ("CraftKit Demo", args.name),
        (
            "Reference notes application",
            args.tagline,
        ),
        (
            "https://github.com/emilio-n8n/craftkit",
            f"https://github.com/{args.owner}/{args.slug}",
        ),
    ]
    for path in (target / "src").rglob("*.rs"):
        replace_text(path, source_replacements)

    if not (target / ".git").exists():
        subprocess.run(["git", "init", "-b", "main"], cwd=target, check=False)
    remote = f"https://github.com/{args.owner}/{args.slug}.git"
    current_remote = subprocess.run(
        ["git", "remote", "get-url", "origin"],
        cwd=target,
        capture_output=True,
        text=True,
        check=False,
    )
    if current_remote.returncode != 0:
        subprocess.run(["git", "remote", "add", "origin", remote], cwd=target, check=False)

    print(f"Created {args.name} in {target}")
    print(f"Remote: {remote}")
    print("Next steps:")
    print(f"  cd {target}")
    print("  cargo generate-lockfile")
    print("  cargo fmt --all")
    print("  cargo test --workspace")


if __name__ == "__main__":
    main()
