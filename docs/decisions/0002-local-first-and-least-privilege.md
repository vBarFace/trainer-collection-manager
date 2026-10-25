# ADR 0002: Local-first and least privilege

**Status:** Accepted

The foundation grants no broad filesystem, shell, process, network or SQL-plugin permissions to the UI. The native layer initializes only its own database directory.

This makes privacy and offline operation the default. Future integrations require explicit design review, typed interfaces and user-visible consent where appropriate.
