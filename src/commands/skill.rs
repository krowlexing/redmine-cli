use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::cli::SkillCommand;
use crate::error::{Error, Result};

pub const SKILL_CONTENT: &str = include_str!("../../skills/redmine/SKILL.md");

pub fn run<W: Write>(cmd: SkillCommand, out: &mut W) -> Result<()> {
    match cmd {
        SkillCommand::Install => {
            let home = home_dir()?;
            let path = install_into(&home)?;
            writeln!(out, "installed\t{}", path.display())?;
            Ok(())
        }
        SkillCommand::Show => {
            write!(out, "{SKILL_CONTENT}")?;
            Ok(())
        }
    }
}

pub fn skill_path(home: &Path) -> PathBuf {
    home.join(".agents").join("skills").join("redmine").join("SKILL.md")
}

pub fn install_into(home: &Path) -> Result<PathBuf> {
    let path = skill_path(home);
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    fs::write(&path, SKILL_CONTENT)?;
    Ok(path)
}

fn home_dir() -> Result<PathBuf> {
    directories::BaseDirs::new()
        .map(|b| b.home_dir().to_path_buf())
        .ok_or_else(|| Error::Config("cannot determine home directory".into()))
}
