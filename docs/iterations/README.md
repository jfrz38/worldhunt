# Iteration Workflow

Iterations split the MVP into reviewable, verifiable increments. Their files
are both forward plans and concise records of what actually happened.

## Status Values

| Status | Meaning |
| --- | --- |
| `Planned` | Approved but implementation has not started |
| `In Progress` | The single iteration currently being implemented |
| `Blocked` | Started but unable to proceed without an external decision or dependency |
| `Completed` | Scope and acceptance criteria have been implemented and verified |
| `Superseded` | Replaced by a later documented plan |

Only one iteration may be `In Progress`. Planning documents are not evidence of
implementation, so all initial iterations start as `Planned`.

## Update Rules

1. Set `Started` and status to `In Progress` before implementation begins.
2. Update task checkboxes as work is completed, not in one batch afterward.
3. Record decisions that affect only the iteration in its `Decisions` section.
4. Create an ADR for decisions with lasting architectural or product impact.
5. Record deviations instead of silently rewriting the original intent.
6. Add commands and concise results to `Verification`; do not paste large logs.
7. Mark `Completed` only after every acceptance criterion and required check
   passes.
8. Update the status table in [`docs/README.md`](../README.md) whenever an
   iteration status changes.
9. Add newly discovered scope to a later iteration unless it is necessary to
   satisfy the active iteration's acceptance criteria.

Completed iteration records should only change to correct factual mistakes or
add links to follow-up work.

## Branch Workflow

All development branches start from `develop`. Work must not be implemented
directly on `develop` or `main`.

An iteration normally uses one branch. If the iteration is too large, divide it
into reviewable parts and create one branch per part instead of using child
branches. Every divided branch also starts from the latest `develop` and merges
back into `develop` independently. The iteration remains `In Progress` until
all its branches are merged and the complete iteration satisfies its acceptance
criteria.

`main` is reserved for release-ready changes promoted from `develop`.

### Branch Names

Branch names follow this structure:

```text
<action>/<iteration>-<task-summary>
```

The iteration number is mandatory for work belonging to an iteration. Names
use lowercase ASCII and kebab-case, without spaces or underscores. The summary
must be short, concrete, and action-oriented; avoid generic names such as
`changes`, `updates`, or `misc`.

Allowed actions are:

| Action | Purpose |
| --- | --- |
| `feature` | Add a new capability or product behavior |
| `fix` | Correct defective behavior |
| `refactor` | Improve structure without intentionally changing behavior |
| `docs` | Change documentation only |
| `test` | Change tests only |
| `chore` | Change tooling, CI, dependencies, or maintenance concerns |

Examples:

```text
feature/001-create-project
feature/001-implement-terminal-lifecycle
chore/001-configure-ci
fix/004-handle-antimeridian-distance
docs/008-reconcile-architecture
```

Before creating a branch, update local `develop` from its configured remote.
Before merging it, run the unit tests, smoke checks, and quality commands
relevant to its part of the iteration. Completing one divided branch does not
complete the iteration by itself.

## Template

```markdown
# Iteration NNN: Title

Status: Planned
Started:
Completed:

## Goal

## Dependencies

## Scope

## Out of Scope

## Tasks

- [ ] Concrete task

## Acceptance Criteria

- [ ] Verifiable criterion

## Verification

Not run; iteration has not started.

## Decisions

None yet.

## Deviations

None yet.

## Outcome

Pending.
```
