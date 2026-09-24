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
            let (path, existed) = install_into(&home)?;
            let label = if existed { "overwritten" } else { "installed" };
            writeln!(out, "{label}\t{}", path.display())?;
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

pub fn install_into(home: &Path) -> Result<(PathBuf, bool)> {
    let path = skill_path(home);
    let dir = path.parent().ok_or_else(|| Error::Config("invalid skill path".into()))?;
    let existed = path.exists();
    fs::create_dir_all(dir).map_err(|e| Error::Config(format!("cannot install skill to {}: {e}", path.display())))?;
    let tmp = dir.join("SKILL.md.tmp");
    fs::write(&tmp, SKILL_CONTENT).map_err(|e| Error::Config(format!("cannot install skill to {}: {e}", path.display())))?;
    fs::rename(&tmp, &path).map_err(|e| Error::Config(format!("cannot install skill to {}: {e}", path.display())))?;
    Ok((path, existed))
}

fn home_dir() -> Result<PathBuf> {
    directories::BaseDirs::new()
        .map(|b| b.home_dir().to_path_buf())
        .ok_or_else(|| Error::Config("cannot determine home directory".into()))
}
