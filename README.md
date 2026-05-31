# 0jot

Turns `@todo:` code markers into GitHub issues, and closes them when the marker
is removed. Second tool in the `0nybl` fleet. Never modifies code.

## Marker

    // @todo: short title
    //   optional indented body line
    //   another body line

The title is hashed into a fingerprint embedded in the issue body, so issues
match markers across runs regardless of file/line.

## CLI

    0jot plan --repo . --issues issues.json --out actions.json

`plan` scans the repo, diffs markers against existing open `todo` issues, and
writes `{create, close}` to `actions.json`. It performs no network calls.

## How it runs

`action/0jot.yml` runs on push to the default branch: it fetches open `todo`
issues via `gh`, runs `0jot plan`, then creates/closes issues with `gh`.
Issues are attributed to the dedicated `0jot[bot]` GitHub App.

Cargo package/lib `jot`; binary `0jot`.
