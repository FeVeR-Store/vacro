use std::{env, path::PathBuf, process::Command};

use serde::Deserialize;

use crate::__private::error::{Error, Result};

#[derive(Deserialize)]
pub(crate) struct Metadata {
    pub target_directory: PathBuf,
}

fn cargo() -> Command {
    match env::var_os("CARGO") {
        Some(cargo) => Command::new(cargo),
        None => Command::new("cargo"),
    }
}

pub fn metadata() -> Result<Metadata> {
    let output = cargo()
        .arg("metadata")
        .arg("--no-deps")
        .arg("--format-version=1")
        .output()
        .map_err(Error::Cargo)?;

    serde_json::from_slice(&output.stdout).map_err(|err| {
        print!("{}", String::from_utf8_lossy(&output.stderr));
        Error::Metadata(err)
    })
}
