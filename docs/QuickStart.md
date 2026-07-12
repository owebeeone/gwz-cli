# Quick Start

GWZ (Git Workspace Zone) coordinates several ordinary Git repositories as one
workspace. The root repository records which member repositories belong to the
workspace and the exact revisions that make up a reproducible state; each
member remains a normal Git repository.

This guide gets you through the first useful workflow. Use the
[repository lifecycle guide](RepoLifecycle.md) when you need the full identity
and recovery rules.

## 1. Install GWZ

On macOS or Linux:

```sh
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/owebeeone/gwz-cli/releases/latest/download/gwz-installer.sh | sh
```

On Windows PowerShell:

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/owebeeone/gwz-cli/releases/latest/download/gwz-installer.ps1 | iex"
```

Confirm the installation:

```sh
gwz --version
gwz --help
```

See [Install](Install.md) for pinned versions, source installs, and release
verification.

## 2. Choose How To Start

### Create A New Workspace

Create an empty Git repository that will own the workspace metadata:

```sh
mkdir demo
cd demo
gwz init
```

Then grow it using the command that matches the repository you have:

| Situation | Command |
| --- | --- |
| The remote repository already exists | `gwz repo clone <url> [path]` |
| The local Git repository already exists | `gwz repo add <path>` |
| The repository does not exist yet | `gwz repo create <path>` |

Create one new local member for this walkthrough:

```sh
gwz repo create services/api
gwz status
```

In another workspace, the corresponding remote-clone or existing-local-repo
forms would be:

```sh
gwz repo clone git@github.com:org/shared.git libs/shared
gwz repo add tools/local-helper
```

Commit the root metadata after checking the result. If a local repository was
registered with `repo add`, explicitly include the manifest:

```sh
gwz add gwz.conf
gwz commit -m "Define the workspace"
```

!!! note "Two pairs of commands sound similar"

    - `gwz clone` clones an entire **workspace root** and materializes its
      members. `gwz repo clone` adds one **member repository** to the workspace
      you are already in.
    - `gwz add` stages **file content** across repositories. `gwz repo add`
      registers an existing **Git repository** as a member.

### Clone An Existing Workspace

If somebody has already published a GWZ workspace, clone its root and
materialize the locked member revisions in one operation:

```sh
gwz clone https://github.com/org/workspace.git work/workspace
cd work/workspace
gwz status
gwz ls --local
```

If the root was cloned with plain `git clone`, finish it with:

```sh
gwz materialize --lock
gwz status
```

## 3. Work Across The Repositories

The everyday loop is intentionally Git-like:

```sh
gwz status
gwz diff
gwz add path/to/file another/member/file
gwz diff --cached
gwz commit -m "Update the shared API"
gwz status
```

Commands discover the workspace from the current directory, including when
run inside a member. Use `--root <path>` only when you need to override that
discovery.

Run the same command in members when a change spans several repositories:

```sh
gwz forall -- git status --short
gwz forall gwz-cli gwz-core -- cargo test
```

Record a named checkpoint before a broad or risky change:

```sh
gwz snapshot before-refactor
# Later, if needed:
gwz materialize --snapshot before-refactor
```

Preview broad mutations before applying them:

```sh
gwz --dry-run pull --head
gwz pull --head
gwz push
```

## 4. Publish A Member Created Locally

`gwz repo create` creates a local repository; it does not create a repository
on GitHub or another hosting service. When an empty hosted repository is ready,
add its `origin` with Git and synchronize that configuration into the GWZ
manifest:

```sh
git -C services/api remote add origin git@github.com:org/api.git
gwz repo sync services/api

printf '# API service\n' > services/api/README.md
gwz add services/api/README.md gwz.conf
gwz commit -m "Create API service"
gwz --member mem_api push
```

`repo sync` records the observed remote and desired branch. It does not create
the hosted repository, fetch, push, change branches, or rewrite the lock. Its
manifest change is currently unstaged, so the example explicitly stages
`gwz.conf`. The member needs at least one commit before its first push.

If the hosted repository already contains history that must be preserved, use
`gwz repo clone` instead of this publish-later flow.

## 5. Detach And Reattach A Member

Detach removes a member from the active composition without deleting its
checkout or historical designation:

```sh
gwz repo detach mem_shared
gwz repo attach mem_shared
```

Attach verifies that every commit previously recorded for the member in
snapshots and markers exists in the checkout. It fails before changing metadata
when that evidence is missing. If no historical commit evidence exists,
explicit attach proceeds with a warning because you named the designation;
automatic `repo add` will not infer an identity from empty evidence.

See [Repository Member Lifecycle](RepoLifecycle.md) before replacing a member,
reattaching a shallow checkout, or reusing a source identity.

## Develop GWZ Itself

The `gwz-dev` workspace is the complete coordinated development checkout:

```sh
gwz clone https://github.com/owebeeone/gwz-dev.git gwz-dev
cd gwz-dev
gwz status
cargo test --workspace
```

Continue with [Root Workspaces](RootWorkspace.md) for the contributor workflow.

## Where To Go Next

- [Concepts](Concepts.md) explains manifests, locks, snapshots, selections, and
  remotes.
- [Workflows](Workflows.md) contains release, maintenance, pull, and scripting
  recipes.
- [CLI Reference](CLI.md) and the [command pages](commands/status.md) document
  every option.
- [Machine Output](MachineOutput.md) covers JSON, JSONL, porcelain, and exit
  codes.
- [Troubleshooting](Troubleshooting.md) covers common failures and recovery.
