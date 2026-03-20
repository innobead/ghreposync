# ghreposync

Sync GitHub repository resources (labels, milestones, …) from a source repo to a target repo.

## Installation

### Download a pre-built binary

Download the latest binary for your platform from the [Releases](https://github.com/innobead/ghreposync/releases) page.

### Build from source

Requires Rust stable and [just](https://just.systems).

```sh
just release                          # optimised binary for the current host → dist/ghreposync-<triple>
just release-target <target-triple>   # cross-compile for a specific target
```

The binary is placed in `dist/`. Add it to your `PATH` or invoke it directly:

```sh
export PATH="$PWD/dist:$PATH"
```

Alternatively, install into Cargo's bin directory:

```sh
cargo install --path .
```

## CLI usage

```
ghreposync [--token <token>] sync --source <owner/repo> --target <owner/repo> [options]
```

`--token` / `GITHUB_TOKEN` env var is required to authenticate with the GitHub API.

### Options

| Flag | Short | Description | Default |
|---|---|---|---|
| `--source` | `-s` | Source repository (`owner/repo`) | required |
| `--target` | `-t` | Target repository (`owner/repo`) | required |
| `--resource` | `-r` | Resources to sync (see below) | `all` |
| `--dry-run` | | Preview changes without applying them | false |

### `--resource` values

| Value | Description |
|---|---|
| `all` | Sync every supported resource |
| `labels` | Repository labels (name, colour, description) |
| `milestones` | Repository milestones (title, description, state, due date) |

Multiple values can be passed as a comma-separated list.

### Examples

```sh
# Sync everything (default)
ghreposync sync --source org/template --target org/my-repo

# Sync labels only
ghreposync sync --source org/template --target org/my-repo --resource labels

# Sync labels and milestones, preview first
ghreposync sync --source org/template --target org/my-repo \
  --resource labels,milestones \
  --dry-run

# Use an explicit token
ghreposync --token ghp_xxx sync --source org/template --target org/my-repo
```

### Sync behaviour

- **Created** — resource exists in source but not in target.
- **Updated** — resource exists in both but differs (colour, description, state, due date, etc.).
- **Unchanged** — resource exists in both and is identical; no API call is made.

Resources are matched by name / title (case-insensitive). Deletion is never performed.

## GitHub Action

This repo is also a reusable GitHub Action. Reference it with `uses: innobead/ghreposync@v1` in any workflow.

### Inputs

| Input | Description | Required | Default |
|---|---|---|---|
| `source` | Source repository (`owner/repo`) | yes | |
| `target` | Target repository (`owner/repo`) | yes | |
| `resource` | Comma-separated resource list, or `all` | no | `all` |
| `dry-run` | Preview changes without applying them | no | `false` |
| `token` | GitHub token with access to both repos | yes | |
| `version` | Release version to download (e.g. `v0.1.0`), or `latest` | no | `latest` |

### Permissions

The token needs **write** access to issues on the target repo (labels and milestones use the issues API). For repos in the same organisation the default `GITHUB_TOKEN` is sufficient with `issues: write`. For cross-organisation targets, pass a PAT with `repo` scope.

### Examples

#### Sync all resources on a schedule

```yaml
name: Sync from template

on:
  schedule:
    - cron: '0 3 * * *'   # daily at 03:00 UTC
  workflow_dispatch:

permissions:
  issues: write

jobs:
  sync:
    runs-on: ubuntu-latest
    steps:
      - uses: innobead/ghreposync@v1
        with:
          source: org/template-repo
          target: org/my-repo
          token: ${{ secrets.GITHUB_TOKEN }}
```

#### Sync labels only, triggered manually with a dry-run option

```yaml
name: Sync labels

on:
  workflow_dispatch:
    inputs:
      dry_run:
        description: Preview changes without applying them
        type: boolean
        default: false

permissions:
  issues: write

jobs:
  sync:
    runs-on: ubuntu-latest
    steps:
      - uses: innobead/ghreposync@v1
        with:
          source: org/template-repo
          target: org/my-repo
          resource: labels
          dry-run: ${{ inputs.dry_run }}
          token: ${{ secrets.GITHUB_TOKEN }}
```

#### Sync to multiple targets

```yaml
jobs:
  sync:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        target:
          - org/repo-a
          - org/repo-b
          - org/repo-c
    steps:
      - uses: innobead/ghreposync@v1
        with:
          source: org/template-repo
          target: ${{ matrix.target }}
          token: ${{ secrets.GITHUB_TOKEN }}
```

#### Cross-organisation sync using a PAT

```yaml
jobs:
  sync:
    runs-on: ubuntu-latest
    steps:
      - uses: innobead/ghreposync@v1
        with:
          source: org-a/template-repo
          target: org-b/my-repo
          token: ${{ secrets.CROSS_ORG_PAT }}
```

## Supported resources

| Resource | Matched by | Fields synced |
|---|---|---|
| Labels | name (case-insensitive) | colour, description |
| Milestones | title (case-insensitive) | description, state, due date |

## License

MIT
