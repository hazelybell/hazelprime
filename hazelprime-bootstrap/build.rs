#![warn(rust_2018_idioms)]

use std::env;
use std::fs::File;
use std::fmt::Write as FmtWrite;
use std::io::Write as IoWrite;
use std::path::Path;

use version_check as rustc;
use quote::quote;

/*
This code is based on the examples from:

The version_check crate documentation,
    Sergio Benitez, 2019: https://docs.rs/crate/version_check/0.9.1
Examples from the rust documentation,
    Alex Crichton, et al. 2019: https://doc.rust-lang.org/cargo/reference/build-scripts.html
*/

type AResult<T> = Result<T, Box<dyn std::error::Error>>;

fn info() -> AResult<String> {
    let mut s = String::new();
    let msg = match rustc::is_feature_flaggable() {
        Some(true) => String::from(
            "// Yes! It's a dev or nightly release!"
            ),
        Some(false) => String::from(
         "// No, it's stable or beta."   
        ),
        None => (quote!{compile_error!("Couldn't determine the rustc version.");}).to_string(),
    };
    writeln!(&mut s, "{}", msg)?;
    return Ok(s);
}

fn main() -> AResult<()> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("crate_top.rs");
    let mut f = File::create(&dest_path).unwrap();
    match rustc::is_feature_flaggable() {
        Some(true) => println!("cargo:rustc-cfg=feature=\"nightly\""),
        Some(false) => (),
        None => panic!("Couldn't determine the rustc version."),
    };
    writeln!(f, "{}", info()?)?;
    return Ok(());
}

