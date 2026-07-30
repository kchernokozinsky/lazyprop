use anyhow::Result;
use vergen_gix::{BuildBuilder, CargoBuilder, Emitter};

fn main() -> Result<()> {
    // Emit build and cargo metadata only. We deliberately avoid git info so
    // source tarballs, shallow clones and tag-less checkouts build cleanly on
    // every platform.
    let build = BuildBuilder::all_build()?;
    let cargo = CargoBuilder::all_cargo()?;
    Emitter::default()
        .add_instructions(&build)?
        .add_instructions(&cargo)?
        .emit()
}
