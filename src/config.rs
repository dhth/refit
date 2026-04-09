use crate::domain::{Config, RawConfig, ValidateConfigError};

const CONFIG_FILE_NAME: &str = ".refit.yml";

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("config file not found")]
    NotFound,
    #[error("couldn't read config file: {0}")]
    Read(#[from] std::io::Error),
    #[error("couldn't parse config file: {0}")]
    Parse(#[from] serde_saphyr::Error),
    #[error("config file is invalid: {0}")]
    Invalid(#[from] ValidateConfigError),
}

pub fn load() -> Result<Config, ConfigError> {
    let contents = std::fs::read_to_string(CONFIG_FILE_NAME).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => ConfigError::NotFound,
        _ => ConfigError::Read(e),
    })?;

    let config = parse_config_str(&contents)?;

    Ok(config)
}

pub fn parse_config_str(contents: &str) -> Result<Config, ConfigError> {
    let raw: RawConfig = serde_saphyr::from_str(contents)?;

    let config = raw.try_into()?;

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::{assert_snapshot, assert_yaml_snapshot};

    //-------------//
    //  SUCCESSES  //
    //-------------//

    #[test]
    fn parsing_correct_config_works() -> anyhow::Result<()> {
        // GIVEN
        let config_str = r#"
sources:
  - name: shared
    repo: git@github.com:dhth/graphc.git
    updates:
      - name: workflows
        path: .github/workflows
        target: .github/workflows

      - name: scripts
        path: scripts/shared
        target: scripts/shared

  - name: templates
    repo: https://github.com/dhth/another-repo.git
    updates:
      - name: basic
        path: templates/basic
        target: templates/basic
"#;
        // WHEN
        let result = parse_config_str(config_str)?;

        // THEN
        assert_yaml_snapshot!(result, @r#"
        sources:
          - name: shared
            repo: "git@github.com:dhth/graphc.git"
            updates:
              - name: workflows
                source_path: ".github/workflows"
                target_path: ".github/workflows"
              - name: scripts
                source_path: scripts/shared
                target_path: scripts/shared
          - name: templates
            repo: "https://github.com/dhth/another-repo.git"
            updates:
              - name: basic
                source_path: templates/basic
                target_path: templates/basic
        "#);

        Ok(())
    }

    //------------//
    //  FAILURES  //
    //------------//

    #[test]
    fn parsing_invalid_yaml_fails() {
        // GIVEN
        let config_str = r#"
sources:
  - name: invalid-yaml
    repo:
"#;
        // WHEN
        let result = parse_config_str(config_str).expect_err("result should've been an error");

        // THEN
        assert_snapshot!(result, @"
        couldn't parse config file: error: line 4 column 9: cannot deserialize null into string; use Option<String>
         --> <input>:4:9
          |
        2 | sources:
        3 |   - name: invalid-yaml
        4 |     repo:
          |         ^ cannot deserialize null into string; use Option<String>
        ");
    }

    #[test]
    fn parsing_config_without_sources_fails() {
        // GIVEN
        let config_str = r#"
not-sources: []
"#;

        // WHEN
        let result = parse_config_str(config_str).expect_err("result should've been an error");

        // THEN
        assert_snapshot!(result, @"
        couldn't parse config file: error: line 2 column 1: missing field `sources`
         --> <input>:2:1
          |
        1 |
        2 | not-sources: []
          | ^ missing field `sources`
        ");
    }

    #[test]
    fn parsing_config_with_empty_source_name_fails() {
        // GIVEN
        let config_str = r#"
sources:
  - name: "   "
    repo: git@github.com:dhth/graphc.git
    updates:
      - name: workflows
        path: .github/workflows
        target: .github/workflows
"#;

        // WHEN
        let result = parse_config_str(config_str).expect_err("result should've been an error");

        // THEN
        assert_snapshot!(result, @"config file is invalid: source at index 1 has an empty name (index starts at 1)");
    }

    #[test]
    fn parsing_config_with_invalid_source_name_fails() {
        // GIVEN
        let config_str = r#"
sources:
  - name: shared/tools
    repo: git@github.com:dhth/graphc.git
    updates:
      - name: workflows
        path: .github/workflows
        target: .github/workflows
"#;

        // WHEN
        let result = parse_config_str(config_str).expect_err("result should've been an error");

        // THEN
        assert_snapshot!(result, @"config file is invalid: source at index 1 has an invalid name `shared/tools` (should conform to the pattern `^[a-z0-9_-]+$`)");
    }

    #[test]
    fn parsing_config_with_duplicate_source_names_fails() {
        // GIVEN
        let config_str = r#"
sources:
  - name: shared
    repo: git@github.com:dhth/graphc.git
    updates:
      - name: workflows
        path: .github/workflows
        target: .github/workflows

  - name: shared
    repo: https://github.com/dhth/another-repo.git
    updates:
      - name: basic
        path: templates/basic
        target: templates/basic
"#;

        // WHEN
        let result = parse_config_str(config_str).expect_err("result should've been an error");

        // THEN
        assert_snapshot!(result, @"config file is invalid: duplicate source name `shared`");
    }

    #[test]
    fn parsing_config_without_repo_fails() {
        // GIVEN
        let config_str = r#"
sources:
  - name: shared
    updates:
      - name: workflows
        path: .github/workflows
        target: .github/workflows
"#;

        // WHEN
        let result = parse_config_str(config_str).expect_err("result should've been an error");

        // THEN
        assert_snapshot!(result, @"
        couldn't parse config file: error: line 4 column 5: missing field `repo`
         --> <input>:4:5
          |
        2 | sources:
        3 |   - name: shared
        4 |     updates:
          |     ^ missing field `repo`
        5 |       - name: workflows
        6 |         path: .github/workflows
          |
        ");
    }

    #[test]
    fn parsing_config_with_empty_repo_fails() {
        // GIVEN
        let config_str = r#"
sources:
  - name: shared
    repo: "  "
    updates:
      - name: workflows
        path: .github/workflows
        target: .github/workflows
"#;

        // WHEN
        let result = parse_config_str(config_str).expect_err("result should've been an error");

        // THEN
        assert_snapshot!(result, @"config file is invalid: source `shared` has an empty repository");
    }

    #[test]
    fn parsing_config_with_duplicate_repos_fails() {
        // GIVEN
        let config_str = r#"
sources:
  - name: shared
    repo: git@github.com:dhth/graphc.git
    updates:
      - name: workflows
        path: .github/workflows
        target: .github/workflows

  - name: templates
    repo: git@github.com:dhth/graphc.git
    updates:
      - name: basic
        path: templates/basic
        target: templates/basic
"#;

        // WHEN
        let result = parse_config_str(config_str).expect_err("result should've been an error");

        // THEN
        assert_snapshot!(result, @"config file is invalid: multiple sources defined for the repo `git@github.com:dhth/graphc.git`; each repo must appear in exactly one source group");
    }

    #[test]
    fn parsing_config_without_updates_fails() {
        // GIVEN
        let config_str = r#"
sources:
  - name: shared
    repo: git@github.com:dhth/graphc.git
"#;

        // WHEN
        let result = parse_config_str(config_str).expect_err("result should've been an error");

        // THEN
        assert_snapshot!(result, @"
        couldn't parse config file: error: line 4 column 5: missing field `updates`
         --> <input>:4:5
          |
        2 | sources:
        3 |   - name: shared
        4 |     repo: git@github.com:dhth/graphc.git
          |     ^ missing field `updates`
        ");
    }

    #[test]
    fn parsing_config_with_empty_update_name_fails() {
        // GIVEN
        let config_str = r#"
sources:
  - name: shared
    repo: git@github.com:dhth/graphc.git
    updates:
      - name: "   "
        path: .github/workflows
        target: .github/workflows
"#;

        // WHEN
        let result = parse_config_str(config_str).expect_err("result should've been an error");

        // THEN
        assert_snapshot!(result, @"config file is invalid: name for update at index 1 in source `shared` is empty (index starts at 1)");
    }

    #[test]
    fn parsing_config_with_invalid_update_name_fails() {
        // GIVEN
        let config_str = r#"
sources:
  - name: shared
    repo: git@github.com:dhth/graphc.git
    updates:
      - name: ci workflows
        path: .github/workflows
        target: .github/workflows
"#;

        // WHEN
        let result = parse_config_str(config_str).expect_err("result should've been an error");

        // THEN
        assert_snapshot!(result, @"config file is invalid: update at index 1 in source `shared` has an invalid name `ci workflows` (should conform to the pattern `^[a-z0-9_-]+$`)");
    }

    #[test]
    fn parsing_config_with_duplicate_update_names_in_source_fails() {
        // GIVEN
        let config_str = r#"
sources:
  - name: shared
    repo: git@github.com:dhth/graphc.git
    updates:
      - name: workflows
        path: .github/workflows
        target: .github/workflows
      - name: workflows
        path: scripts/shared
        target: scripts/shared
"#;

        // WHEN
        let result = parse_config_str(config_str).expect_err("result should've been an error");

        // THEN
        assert_snapshot!(result, @"config file is invalid: duplicate update name `workflows` in source `shared`");
    }

    #[test]
    fn parsing_config_without_path_fails() {
        // GIVEN
        let config_str = r#"
sources:
  - name: shared
    repo: git@github.com:dhth/graphc.git
    updates:
      - name: workflows
        target: .github/workflows
"#;

        // WHEN
        let result = parse_config_str(config_str).expect_err("result should've been an error");

        // THEN
        assert_snapshot!(result, @"
        couldn't parse config file: error: line 7 column 9: missing field `path`
         --> <input>:7:9
          |
        5 |     updates:
        6 |       - name: workflows
        7 |         target: .github/workflows
          |         ^ missing field `path`
        ");
    }

    #[test]
    fn parsing_config_with_empty_path_fails() {
        // GIVEN
        let config_str = r#"
sources:
  - name: shared
    repo: git@github.com:dhth/graphc.git
    updates:
      - name: workflows
        path: "   "
        target: .github/workflows
"#;

        // WHEN
        let result = parse_config_str(config_str).expect_err("result should've been an error");

        // THEN
        assert_snapshot!(result, @"config file is invalid: path for update `shared/workflows` is empty");
    }

    #[test]
    fn parsing_config_with_absolute_path_fails() {
        // GIVEN
        let config_str = r#"
sources:
  - name: shared
    repo: git@github.com:dhth/graphc.git
    updates:
      - name: workflows
        path: /tmp/workflows
        target: .github/workflows
"#;

        // WHEN
        let result = parse_config_str(config_str).expect_err("result should've been an error");

        // THEN
        assert_snapshot!(result, @"config file is invalid: path for update `shared/workflows` (`/tmp/workflows`) is not a relative path");
    }

    #[test]
    fn parsing_config_with_parent_dir_in_path_fails() {
        // GIVEN
        let config_str = r#"
sources:
  - name: shared
    repo: git@github.com:dhth/graphc.git
    updates:
      - name: workflows
        path: ../workflows
        target: .github/workflows
"#;

        // WHEN
        let result = parse_config_str(config_str).expect_err("result should've been an error");

        // THEN
        assert_snapshot!(result, @"config file is invalid: path for update `shared/workflows` (`../workflows`) must not contain `..`");
    }

    #[test]
    fn parsing_config_without_target_fails() {
        // GIVEN
        let config_str = r#"
sources:
  - name: shared
    repo: git@github.com:dhth/graphc.git
    updates:
      - name: workflows
        path: .github/workflows
"#;

        // WHEN
        let result = parse_config_str(config_str).expect_err("result should've been an error");

        // THEN
        assert_snapshot!(result, @"
        couldn't parse config file: error: line 7 column 9: missing field `target`
         --> <input>:7:9
          |
        5 |     updates:
        6 |       - name: workflows
        7 |         path: .github/workflows
          |         ^ missing field `target`
        ");
    }

    #[test]
    fn parsing_config_with_empty_target_fails() {
        // GIVEN
        let config_str = r#"
sources:
  - name: shared
    repo: git@github.com:dhth/graphc.git
    updates:
      - name: workflows
        path: .github/workflows
        target: "   "
"#;

        // WHEN
        let result = parse_config_str(config_str).expect_err("result should've been an error");

        // THEN
        assert_snapshot!(result, @"config file is invalid: target for update `shared/workflows` is empty");
    }

    #[test]
    fn parsing_config_with_absolute_target_fails() {
        // GIVEN
        let config_str = r#"
sources:
  - name: shared
    repo: git@github.com:dhth/graphc.git
    updates:
      - name: workflows
        path: .github/workflows
        target: /tmp/workflows
"#;

        // WHEN
        let result = parse_config_str(config_str).expect_err("result should've been an error");

        // THEN
        assert_snapshot!(result, @"config file is invalid: target for update `shared/workflows` (`/tmp/workflows`) is not a relative path");
    }

    #[test]
    fn parsing_config_with_parent_dir_in_target_fails() {
        // GIVEN
        let config_str = r#"
sources:
  - name: shared
    repo: git@github.com:dhth/graphc.git
    updates:
      - name: workflows
        path: .github/workflows
        target: ../workflows
"#;

        // WHEN
        let result = parse_config_str(config_str).expect_err("result should've been an error");

        // THEN
        assert_snapshot!(result, @"config file is invalid: target for update `shared/workflows` (`../workflows`) must not contain `..`");
    }

    #[test]
    fn parsing_config_with_relative_paths_without_parent_dir_works() -> anyhow::Result<()> {
        // GIVEN
        let config_str = r#"
sources:
  - name: shared
    repo: git@github.com:dhth/graphc.git
    updates:
      - name: workflows
        path: ./nested/workflows
        target: nested/workflows
"#;

        // WHEN
        let result = parse_config_str(config_str)?;

        // THEN
        assert_yaml_snapshot!(result, @r#"
        sources:
          - name: shared
            repo: "git@github.com:dhth/graphc.git"
            updates:
              - name: workflows
                source_path: "./nested/workflows"
                target_path: nested/workflows
        "#);

        Ok(())
    }
}
