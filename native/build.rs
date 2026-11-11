use std::{env, fs, path::PathBuf, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=../logo.ico");
    println!("cargo:rerun-if-env-changed=WindowsSdkDir");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let sdk = env::var_os("WindowsSdkDir")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env::var_os("ProgramFiles(x86)").expect("Program Files"))
                .join("Windows Kits/10")
        });
    let mut versions: Vec<_> = fs::read_dir(sdk.join("bin"))
        .expect("Windows SDK")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect();
    versions.sort();
    let compiler = versions
        .into_iter()
        .rev()
        .map(|path| path.join("x64/rc.exe"))
        .find(|path| path.exists())
        .expect("Windows resource compiler");
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let icon = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap())
        .join("../logo.ico")
        .canonicalize()
        .unwrap();
    let script = out.join("mhsb.rc");
    let resource = out.join("mhsb.res");
    let icon_path = icon
        .to_string_lossy()
        .trim_start_matches(r"\\?\")
        .replace('\\', "/");
    fs::write(
        &script,
        format!(
            r#"
1 ICON "{icon_path}"
1 VERSIONINFO
FILEVERSION 0,2,1,0
PRODUCTVERSION 0,2,1,0
FILEOS 0x40004L
FILETYPE 0x1L
BEGIN
 BLOCK "StringFileInfo"
 BEGIN
  BLOCK "040904B0"
  BEGIN
   VALUE "FileDescription", "MHSB - Making Hair Sims Better\0"
   VALUE "ProductName", "MHSB\0"
   VALUE "FileVersion", "0.2.1\0"
   VALUE "ProductVersion", "0.2.1\0"
  END
 END
 BLOCK "VarFileInfo"
 BEGIN
  VALUE "Translation", 0x0409, 1200
 END
END
"#
        ),
    )
    .unwrap();
    let status = Command::new(compiler)
        .arg("/nologo")
        .arg("/fo")
        .arg(&resource)
        .arg(&script)
        .status()
        .expect("Run resource compiler");
    assert!(status.success(), "Resource compilation failed");
    println!("cargo:rustc-link-arg-bin=mhsb={}", resource.display());
}
