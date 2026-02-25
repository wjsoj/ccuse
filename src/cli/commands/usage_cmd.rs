use crate::error::{Error, Result};
use std::process::Command;

pub fn run_ccusage(args: &[String]) -> Result<()> {
    // First try bunx
    let mut bunx_cmd = Command::new("bunx");
    bunx_cmd.arg("ccusage@latest");
    bunx_cmd.args(args);

    if bunx_cmd.status()?.success() {
        return Ok(());
    }

    // Fallback to npx
    let mut npx_cmd = Command::new("npx");
    npx_cmd.arg("-y");
    npx_cmd.arg("ccusage@latest");
    npx_cmd.args(args);

    let status = npx_cmd.status()?;
    if !status.success() {
        return Err(Error::CcusageError(format!(
            "ccusage failed with exit code: {}",
            status.code().unwrap_or(1)
        )));
    }

    Ok(())
}
