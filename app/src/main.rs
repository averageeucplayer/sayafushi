#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    sayafushi_lib::run()?;

    Ok(())
}

