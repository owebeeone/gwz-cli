# Repository Member Lifecycle

Use the `gwz repo` command family to bring individual repositories into an
existing workspace and to change which historical member designations are
active.

## Clone A New Member

Clone a Git repository into the workspace and register it as an active member:

```sh
gwz repo clone git@github.com:org/shared.git libs/shared
```

The member path is optional and otherwise derives from the URL. The destination
must be available for a new checkout. `--member-id` and `--source-id` override
the derived designation and logical source identities:

```sh
gwz repo clone git@github.com:org/replacement.git libs/shared \
  --member-id mem_replacement
```

The source id defaults from the final member id, so `mem_replacement` produces
`src_replacement` unless `--source-id` is supplied. A derived member-id
collision requires an explicit new `--member-id`; GWZ does not auto-suffix it.

`repo clone` creates a new designation. It does not infer that a historical row
should be reactivated. Use `repo attach` for a retained historical checkout, or
bare `repo add` for the evidence-backed convenience path described below.

Preview the clone without creating the checkout or changing workspace
artifacts:

```sh
gwz --dry-run repo clone git@github.com:org/shared.git libs/shared
```

## Publish A Repository Created Locally

`gwz repo create` creates and registers a local Git repository. It does not
create a repository on GitHub or another host, and the new member initially has
no remote:

```sh
gwz repo create services/api --member-id mem_api
```

If the remote repository already contains history that should be preserved,
use `gwz repo clone` instead. Use the following flow when the local member is
the new source of truth and an empty hosted repository is created later.

First create the empty remote through the hosting service, then associate the
local member with it using normal Git:

```sh
git -C services/api remote add origin git@github.com:org/api.git
```

GWZ does not infer out-of-band Git configuration changes. Synchronize the
observed remote and current desired branch into the workspace manifest:

```sh
gwz repo sync services/api
```

`repo sync` changes manifest metadata only. It does not fetch, push, check out
a branch, or rewrite the lock. The current implementation also does not stage
the rewritten manifest, so include `gwz.conf` when staging the initial change:

```sh
printf '# API service\n' > services/api/README.md
gwz add services/api/README.md gwz.conf
gwz commit -m "Create API service"
gwz --member mem_api push
```

The selected push sends the member's current branch to the recorded `origin`
without also trying to push the workspace root. A branch must have at least one
commit before it can be pushed.

## Detach Without Deleting

Remove an active member from the current workspace composition without deleting
its identity or checkout:

```sh
gwz repo detach libs/shared
# The member id is also accepted:
gwz repo detach mem_shared
```

Detach sets the manifest row inactive and removes its current lock row.
Snapshots and markers remain unchanged. If the nested Git checkout remains on
disk, GWZ continues to protect it from accidental workspace-root staging.

The command accepts exactly one active member id or workspace-relative path.
Do not combine its positional operand with global selection flags such as
`--target`, `--no-target`, `--member`, or `--all`.

## Attach The Historical Designation

Reactivate an inactive designation while preserving its member and source ids:

```sh
gwz repo attach mem_shared
```

Attach requires the literal historical member id, not a path or an `@`
selector. The checkout must still exist at the recorded path and must be a Git
repository. On success, GWZ makes the row active, observes the checkout, and
recreates its lock state. It does not rewrite snapshots or markers.

GWZ verifies every commit recorded for that member in snapshots and markers. If
all recorded commits exist, the explicit historical identity is accepted. An
already-active id is a successful no-op; an unknown id is an error.

When no snapshot or marker records a commit for the explicitly named member,
attach is allowed because the user chose the identity and there is no
contradictory recorded hash. The successful response and warning event use this
exact text:

```text
attached <member_id>; no snapshot or marker commit evidence was available to verify repository identity
```

## Re-add A Detached Checkout

When the detached checkout already exists on disk, bare add can restore its old
designation without requiring the member id:

```sh
gwz repo add libs/shared
```

For each inactive manifest row at that exact path, GWZ checks the locally
recorded historical evidence. Bare add reactivates a row only when exactly one
candidate has a non-empty evidence set and every recorded commit exists in the
repository. If the path has no inactive history, bare add follows the normal
new-member flow.

Empty evidence cannot drive automatic add-to-attach inference. Zero or multiple
verified historical candidates require an explicit choice: use `gwz repo
attach <member-id>` to preserve a particular historical identity, or supply a
new `--member-id` to create a new designation.

```sh
gwz repo add libs/shared --member-id mem_shared_v2
```

Supplying a new member id means “create a new designation”; it does not
reactivate the old row. Its source id defaults from the final member id, so the
example produces `src_shared_v2` unless `--source-id` is supplied.

## Historical Commit Evidence

Remote URLs are hints, not proof of repository identity. GWZ instead collects
every non-null Git commit recorded at:

- `snapshots/*.yaml` → `members.<member_id>.commit`
- `markers/*.yaml` → `members.<member_id>.commit`

It deduplicates the hashes while retaining artifact provenance for error
messages. Every required object must exist locally and resolve to a commit.
This gate applies to explicit attach, evidence-backed bare add, and explicit
reuse of an existing source id.

If any required object is absent, the operation fails with
`SourceIdentityMismatch` before changing the manifest or lock. The error detail
identifies the missing hashes and their snapshot or marker provenance. This can
mean that the checkout is the wrong repository, but it also occurs when a
shallow or incomplete checkout lacks older objects.

GWZ deliberately performs no fetch during identity verification. Verify the
checkout, fetch sufficient history with normal Git commands, and retry:

```sh
git -C libs/shared fetch --all --tags
# If the checkout is shallow and the server supports it:
git -C libs/shared fetch --unshallow --all --tags
gwz repo attach mem_shared
```

An unreadable snapshot or marker fails closed because verification requires a
complete view of the recorded evidence.

Explicitly reusing a source id for a new designation is subject to the same
gate. When that source has no recorded commit evidence, reuse succeeds with:

```text
accepted source identity <source_id>; no snapshot or marker commit evidence was available to verify repository identity
```

## Replace A Member At The Same Path

Keep the historical row inactive and give the replacement a new member id:

```sh
gwz repo detach mem_shared
mv libs/shared ../shared-old
gwz repo clone git@github.com:org/replacement.git libs/shared \
  --member-id mem_replacement
```

The inactive `mem_shared` history remains available, while path-based selection
resolves the new active designation.
