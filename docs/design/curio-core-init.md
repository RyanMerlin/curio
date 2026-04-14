# Curio Core Init Contract

This document defines the deterministic Confluence tree behavior for `curio init`.

## Goal

Every successful `curio init` must leave Confluence in one exact managed shape under the configured space:

- `CURIO`
  - `Published`
  - `Intake`
  - `Staged`
  - `Review`
  - `Config`

And under `Config`:

- `Northstar`
- `CURIO Readme`
- `Settings`

No exceptions.

## Write Boundary

- `CURIO_SPACE_KEY` selects the Confluence space.
- `CURIO` is a page inside that space, not a Confluence space.
- `CURIO_CONFLUENCE_PARENT_PAGE_ID` stores the managed root page id.
- All Curio-managed writes must remain within that page subtree.

## Init Modes

### Normal init

`curio init`

Behavior:

- create missing local wiki scaffold
- resolve or create the `CURIO` root page
- resolve or create the required top-level pages
- resolve or create the required `Config` child pages
- write required body content
- validate the live Confluence tree

### Destructive reset init

`curio init --reset --confirm-nuke`

Behavior:

- perform all normal init behavior
- before rebuild, delete all descendants under the managed `CURIO` root page
- do not delete anything outside the managed `CURIO` subtree
- rebuild the exact required tree
- validate the live Confluence tree

`--confirm-nuke` is mandatory whenever `--reset` is used.

## Deterministic Root Resolution

Init must resolve the root in this order:

1. stored `CURIO_CONFLUENCE_PARENT_PAGE_ID`
2. exact `CURIO` page title lookup inside the configured space
3. create a new `CURIO` page in the configured space

The resolved root is valid only if:

- title is exactly `CURIO`
- page status is current
- page belongs to the configured Confluence space

Otherwise Curio must reject it and continue resolution.

## Required Page Content

The following pages must have non-empty meaningful body content:

- `CURIO`
- `Published`
- `Intake`
- `Staged`
- `Review`
- `Config`
- `Northstar`
- `CURIO Readme`
- `Settings`

`CURIO` must also have:

- the hero image attachment
- a landing-page body with links to the five top-level sections

## Post-Init Validation

After every init run, Curio must validate:

1. the `CURIO` root exists in the configured Confluence space
2. the stored root id resolves to that page
3. the root title is exactly `CURIO`
4. the hero image attachment exists on the root page
5. the direct children of `CURIO` are exactly:
   - `Published`
   - `Intake`
   - `Staged`
   - `Review`
   - `Config`
6. each required child is a direct child of `CURIO`
7. the direct children of `Config` are exactly:
   - `Northstar`
   - `CURIO Readme`
   - `Settings`
8. each required `Config` child is a direct child of `Config`
9. each required page body is meaningfully populated

If any validation check fails, `curio init` must exit non-zero and print a concrete defect report.

## Deletion Scope

When reset is requested, Curio may delete:

- descendants of the managed `CURIO` root page

Curio may not delete:

- the Confluence space
- pages outside the managed `CURIO` root page
- unrelated top-level pages in the same space

## Sync Relationship

`init` owns deterministic tree creation and reset.

`sync` must assume the managed tree exists and repair only within that contract.

If `sync` finds a missing or malformed managed tree, it should call the same deterministic tree routine rather than improvising structure.

## Logging

Init should report:

- target space
- resolved root page id
- whether the root was reused or created
- number of descendants deleted during reset
- validation result

No silent repair.
