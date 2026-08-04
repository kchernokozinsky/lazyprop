use anyhow::Result;
use vergen::{BuildBuilder, Emitter};

fn main() -> Result<()> {
    // Emit build metadata only (VERGEN_BUILD_DATE). We deliberately avoid git
    // info so source tarballs, shallow clones and tag-less checkouts build
    // cleanly on every platform.
    let build = BuildBuilder::all_build()?;
    Emitter::default().add_instructions(&build)?.emit()
}
