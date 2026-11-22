# ArtCraft Launcher metadata

CraftKit applications can be described to the ArtCraft Launcher. There are two
different onboarding paths, and they should not be confused:

1. **Approved community app** — the launcher maintainers review the project and
   pin it in the launcher catalog (as they did for SolidCraft and MailCraft).
   This needs a launcher release and, on macOS, a matching Developer ID team.
2. **Unofficial app via `craft-app.json`** — the proposal in
   `storytold/craft-launcher` issue #989. The user adds the repository in the
   launcher settings. The app is shown as unofficial, without review or
   endorsement; the launcher only enforces the release contract.

The descriptor is data. The launcher never runs third-party code inside its own
process.

## Descriptor

`craft-app.json` lives at the repository root and is capped at 16 KiB:

```json
{
  "schema": 1,
  "id": "diagramcraft",
  "name": "DiagramCraft",
  "tagline": "Diagram editor for shapes, connectors and SVG export",
  "repo": "emilio-n8n/diagramcraft",
  "publisher": "emilio-n8n",
  "upstream": null,
  "bundle_id": "io.github.emilion8n.diagramcraft",
  "msi_publisher": "emilio-n8n",
  "apple_team": null,
  "website": "https://github.com/emilio-n8n/diagramcraft",
  "color": "#e8573f",
  "icon": "diagramcraft-128.png"
}
```

Rules enforced by `tools/launcher_check.py`:

- `id` matches `^[a-z0-9-]+$` and equals the repository name;
- `repo` is the exact source repository;
- `bundle_id` does not start with `ai.storyteller.`;
- `publisher` and `msi_publisher` do not claim Learning Machines LLC / ArtCraft;
- `apple_team` is `null` or a team other than ArtCraft's `DJ6XS33FX8`;
- the icon is a 128×128 PNG in `assets/` and attached to releases as
  `<id>-128.png`;
- release files follow `<id>-<version>-<os>-<arch>.<ext>` and every file is
  covered by `SHA256SUMS.txt`.

## Release contract

The launcher selects packages by OS and architecture. Supported names include:

```text
diagramcraft-0.1.0-linux-x86_64.AppImage
diagramcraft-0.1.0-linux-x86_64.tar.gz
diagramcraft-0.1.0-linux-x86_64.deb
diagramcraft-0.1.0-linux-x86_64.rpm
diagramcraft-0.1.0-windows-x64.zip
diagramcraft-0.1.0-windows-x64.msi
diagramcraft-0.1.0-macos-aarch64.dmg
diagramcraft-0.1.0-macos-x86_64.dmg
craft-app.json
diagramcraft-128.png
SHA256SUMS.txt
```

`SHA256SUMS.txt` must cover every file above. The launcher verifies it before
running or installing anything.

## Packaging and signing

- **Linux**: AppImage is preferred; tar.gz is an accepted fallback. deb/rpm are
  useful for users who prefer a package manager.
- **Windows**: MSI is preferred; a portable ZIP is the fallback. An MSI needs an
  Authenticode certificate and its publisher must match `msi_publisher`.
- **macOS**: DMG is the launcher's preferred package and needs a Developer ID
  signature plus notarization to install. With `apple_team: null`, the launcher
  will not install a DMG; it can still open an existing copy or use a portable
  archive manually.

No certificate is required for local builds or Linux portable archives. Do not
claim an approved community app until the maintainers have reviewed it.

## Checklist

- [ ] `craft-app.json` and the 128×128 icon are committed.
- [ ] `python3 tools/launcher_check.py .` passes.
- [ ] A release contains archives, manifest, icon and `SHA256SUMS.txt`.
- [ ] `python3 tools/launcher_check.py . --release-dir dist` passes.
- [ ] The release was downloaded and launched on a real machine.
- [ ] Canary: no official namespace, no official publisher, no ArtCraft team.
- [ ] Submission issue/PR or Discord message prepared, with a short demo.

See [templates/app/LAUNCHER.md.tmpl](../templates/app/LAUNCHER.md.tmpl) for the
generated application copy.
