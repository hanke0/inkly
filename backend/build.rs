use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-env-changed=SKIP_FRONTEND_BUILD");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let frontend_dir = manifest_dir.join("../frontend");
    let dist_dir = frontend_dir.join("dist");
    println!("cargo:rustc-env=FRONTEND_DIST_DIR={}", dist_dir.display());

    // Rebuild frontend when these directories change.
    // (Using directories is enough for the scaffold; Cargo will re-run on any file changes under them.)
    println!(
        "cargo:rerun-if-changed={}",
        frontend_dir.join("src").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        frontend_dir.join("index.html").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        frontend_dir.join("package.json").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        frontend_dir.join("package-lock.json").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        frontend_dir.join("vite.config.ts").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        frontend_dir.join("tailwind.config.cjs").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        frontend_dir.join("postcss.config.js").display()
    );

    let skip = env::var("SKIP_FRONTEND_BUILD").ok().as_deref() == Some("1");
    if skip {
        if !dist_dir.exists() {
            panic!(
                "SKIP_FRONTEND_BUILD=1 but frontend dist not found at {}",
                dist_dir.display()
            );
        }
        return;
    }

    // Install frontend deps from lockfile, then build so the backend binary embeds `frontend/dist`.
    let ci_status = Command::new("npm")
        .arg("ci")
        .current_dir(&frontend_dir)
        .status()
        .expect("failed to spawn npm ci");

    if !ci_status.success() {
        panic!(
            "npm ci failed (exit code {:?})",
            ci_status.code().unwrap_or(-1)
        );
    }

    let status = Command::new("npm")
        .arg("run")
        .arg("build")
        .current_dir(&frontend_dir)
        .status()
        .expect("failed to spawn npm run build");

    if !status.success() {
        panic!(
            "frontend build failed (exit code {:?})",
            status.code().unwrap_or(-1)
        );
    }

    if !dist_dir.exists() {
        panic!(
            "frontend build did not produce dist dir at {}",
            dist_dir.display()
        );
    }
}
