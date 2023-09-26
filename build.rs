use std::env::var;

use fs_extra::{copy_items, dir::CopyOptions};

fn main() -> Result<(), String> {
    // Rerun if there are any changes in our resource folder
    println!("cargo:rerun-if-changed=res/**");

    let copy_options = CopyOptions::new().overwrite(true);
    copy_items(
        &["res/"],
        var("OUT_DIR").map_err(|x| x.to_string())?,
        &copy_options,
    )
    .map_err(|x| x.to_string())?;

    Ok(())
}
