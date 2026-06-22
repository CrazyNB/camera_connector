# Documentation Index

This directory contains the current product, architecture, protocol,
development, and validation documentation for Camera Connector.

## Current Source Of Truth

- `architecture.md`: semantic module boundaries and file ownership.
- `product/PRD.md`: current product route, scope, requirements, and milestones.
- `protocol.md`: FTP push behavior, storage rules, transfer log, device status,
  and receiver runtime metadata.
- `development.md`: local toolchain, verification commands, and CLI workflows.
- `compatibility.md`: real-camera and Android physical-device validation
  records.
- `troubleshooting.md`: receiver, login, upload, publish, and diagnostics triage.
- `product/android-app-architecture.md`: Android shell, gateway, foreground
  service, storage, and native packaging.
- `product/mobile-app-handoff.md`: mobile implementation surface and handoff
  notes.
- `product/prototype-spec.md`: prototype navigation and data contract alignment.

## Historical Plans And Specs

`docs/superpowers/specs/` and `docs/superpowers/plans/` are implementation
history. They are useful for understanding why decisions were made, but they are
not automatically current. When current behavior and an older plan disagree,
prefer the code plus the current source-of-truth documents above.

## Documentation Update Rule

When changing product behavior or module ownership:

1. Update `architecture.md` when a semantic boundary or owning module changes.
2. Update `product/PRD.md` when scope, non-goals, requirements, or milestones
   change.
3. Update `protocol.md` when receiver, transfer, storage, or runtime metadata
   contracts change.
4. Update app-specific docs when platform behavior changes.
5. Update `compatibility.md` after every real-camera or physical-device test.
6. Update `troubleshooting.md` when a recurring failure gains a stable diagnosis
   or recovery path.
