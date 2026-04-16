---
name: refit
description: Use refit when you need to sync local files or directories from paths in remote git repositories, or add/update a project-local .refit.yml configuration.
---

`refit` replaces local paths with contents from remote git repositories based on a `.refit.yml` file in the current directory.

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

## Apply updates

Use `refit run` with a regex that matches update IDs:

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
