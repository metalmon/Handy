# GitHub Release Pipeline Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Publish a GitHub Release containing an unsigned Windows installer and Linux `.deb`/`.rpm` packages that run on Astra Linux SE 1.8 and РЕДОС 8, re-runnable under the same tag without a version bump.

**Architecture:** Reuse the existing reusable `build.yml` workflow and trim its callers to two targets. Release creation becomes idempotent per tag (find release via the paginated list, delete its assets, reuse it) and a new `publish-release` job flips the draft to published only after every matrix target succeeds. Code signing is deleted from the repo entirely so the Windows build works without secrets; the Linux glibc baseline is asserted in CI instead of trusted.

**Tech Stack:** GitHub Actions, `actions/github-script`, `tauri-apps/tauri-action@v0`, Tauri v2, Debian/rpm packaging, `objdump`, PowerShell.

**Spec:** `docs/superpowers/specs/2026-10-02-github-release-pipeline-design.md`

---

## Verification strategy (read first)

There is no unit-test harness for GitHub Actions YAML in this repo. Three substitutes are used, in increasing strength:

1. **`bunx prettier --check --end-of-line auto .github/workflows/`** — Prettier parses YAML, so this fails on any syntax error introduced by a bad edit.

   `--end-of-line auto` is mandatory on this machine. `.prettierrc` sets `endOfLine: lf` while `core.autocrlf=true` gives this Windows checkout CRLF files, so a bare `--check` reports style violations on every pre-existing file regardless of its content. Without the flag the gate is red before any work starts and proves nothing. Never run `prettier --write` to "fix" it — that rewrites line endings across whole files and buries the real diff.
2. **`cargo test -p handy` / `bun run build` / `bun run lint`** — guard the Rust, frontend and i18n state that config edits could disturb.
3. **The first real `release.yml` run** — the authoritative check. Tasks 5, 6 and 7 only truly pass when CI is green.

Do not claim a task is verified because the YAML parses. Only the CI run proves the packaging behaviour.

---

## File Structure

| File | Responsibility after this plan |
|---|---|
| `.github/workflows/release.yml` | Idempotent release upsert, two-target matrix, post-success publish |
| `.github/workflows/build.yml` | Shared build; no signing machinery; audits model presence and glibc baseline |
| `.github/workflows/main-build.yml` | Push-to-main check, two targets |
| `.github/workflows/build-test.yml` | Manual test build, two targets |
| `.github/workflows/pr-test-build.yml` | PR check, two targets |
| `src-tauri/tauri.conf.json` | No `signCommand` |
| `BUILD.md` | Release procedure; signing troubleshooting section deleted |
| `README.md` | Install instructions for what this fork actually ships |

---

### Task 1: Delete code signing from the repository

**Files:**
- Modify: `src-tauri/tauri.conf.json:74`
- Modify: `.github/workflows/build.yml:52`, `.github/workflows/build.yml:166-176`, `.github/workflows/build.yml:519-530`
- Modify: `.github/workflows/release.yml:76`, `.github/workflows/main-build.yml:51`, `.github/workflows/build-test.yml:40`, `.github/workflows/pr-test-build.yml:46`

- [ ] **Step 1: Remove `signCommand` from tauri.conf.json**

In `src-tauri/tauri.conf.json` delete this line (and its trailing comma on the preceding line if it becomes last in the object):

```json
      "signCommand": "trusted-signing-cli -e https://eus.codesigning.azure.net/ -a CJ-Signing -c cjpais-dev -d Handy %1",
```

Verify the file still parses as JSON:

```bash
bun -e "JSON.parse(require('fs').readFileSync('src-tauri/tauri.conf.json','utf8')); console.log('json ok')"
```

Expected: `json ok`

- [ ] **Step 2: Remove the orphaned `TSC_VERSION` env var**

In `.github/workflows/build.yml` delete line 52:

```yaml
  TSC_VERSION: "0.9.0"
```

It is referenced only by the `trusted-signing-cli` steps deleted in Step 3.

- [ ] **Step 3: Remove the trusted-signing-cli steps**

In `.github/workflows/build.yml` delete both steps:

```yaml
      - name: Cache trusted-signing-cli
        if: contains(inputs.platform, 'windows') && inputs.sign-binaries
        id: cache-tsc
        uses: actions/cache@v5
        with:
          path: ~/.cargo/bin/trusted-signing-cli*
          key: trusted-signing-cli-${{ env.TSC_VERSION }}-${{ runner.os }}-${{ runner.arch }}

      - name: Install trusted-signing-cli
        if: contains(inputs.platform, 'windows') && inputs.sign-binaries && steps.cache-tsc.outputs.cache-hit != 'true'
        run: cargo install --locked --force trusted-signing-cli@${{ env.TSC_VERSION }}
```

- [ ] **Step 4: Remove signing env vars from the Build with Tauri step**

In `.github/workflows/build.yml`, inside the `Build with Tauri` step, delete these twelve lines from `env:`:

```yaml
          APPLE_ID: ${{ inputs.sign-binaries && secrets.APPLE_ID || '' }}
          APPLE_ID_PASSWORD: ${{ inputs.sign-binaries && secrets.APPLE_ID_PASSWORD || '' }}
          APPLE_PASSWORD: ${{ inputs.sign-binaries && secrets.APPLE_PASSWORD || '' }}
          APPLE_TEAM_ID: ${{ inputs.sign-binaries && secrets.APPLE_TEAM_ID || '' }}
          APPLE_CERTIFICATE: ${{ inputs.sign-binaries && secrets.APPLE_CERTIFICATE || '' }}
          APPLE_CERTIFICATE_PASSWORD: ${{ inputs.sign-binaries && secrets.APPLE_CERTIFICATE_PASSWORD || '' }}
          APPLE_SIGNING_IDENTITY: ${{ inputs.sign-binaries && env.CERT_ID || '' }}
          AZURE_CLIENT_ID: ${{ inputs.sign-binaries && secrets.AZURE_CLIENT_ID || '' }}
          AZURE_CLIENT_SECRET: ${{ inputs.sign-binaries && secrets.AZURE_CLIENT_SECRET || '' }}
          AZURE_TENANT_ID: ${{ inputs.sign-binaries && secrets.AZURE_TENANT_ID || '' }}
          TAURI_SIGNING_PRIVATE_KEY: ${{ inputs.sign-binaries && secrets.TAURI_SIGNING_PRIVATE_KEY || '' }}
          TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ inputs.sign-binaries && secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD || '' }}
```

Leave `GITHUB_TOKEN`, `LINUXDEPLOY_EXCLUDED_LIBRARIES` and `LINUXDEPLOY_OUTPUT_VERSION` untouched — they are not signing.

- [ ] **Step 5: Set `sign-binaries: false` in all four callers**

In each of `.github/workflows/release.yml`, `.github/workflows/main-build.yml`, `.github/workflows/build-test.yml`, `.github/workflows/pr-test-build.yml`, change:

```yaml
      sign-binaries: true
```

to:

```yaml
      sign-binaries: false
```

- [ ] **Step 6: Verify no signing references remain**

```bash
rg -n "trusted-signing|signCommand|AZURE_CLIENT|TAURI_SIGNING_PRIVATE_KEY|TSC_VERSION|cache-tsc" .github/workflows src-tauri/tauri.conf.json
```

Expected: no output.

The Apple signing variables are deliberately absent from this grep. `build.yml`
keeps its macOS certificate-import steps (guarded by `if: contains(inputs.platform,
'macos')`) so macOS support can return later, and those steps reference
`secrets.APPLE_CERTIFICATE`. They are inert for this fork — every caller now
passes `sign-binaries: false`, and the input's own default in `build.yml` was
already `false`.

- [ ] **Step 7: Verify the workflows still parse**

```bash
bunx prettier --check --end-of-line auto .github/workflows/release.yml .github/workflows/build.yml .github/workflows/main-build.yml .github/workflows/build-test.yml .github/workflows/pr-test-build.yml
```

Expected: `All matched files use Prettier code style!`

- [ ] **Step 8: Commit**

```bash
git add src-tauri/tauri.conf.json .github/workflows/
git commit -m "fix: drop code signing so Windows builds need no secrets"
```

---

### Task 2: Make release creation idempotent per tag

**Files:**
- Modify: `.github/workflows/release.yml:6-39`

**Why:** Re-running the workflow must overwrite the assets of the existing release. `createRelease` fails on an existing tag, and re-uploading an asset with a name that already exists fails with 422. Also, nothing ever published the draft, so the release was never publicly visible.

- [ ] **Step 1: Replace the release-creation job**

In `.github/workflows/release.yml`, replace the whole `create-release:` job (lines 6-39) with:

```yaml
  ensure-release:
    permissions:
      contents: write
    runs-on: ubuntu-latest
    outputs:
      # actions/github-script exposes exactly one output, named `result`, holding
      # whatever the script returns. The step id is NOT the output name, so
      # `steps.ensure-release.outputs.release-id` would resolve to empty.
      release-id: ${{ steps.ensure-release.outputs.result }}
      version: ${{ steps.get-version.outputs.version }}
    steps:
      - name: Checkout repository
        uses: actions/checkout@v5

      - name: Get version from tauri.conf.json
        id: get-version
        shell: bash
        run: |
          VERSION=$(grep -o '"version": "[^"]*"' src-tauri/tauri.conf.json | cut -d'"' -f4)
          echo "Application version from tauri.conf.json: $VERSION"
          echo "version=$VERSION" >> "$GITHUB_OUTPUT"

      - name: Ensure Draft Release (reuse tag, replace assets)
        id: ensure-release
        uses: actions/github-script@v9
        with:
          script: |
            const tag = `v${{ steps.get-version.outputs.version }}`;
            // GET /releases/tags/{tag} never returns drafts, so a re-release would
            // find nothing. The paginated list is the only way to see a draft.
            const releases = await github.paginate(github.rest.repos.listReleases, {
              owner: context.repo.owner,
              repo: context.repo.repo,
              per_page: 100
            });
            const existing = releases.find((r) => r.tag_name === tag);

            if (existing) {
              core.info(
                `Reusing release #${existing.id} for ${tag}; deleting ${existing.assets.length} existing asset(s)`
              );
              // Re-uploading an asset whose name already exists fails with 422,
              // so a re-release has to clear the previous set first.
              for (const asset of existing.assets) {
                await github.rest.repos.deleteReleaseAsset({
                  owner: context.repo.owner,
                  repo: context.repo.repo,
                  asset_id: asset.id
                });
              }
              return existing.id;
            }

            core.info(`No release found for ${tag}; creating a draft`);
            const { data } = await github.rest.repos.createRelease({
              owner: context.repo.owner,
              repo: context.repo.repo,
              tag_name: tag,
              name: tag,
              draft: true,
              prerelease: false,
              generate_release_notes: true
            });
            return data.id;
```

- [ ] **Step 2: Add the concurrency guard**

Directly under `on: workflow_dispatch` in `.github/workflows/release.yml`, add:

```yaml
# Two concurrent runs would delete each other's release assets mid-upload.
concurrency:
  group: release
  cancel-in-progress: false
```

The file header must now read:

```yaml
name: "Release"

on: workflow_dispatch

concurrency:
  group: release
  cancel-in-progress: false

jobs:
```

- [ ] **Step 3: Add the publish job**

The `needs` list must include **both** jobs. `needs` holds only *direct*
dependencies — GitHub's contexts reference states it "doesn't include
implicitly dependent jobs (for example, dependent jobs of a dependent job)" —
and a dereference of an absent property "will evaluate to an empty string".
Since `ensure-release` is only a transitive dependency via `publish-tauri`,
omitting it would substitute an empty string into the script body and produce
`const releaseId = ;`, a hard SyntaxError on every run.

Append to the end of `.github/workflows/release.yml`, after the `publish-tauri` job:

```yaml
  # Publishing is a separate, final job so a failed target can never ship a
  # half-built release: needs: makes this unreachable unless every target is green.
  publish-release:
    permissions:
      contents: write
    needs: [ensure-release, publish-tauri]
    runs-on: ubuntu-latest
    steps:
      - name: Publish release
        uses: actions/github-script@v9
        with:
          script: |
            const releaseId = ${{ needs.ensure-release.outputs.release-id }};
            await github.rest.repos.updateRelease({
              owner: context.repo.owner,
              repo: context.repo.repo,
              release_id: releaseId,
              draft: false
            });
            core.info(`Published release #${releaseId}`);
```

`release_id` receives the numeric release id as a string, which the REST client
accepts. The script returns a bare number, not an object, so
`steps.ensure-release.outputs.result` is that number with no JSON quoting.

- [ ] **Step 4: Update the dependency reference**

In `.github/workflows/release.yml` change:

```yaml
    needs: create-release
```

to:

```yaml
    needs: ensure-release
```

and inside the `with:` block of that job change:

```yaml
      release-id: ${{ needs.create-release.outputs.release-id }}
```

to:

```yaml
      release-id: ${{ needs.ensure-release.outputs.release-id }}
```

- [ ] **Step 5: Verify the workflow parses and no stale references remain**

```bash
rg -n "create-release" .github/workflows/release.yml
bunx prettier --check --end-of-line auto .github/workflows/release.yml
```

Expected: `rg` finds nothing; Prettier reports the file is formatted.

- [ ] **Step 6: Commit**

```bash
git add .github/workflows/release.yml
git commit -m "feat: make release creation idempotent and publish after green builds"
```

---

### Task 3: Trim the release matrix to Windows and Linux

**Files:**
- Modify: `.github/workflows/release.yml:45-69`

- [ ] **Step 1: Replace the matrix**

In `.github/workflows/release.yml`, inside the `publish-tauri` job's `strategy.matrix`, delete the entire `include:` list and replace it with:

```yaml
        include:
          # Built on 22.04 so the packaged binary requires at most GLIBC_2.35.
          # Both release targets ship glibc 2.36, so this runs on both; a build
          # on 24.04 (glibc 2.39) would not necessarily start on RedOS 8.
          - platform: "ubuntu-22.04"
            args: "--bundles deb,rpm"
            target: "x86_64-unknown-linux-gnu"
          - platform: "windows-latest"
            args: ""
            target: "x86_64-pc-windows-msvc"
```

The job keeps `fail-fast: false`, `asset-prefix: "handy"`, `upload-artifacts: false` and `no-cache: true` unchanged.

- [ ] **Step 2: Verify the workflow parses**

```bash
bunx prettier --check --end-of-line auto .github/workflows/release.yml
```

Expected: `All matched files use Prettier code style!`

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/release.yml
git commit -m "ci: build only Windows and Linux packages for releases"
```

---

### Task 4: Trim the three non-release matrices

**Files:**
- Modify: `.github/workflows/main-build.yml:22-43`
- Modify: `.github/workflows/build-test.yml` (matrix `include:` list)
- Modify: `.github/workflows/pr-test-build.yml` (matrix `include:` list)

**Why:** The fork does not ship macOS, ARM or AppImage. Leaving seven targets in the per-PR workflows would spend CI minutes on platforms nobody releases.

- [ ] **Step 1: Replace the main-build matrix**

In `.github/workflows/main-build.yml`, replace the whole `include:` list with:

```yaml
        include:
          - platform: "ubuntu-22.04"
            args: "--bundles deb,rpm"
            target: "x86_64-unknown-linux-gnu"
          - platform: "windows-latest"
            args: ""
            target: "x86_64-pc-windows-msvc"
```

- [ ] **Step 2: Replace the build-test matrix**

In `.github/workflows/build-test.yml`, replace the whole `include:` list with:

```yaml
        include:
          - platform: "ubuntu-22.04"
            args: "--bundles deb,rpm"
            target: "x86_64-unknown-linux-gnu"
          - platform: "windows-latest"
            args: ""
            target: "x86_64-pc-windows-msvc"
```

- [ ] **Step 3: Replace the pr-test-build matrix**

In `.github/workflows/pr-test-build.yml`, replace the whole `include:` list with:

```yaml
        include:
          - platform: "ubuntu-22.04"
            args: "--bundles deb,rpm"
            target: "x86_64-unknown-linux-gnu"
          - platform: "windows-latest"
            args: ""
            target: "x86_64-pc-windows-msvc"
```

- [ ] **Step 4: Verify all three parse and contain only two targets**

```bash
bunx prettier --check --end-of-line auto .github/workflows/main-build.yml .github/workflows/build-test.yml .github/workflows/pr-test-build.yml
rg -c "platform: " .github/workflows/main-build.yml .github/workflows/build-test.yml .github/workflows/pr-test-build.yml
```

Expected: Prettier clean; each file reports `2`.

- [ ] **Step 5: Commit**

```bash
git add .github/workflows/main-build.yml .github/workflows/build-test.yml .github/workflows/pr-test-build.yml
git commit -m "ci: drop macOS and ARM from the per-push and per-PR builds"
```

---

### Task 5: Assert the bundled model survives Windows packaging

**Files:**
- Modify: `.github/workflows/build.yml:832-856` (the `Assert-PackageContents` function)

**Why:** The app is offline by design. A packaged build missing `model.int8.onnx` installs and launches fine, then fails silently at transcription time. The existing audit only checks runtime DLLs.

The check uses the CLI's own model registry rather than a hardcoded path, so it verifies real resolution instead of guessing where the bundler put the file. `handy.exe --list-models --json` exits before GUI init and prints `ModelInfo[]`; locally the bundled model reports `"source": { "Bundled": ... }` and `"is_downloaded": true`.

- [ ] **Step 1: Add the model assertion**

In `.github/workflows/build.yml`, inside `Assert-PackageContents`, insert after the staged-DLL loop and before the existing `--list-devices` call:

```powershell
            # Offline by design: a package missing the model installs and launches
            # fine, then fails silently at transcription time. Ask the app's own model
            # registry instead of guessing the path the bundler chose.
            $modelsJson = (& $handy.FullName --list-models --json | Out-String)
            $modelsJson | Write-Host
            if ($LASTEXITCODE -ne 0) {
                throw "$Label handy.exe --list-models --json failed"
            }
            $gigaam = ($modelsJson | ConvertFrom-Json) | Where-Object { $_.id -eq "gigaam-v3-e2e-ctc" }
            if (-not $gigaam) {
                throw "$Label package does not expose the bundled gigaam-v3-e2e-ctc model"
            }
            if (-not $gigaam.is_downloaded) {
                throw "$Label exposes gigaam-v3-e2e-ctc but is_downloaded is false - model files are missing from the installer"
            }
            Write-Host "$Label bundled model verified (is_downloaded=true)"
```

- [ ] **Step 2: Confirm the JSON shape locally**

This step is the regression check for the assertion above; run it against the locally built binary before trusting the CI version:

```bash
src-tauri\target\release\handy.exe --list-models --json
```

Expected output contains `"id": "gigaam-v3-e2e-ctc"` and `"is_downloaded": true`. If it does not, the model is missing from your local `target/release` — run `bun run fetch:model && bun run build` first.

- [ ] **Step 3: Verify the workflow parses**

```bash
bunx prettier --check --end-of-line auto .github/workflows/build.yml
```

Expected: `All matched files use Prettier code style!`

- [ ] **Step 4: Commit**

```bash
git add .github/workflows/build.yml
git commit -m "ci: assert the packaged Windows app finds its bundled model"
```

---

### Task 6: Ship ONNX Runtime inside the rpm package

**Files:**
- Modify: `.github/workflows/build.yml:382-389` (Linux ORT step)
- Modify: `.github/workflows/build.yml:670-694` (rpm audit block)

**Why:** Moving rpm from `ubuntu-24.04` to `ubuntu-22.04` in Task 3 breaks the rpm, and the audit will not catch it. This was introduced by that move, not inherited:

- On `ubuntu-24.04` the Linux ORT step never ran, so `ORT_PREFER_DYNAMIC_LINK` was unset and ORT was linked **statically**. The rpm shipped a self-contained binary.
- On `ubuntu-22.04` the ORT step sets `ORT_PREFER_DYNAMIC_LINK=1`, so the binary ends up with `NEEDED libonnxruntime.so.1` (stated at `build.yml:383`).
- `src-tauri/build.rs:122-124` returns early for non-Windows targets, so `transcribe-libs/` never receives that `.so` on Linux. The only injection is the `.bundle.linux.deb.files` entry at `build.yml:387-389`.
- `tauri.conf.json:62-64` maps `rpm.files["/usr/lib/Handy"] = "transcribe-libs"`, which is therefore empty of ORT.

Result: the deb ships the library, the rpm does not, and `handy` cannot start on РЕДОС 8. The rpm audit at `build.yml:674-694` lists the package but never checks for ORT, unlike the deb audit at `build.yml:656-658`.

- [ ] **Step 1: Inject the ORT SONAME into the rpm mapping too**

In `.github/workflows/build.yml`, in the "Install ONNX Runtime (x86_64 Linux, Ubuntu 22.04)" step, change the final `jq` invocation so it writes both mappings:

```bash
          # deb.files and rpm.files keys = destination in package, value = source on disk.
          # Both need it: on 22.04 ORT is dynamically linked, so a package that omits the
          # library produces a binary that cannot start.
          jq --arg so1 "$ORT_DIR/lib/libonnxruntime.so.1" \
            '.bundle.linux.deb.files["/usr/lib/Handy/libonnxruntime.so.1"] = $so1
             | .bundle.linux.rpm.files["/usr/lib/Handy/libonnxruntime.so.1"] = $so1' \
            src-tauri/tauri.conf.json > tmp.json && mv tmp.json src-tauri/tauri.conf.json
```

- [ ] **Step 2: Assert the rpm contains the ORT library**

In the same file, inside the `if compgen -G "${BUNDLE_DIR}/rpm/*.rpm"` audit block, add the counterpart of the deb audit's ORT check:

```bash
            require_pattern "$listing" 'usr/lib/Handy/libonnxruntime\.so\.[0-9]+$' "ONNX Runtime library (SONAME)"
```

- [ ] **Step 3: Verify**

```bash
rg -n "deb\.files|rpm\.files" .github/workflows/build.yml
bunx prettier --check --end-of-line auto .github/workflows/build.yml
```

Expected: the `jq` filter now sets both `deb.files` and `rpm.files`; the rpm audit gains a `require_pattern` for `usr/lib/Handy/libonnxruntime\.so`; prettier clean.

Commit:

```bash
git add .github/workflows/build.yml
git commit -m "fix: ship ONNX Runtime inside the rpm package"
```

---

### Task 7: Assert the model is packaged and the glibc baseline holds on Linux

**Files:**
- Modify: `.github/workflows/build.yml:670-716` (deb and rpm audit blocks)
- Modify: `.github/workflows/build.yml` (after the rpm block, before the AppImage block)

**Why:** Two distinct failure modes. A missing model is the same silent failure as on Windows. A raised glibc is subtler: a runner update from 22.04 to a newer image would silently ship packages that no longer start on Astra Linux SE 1.8 or РЕДОС 8, and nothing else in the pipeline would notice.

- [ ] **Step 1: Add the model requirement to the deb audit**

In `.github/workflows/build.yml`, inside the `if compgen -G "${BUNDLE_DIR}/deb/*.deb"` block, add after the ONNX Runtime `require_pattern` line:

```bash
              require_pattern "$listing" 'giga-am-v3-int8/model\.int8\.onnx' "bundled GigaAM model"
              require_pattern "$listing" 'giga-am-v3-int8/vocab\.txt' "GigaAM vocabulary"
```

- [ ] **Step 2: Add the same requirement to the rpm audit**

In the `if compgen -G "${BUNDLE_DIR}/rpm/*.rpm"` block, add after the `libggml-cpu` `require_pattern` line:

```bash
              require_pattern "$listing" 'giga-am-v3-int8/model\.int8\.onnx' "bundled GigaAM model"
              require_pattern "$listing" 'giga-am-v3-int8/vocab\.txt' "GigaAM vocabulary"
```

- [ ] **Step 3: Add the glibc baseline assertion**

In `.github/workflows/build.yml`, insert this step after the `Audit Linux package runtime contents` step (i.e. immediately before `Audit Windows package runtime contents`):

```yaml
      - name: Assert Linux glibc baseline
        if: contains(inputs.platform, 'ubuntu')
        shell: bash
        run: |
          set -euo pipefail
          PROFILE="${{ steps.build-profile.outputs.profile }}"
          BINARY="src-tauri/target/${PROFILE}/handy"

          if [ ! -f "$BINARY" ]; then
            echo "ERROR: no built binary at $BINARY" >&2
            exit 1
          fi

          VERSIONS="$(objdump -T "$BINARY" | grep -oE 'GLIBC_[0-9]+\.[0-9]+' | sort -Vu || true)"
          if [ -z "$VERSIONS" ]; then
            echo "ERROR: found no GLIBC symbol versions in $BINARY - is the binary stripped of dynamic symbols?" >&2
            exit 1
          fi

          HIGHEST="$(printf '%s\n' "$VERSIONS" | tail -n 1)"
          echo "Distinct GLIBC versions required by handy:"
          printf '%s\n' "$VERSIONS" | sed 's/^/  /'
          echo "Highest required: ${HIGHEST}"

          # Astra Linux SE 1.8 and RedOS 8 both ship glibc 2.36. This job builds on
          # ubuntu-22.04 (glibc 2.35), so anything at or below 2.35 runs on both. If a
          # runner image bump raises the requirement, the packages would silently stop
          # starting on the release targets - fail the build instead.
          REQUIRED="${HIGHEST#GLIBC_}"
          if awk -v have="$REQUIRED" -v max="2.35" 'BEGIN { exit !(have > max) }'; then
            echo "ERROR: handy requires GLIBC_${REQUIRED}, above the GLIBC_2.35 baseline for Astra Linux SE 1.8 / RedOS 8" >&2
            exit 1
          fi
          echo "glibc baseline OK: GLIBC_${REQUIRED} <= GLIBC_2.35"
```

`awk` is used for the comparison rather than bash string comparison, because `sort -V` orders `2.9` and `2.35` correctly but a lexicographic `[[ > ]]` test would not.

- [ ] **Step 4: Verify the workflow parses and the awk comparison behaves**

```bash
bunx prettier --check --end-of-line auto .github/workflows/build.yml
awk -v have="2.35" -v max="2.35" 'BEGIN { exit !(have > max) }'; echo "2.35 -> exit $?"
awk -v have="2.39" -v max="2.35" 'BEGIN { exit !(have > max) }'; echo "2.39 -> exit $?"
```

Expected: Prettier clean; `2.35 -> exit 0` (no failure); `2.39 -> exit 1` (failure, as intended).

- [ ] **Step 5: Commit**

```bash
git add .github/workflows/build.yml
git commit -m "ci: assert the bundled model and the glibc baseline in Linux packages"
```

---

### Task 8: Isolate the Rust cache for no-cache release builds

**Files:**
- Modify: `.github/workflows/build.yml:110-119`

**Why:** The `Rust cache` step is skipped when `no-cache: true`, but the cache key is derived from platform and target only. A `main-build` run with caching and a release run without it share a key, so the release could still be served a snapshot that a cached build wrote moments earlier — defeating the "releases always build from scratch" guarantee stated in the comment.

- [ ] **Step 1: Make the cache key depend on no-cache**

In `.github/workflows/build.yml`, replace the `Rust cache` step with:

```yaml
      - name: Rust cache
        # Release builds skip the cache (no-cache: true) so the artifact is
        # always compiled clean — no risk of a stale cached native lib being
        # linked. PR/test builds keep caching for speed. The key carries the
        # cache mode so a cached PR build and an uncached release build can never
        # share a snapshot.
        if: ${{ !inputs.no-cache }}
        uses: swatinem/rust-cache@v2
        with:
          workspaces: "./src-tauri -> target"
          key: ${{ inputs.platform }}-${{ inputs.target }}-${{ inputs.no-cache && 'nocache' || 'cached' }}-no-bin-v1
          cache-bin: false
```

- [ ] **Step 2: Verify the workflow parses**

```bash
bunx prettier --check --end-of-line auto .github/workflows/build.yml
```

Expected: `All matched files use Prettier code style!`

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/build.yml
git commit -m "ci: keep release builds out of the shared Rust cache namespace"
```

---

### Task 9: Rewrite the signing troubleshooting section

**Files:**
- Modify: `BUILD.md:295-323`

**Why:** The section documents a `signCommand` that Task 1 deletes. Left in place it would send developers chasing a workaround for a problem that no longer exists.

- [ ] **Step 1: Delete the obsolete section**

In `BUILD.md`, delete the entire section starting with the line `### Windows \`tauri build\` fails at bundling with \`program not found\`` and running to the end of the file (lines 295-323, ending with the `bun run tauri build --bundles nsis --config nosign.json` fence).

- [ ] **Step 2: Add the release procedure**

Append to `BUILD.md`:

````markdown
### Cutting a release

1. Set the version in `src-tauri/tauri.conf.json` (currently `0.9.7`).
2. Run the **Release** workflow (`workflow_dispatch`) from the Actions tab.

The workflow builds two targets and publishes them under the tag `v<version>`:

| Artifact | Target |
| --- | --- |
| `Handy_<version>_x64-setup.exe` (NSIS, shows `ХЭНДИ`) | Windows x64 |
| `Handy_<version>_x64_en-US.msi` (WiX, shows `Handy`) | Windows x64 |
| `handy_<version>_amd64.deb` | Astra Linux SE 1.8, Debian/Ubuntu |
| `handy-<version>-1.x86_64.rpm` | РЕДОС 8 |

Artifacts are **unsigned**. Windows will show a SmartScreen warning and
macOS-style quarantine prompts do not apply, but no signature is present.

Re-running the workflow replaces the assets of the same release and publishes
it again — no version bump needed. If any target fails, the release stays a
draft and nothing is published.

### Linux compatibility baseline

Both Linux packages are built on `ubuntu-22.04`, so the packaged binary
requires at most `GLIBC_2.35`. The release targets ship glibc 2.36:

| Target | Base | glibc |
| --- | --- | --- |
| Astra Linux SE 1.8 | Debian 12 | 2.36 |
| РЕДОС 8 | RHEL-like (DNF) | 2.36 |

CI asserts the highest required `GLIBC_x.y` symbol version after every Linux
build, so a runner image bump cannot silently break both targets.

The `.rpm` declares a `webkit2gtk4.1` dependency. If РЕДОС 8 cannot resolve it,
install it fails regardless of how the package was built — check first with
`dnf list webkit2gtk4.1`.
````

- [ ] **Step 3: Verify formatting**

```bash
bunx prettier --check --end-of-line auto BUILD.md
```

Expected: `All matched files use Prettier code style!`

- [ ] **Step 4: Commit**

```bash
git add BUILD.md
git commit -m "docs: replace signing workaround with the release procedure"
```

---

### Task 10: Document the install paths this fork actually ships

**Files:**
- Modify: `README.md:34-48`

**Why:** The current Installation section offers macOS, winget and links to the upstream `cjpais/Handy` releases. None of that describes this fork's output.

- [ ] **Step 1: Replace the Installation section**

In `README.md`, replace lines 34-48 with:

````markdown
### Installation

Grab the artifacts from the [releases page](https://github.com/metalmon/Handy/releases).

**Windows** — download `Handy_<version>_x64-setup.exe` and run it. The
installer and the Start menu entry are named `ХЭНДИ`. Artifacts are unsigned,
so Windows SmartScreen will warn on first run.

**Astra Linux SE 1.8, Debian, Ubuntu** — install the `.deb` with APT so
dependencies resolve automatically:

```bash
sudo apt install ./handy_<version>_amd64.deb
```

Do not use `dpkg -i` unless dependencies are already installed; if you did,
run `sudo apt --fix-broken install`.

**РЕДОС 8** — install the `.rpm` with DNF:

```bash
sudo dnf install ./handy-<version>-1.x86_64.rpm
```

If DNF reports that `webkit2gtk4.1` cannot be found, the release targets are
missing a GTK/WebKit package — see [BUILD.md](BUILD.md#linux-compatibility-baseline).

Once installed, launch Handy, grant microphone permissions, pick your
shortcuts in Settings, and start transcribing.
````

- [ ] **Step 2: Verify formatting**

```bash
bunx prettier --check --end-of-line auto README.md
```

Expected: `All matched files use Prettier code style!`

- [ ] **Step 3: Commit**

```bash
git add README.md
git commit -m "docs: document install paths for Astra Linux, RedOS and Windows"
```

---

### Task 11: Full local regression run

**Files:** none — verification only.

- [ ] **Step 1: Frontend checks**

```bash
bun run build
bun run lint
bunx tsc --noEmit
bun run check:translations
```

Expected: all four exit 0.

- [ ] **Step 2: Rust checks**

```bash
cargo fmt --check
cargo test -p handy
```

Expected: formatting clean; 255 tests pass.

- [ ] **Step 3: Prove the Windows build needs no workaround**

This is the whole point of Task 1, verified end to end:

```bash
bun run tauri build --bundles nsis
```

Expected: an installer appears at
`src-tauri/target/release/bundle/nsis/Handy_0.9.7_x64-setup.exe` and the log
contains **no** `Signing ... with a custom signing command` line and no
`program not found`. Expect this to take roughly ten minutes and to produce a
file around 165 MB.

If it still fails with `program not found`, a `signCommand` survives somewhere —
check `.vscode`, `tauri.*.conf.json` merge files and any leftover
`--config` in `package.json` scripts.

- [ ] **Step 4: Confirm the installer really carries the model**

```bash
Select-String -Path src-tauri/target/release/nsis/x64/installer.nsi -Pattern "model.int8.onnx" -Quiet
```

Expected: `True`

- [ ] **Step 5: Confirm a clean tree**

```bash
git status --short
git log --oneline -12
```

Expected: no modified tracked files; ten commits on top of `472c42a`.

- [ ] **Step 6: Hand off to CI**

Nothing in this plan proves the Linux packages install on Astra Linux SE 1.8 or
РЕДОС 8 — that machine is not reachable from here. Report to the user that the
pipeline is ready and that the first `release.yml` run is the authoritative
check, along with the `dnf list webkit2gtk4.1` verification they must run on
РЕДОС 8.

---

## Plan self-review

**Spec coverage**

| Spec section | Tasks |
| --- | --- |
| 5. Матрица сборки | 3, 4 |
| 6.1 Идемпотентный `ensure-release` | 2 |
| 6.2 Параметры матрицы | 3 |
| 6.3 Публикация и `concurrency` | 2 |
| 7.1 Снятие подписи | 1 |
| 7.2 Сохранённые платформенные ветки | no task — deliberate, nothing to do |
| 7.3 Новые проверки | 5, 6 |
| 7.4 Матрицы тестовых воркфлоу | 4 |
| 8. Документация | 8, 9 |
| 9. Верификация | 10, Step 6 |
| 11. Критерии приёмки | 3, 5, 6, 10 |

**Deviation from spec, deliberate:** the spec did not mention the shared Rust
cache key. Task 8 adds it because the spec's requirement that release builds
never restore a cached native library is not actually enforced by the code as
written.

**Placeholders:** none. Every step carries literal content or a literal command.

**Consistency:** `ensure-release` is the single name used in Steps 1, 4 and 6;
`is_downloaded` and `gigaam-v3-e2e-ctc` match the CLI output verified in
Task 5 Step 2; `GLIBC_2.35` matches the baseline in Tasks 7, 9 and 11.