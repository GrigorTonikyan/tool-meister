use anyhow::Result;
use vergen::{BuildBuilder, CargoBuilder, Emitter};
use vergen_gix::GixBuilder;

fn main() -> Result<()> {
    let build = BuildBuilder::all_build()?;
    // let gix = GixBuilder::all_git()?;
    let cargo = CargoBuilder::all_cargo()?;
    Emitter::default()
        .add_instructions(&build)?
        // .add_instructions(&gix)?
        .add_instructions(&cargo)?
        .emit()
}
