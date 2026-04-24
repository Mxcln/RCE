use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let workspace_root = manifest_dir.parent().unwrap().parent().unwrap().parent().unwrap().to_path_buf();
    let ckociemba_dir = workspace_root.join("ref/kociemba/kociemba/ckociemba");
    let include_dir = ckociemba_dir.join("include");
    let wrapper_dir = manifest_dir.join("csrc");

    cc::Build::new()
        .include(&include_dir)
        .include(&wrapper_dir)
        .file(ckociemba_dir.join("coordcube.c"))
        .file(ckociemba_dir.join("cubiecube.c"))
        .file(ckociemba_dir.join("facecube.c"))
        .file(ckociemba_dir.join("search.c"))
        .file(ckociemba_dir.join("prunetable_helpers.c"))
        .file(wrapper_dir.join("rce_kociemba_wrapper.c"))
        .flag_if_supported("-std=c99")
        .flag_if_supported("-O3")
        .flag_if_supported("-D_XOPEN_SOURCE=700")
        .compile("rce_kociemba");

    println!(
        "cargo:rerun-if-changed={}",
        ckociemba_dir.join("coordcube.c").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        ckociemba_dir.join("cubiecube.c").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        ckociemba_dir.join("facecube.c").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        ckociemba_dir.join("search.c").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        ckociemba_dir.join("prunetable_helpers.c").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        include_dir.join("search.h").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        include_dir.join("coordcube.h").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        wrapper_dir.join("rce_kociemba_wrapper.h").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        wrapper_dir.join("rce_kociemba_wrapper.c").display()
    );
}
