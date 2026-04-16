---
name: refit
description: Use refit when you need to sync local files or directories from paths in remote git repositories, inspect the diff for a configured update, or add/update a project-local .refit.yml configuration.
---

`refit` is a user-facing CLI for replacing local paths with contents from remote git repositories based on a `.refit.yml` file in the current directory. The agent may be asked to inspect configuration, explain what a configured update would change, or run `refit` commands on the user's behalf.

Treat `refit` commands as user-directed operations. Do not run `refit diff` or `refit run` unless the user asks you to inspect or apply updates.

## Config shape

Create `.refit.yml` in the project root:

```yaml
sources:
  - name: shared
    repo: git@github.com:user/repo.git
    updates:
      - name: skill-a
        path: plugins/skill-a
        target: skills/skill-a
      - name: skill-b
        path: plugins/skill-b
        target: skills/skill-b
```

- Each source groups updates for exactly one remote repo.
- Each source `name` and update `name` must match `^[a-z0-9_-]+$`.
- `path` and `target` must be relative paths.
- `path` and `target` must not contain `..`.
- A repo may appear in only one source group.
- Update IDs are always `source-name/update-name`.

## Inspect before applying

Use `refit diff` when you need to inspect or explain what one configured update would change before applying it:

```bash
refit diff shared/skill-a
```

- `diff` requires one exact update ID.
- If the local target does not exist yet, `refit` shows the diff against an empty directory.

## Apply updates

Use `refit run` when the user wants to apply one or more configured updates. It accepts a regex that matches update IDs:

```bash
refit run '^shared/skill-a$'
refit run '^shared/'
refit run 'skill-(a|b)$'
```

- `run` matches against `source-name/update-name`.
- Without `-y`, `refit` shows the plan and asks for confirmation.
- Use `-y` only when the intended match set is already clear.

```bash
refit run -y '^shared/skill-a$'
```
