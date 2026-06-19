# Test Fixtures — `src-tauri/tests/fixtures/`

Binary/text fixtures used by `cargo test` for the icon cache & ZIP-reading
regression tests. Resolved from `env!("CARGO_MANIFEST_DIR")/tests/fixtures`.

## Inventory

| File | Purpose |
|------|---------|
| `dummy.zip` | Valid ZIP with 2 entries (`hello.txt`, `sub/nested.txt`) — used by `extract_zip_entry` happy-path test. |
| `malformed.bin` | 256 random bytes (seeded) — NOT a valid ZIP. Used by `extract_zip_entry` error-path test. |

## Safety

- All fixtures are **synthetic** — no real APK, no copyrighted app icons, no
  real device identifiers (MAC/serial). (PRD NFR-6)
- `malformed.bin` is deterministic (Python `random.seed(42)`).
- No fixture reads environment variables or performs network access.

## Regeneration

```bash
python -c "
import zipfile, random
with zipfile.ZipFile('src-tauri/tests/fixtures/dummy.zip', 'w', zipfile.ZIP_DEFLATED) as z:
    z.writestr('hello.txt', 'hello')
    z.writestr('sub/nested.txt', 'world')
random.seed(42)
with open('src-tauri/tests/fixtures/malformed.bin', 'wb') as f:
    f.write(bytes(random.randint(0,255) for _ in range(256)))
"
```

## Deviation note (2026-06-18)

The original blueprint (§2.1) also listed `sample-adaptive.xml` and
`sample-aapt2-stdout.txt`. These were dropped because:

- `parse_adaptive_xml(apk_path, xml_entry)` is **not** a pure XML parser — it
  reads the XML from a ZIP and calls `resolve_drawable` (more ZIP I/O). Its
  pure sub-logic `extract_drawable_ref(&str)` is tested directly instead.
- `resolve_icon_paths(aapt2_path, apk_path)` runs the `aapt2` binary. Its pure
  sub-logic `extract_density(&str)` is tested directly instead.

Full end-to-end tests for those two functions would require either pure-parser
extraction refactors or a stub `aapt2` binary — deferred (see plan
"Discovered Issues").