# ADR 0002: Local-first and least privilege

**Status:** Accepted

The foundation grants no broad filesystem, shell, process, network or SQL-plugin permissions to the UI. The native layer initializes only its own database directory.

This makes privacy and offline operation the default. Future integrations require explicit design review, typed interfaces and user-visible consent where appropriate.

## Task 003 implementation

The Inventory workflow now exposes a native single-file picker and bounded PNG/JPEG preview behind a main-window command capability. The renderer cannot provide paths or call arbitrary reads. Images are processed temporarily; explicit confirmation persists only reviewed structured fields and tags atomically in the existing per-user database. No broad filesystem, network, SQL or dialog plugin permission was added. See [manual review boundaries](../development/screenshot-review.md).
