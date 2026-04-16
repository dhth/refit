use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

const SKILL_MD: &[u8] = include_bytes!("assets/SKILL.md");

#[derive(Debug, thiserror::Error)]
pub enum WriteSkillError {
    #[error("couldn't get current working directory: {0}")]
    CouldntGetCwd(#[from] std::io::Error),
    #[error("skill file already exists: {0}")]
    SkillFileAlreadyExists(String),
    #[error("couldn't create skill directory '{path}': {source}")]
    CouldntCreateSkillDirectory {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("couldn't write skill file '{path}': {source}")]
    CouldntWriteSkillFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

pub fn handle_write_skill() -> Result<(), WriteSkillError> {
    let cwd = std::env::current_dir()?;
    let skill_dir = cwd.join("refit");
    let skill_file = skill_dir.join("SKILL.md");

    std::fs::create_dir_all(&skill_dir).map_err(|source| {
        WriteSkillError::CouldntCreateSkillDirectory {
            path: skill_dir,
            source,
        }
    })?;

    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&skill_file)
        .map_err(|source| match source.kind() {
            std::io::ErrorKind::AlreadyExists => {
                WriteSkillError::SkillFileAlreadyExists(skill_file.display().to_string())
            }
            _ => WriteSkillError::CouldntWriteSkillFile {
                path: skill_file.clone(),
                source,
            },
        })?;

    file.write_all(SKILL_MD)
        .map_err(|source| WriteSkillError::CouldntWriteSkillFile {
            path: skill_file.clone(),
            source,
        })?;

    println!(
        "skill written to {}; you can adapt it as you see fit",
        skill_file.to_string_lossy()
    );

    Ok(())
}
