# Frozen documentation snapshots

Each subdirectory is a read-only copy of `docs/content` from a published tag.
Later builds must not rewrite these trees. Recreate a snapshot with:

```bash
node scripts/snapshot-docs-version.mjs --tag v2.90.0 --prefix 2.90
```
