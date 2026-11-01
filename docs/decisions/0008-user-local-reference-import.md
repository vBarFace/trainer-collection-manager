# 0008: User-local reference acquisition

Date: 2026-11-01. Supersedes ADR 0006's bundled-only activation boundary.

The user explicitly selects an already acquired JSON source and identifies its
upstream revision. Native adapters normalize supported fields; the existing
schema-1 Catalog validator remains authoritative. Unsupported fields stay
unknown. No upstream data is bundled, committed or downloaded in the background.
Offline import is the first acquisition mechanism; an in-app downloader is
deferred until the provider-specific limits and error contracts are implemented.

Store normalized packages in the existing per-user SQLite database, not the
repository. Migration 9 adds active/previous slots, an immutable version digest
ledger, and cumulative species/form identity ledgers. Activation validates the
whole candidate and identity continuity, then rotates slots in one transaction.
The in-memory catalog changes only after commit. Rollback validates and swaps
the previous known package through the same transaction. A restart loads and
validates the active package. Corruption fails visibly rather than silently
substituting another dataset for pinned identities.

Keep two full packages; cap the immutable version ledger at 4,096 versions.
SHA-256 covers exact imported source bytes and canonical serialized packages.
Changing provider namespaces does not rebind confirmed UUIDs. Dataset version
changes are strategic context changes, independently of appraisal revisions.

`sha2` 0.10.9 is promoted from the already locked dependency graph to a direct
dependency for original-code provenance hashing (MIT OR Apache-2.0; package
license reviewed locally). Public data access and software licenses do not
resolve underlying game-data rights. Preserve upstream notices and describe
rights uncertainty in Settings and the source coverage matrix. Local acquisition
does not imply redistribution or unrestricted usage authorization.
