<p align="center">
  <h1 align="center">refit</h1>
  <p align="center">
    <a href="https://github.com/dhth/refit/actions/workflows/main.yml"><img alt="GitHub release" src="https://img.shields.io/github/actions/workflow/status/dhth/refit/main.yml?style=flat-square"></a>
  </p>
</p>

`refit` lets you replace local paths with contents from remote git repositories.

> [!NOTE]
> refit is alpha software. Its interface and behaviour might change in the near
> future.

⚡️ Usage
---

`refit` requires a YAML config file which looks like the following.

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

```text
$ refit -h

Usage: refit <COMMAND>

Commands:
  run   Run all updates whose names match the regex
  diff  Show the diff for exactly one update
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```
