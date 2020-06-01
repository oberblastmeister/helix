// mod editor;
mod component;

// use editor::Editor;

use argh::FromArgs;
use std::{env, path::PathBuf};

use anyhow::Error;

#[derive(FromArgs)]
/// A post-modern text editor.
pub struct Args {
    #[argh(positional)]
    files: Vec<PathBuf>,
}

fn main() -> Result<(), Error> {
    let args: Args = argh::from_env();
    // let mut editor = Editor::new(args)?;

    // editor.run()?;

    Ok(())
}