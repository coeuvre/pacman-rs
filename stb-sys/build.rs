use std::{
    process::Command,
    env,
    path::{Path, PathBuf}
};

static STB_COMMIT: &str = "e6afb9cbae4064da8c3e69af3ff5c4629579c1d2";

fn main() {
    let stb_repo_path = download_stb();
    compile_stb(&stb_repo_path);
}

fn download_stb() -> PathBuf {
    let out_dir = env::var("OUT_DIR").unwrap();

    let stb_git_repo_url = "https://github.com/nothings/stb";
    let stb_repo_path = Path::new(&out_dir).join("stb");

    if !stb_repo_path.exists() {
        run_command("git", &["clone", stb_git_repo_url, stb_repo_path.to_str().unwrap()]);
        run_command_in_dir("git", &["checkout", STB_COMMIT], &stb_repo_path);
    }

    stb_repo_path
}

fn compile_stb(stb_repo_path: &PathBuf) {
    let crate_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    let mut build = cc::Build::new();
    let include_flag = format!("-I{}", stb_repo_path.to_str().unwrap());
    build.warnings(false).static_flag(true).flag(&include_flag);

    if cfg!(feature = "image") {
        build.file(crate_path.join("stb_image.c"));
    }

    build.compile("stb");
}

fn run_command(cmd: &str, args: &[&str]) {
    do_run_command(cmd, Command::new(cmd).args(args))
}

fn run_command_in_dir<P: AsRef<Path>>(cmd: &str, args: &[&str], current_dir: P) {
    do_run_command(cmd, Command::new(cmd).args(args).current_dir(current_dir))
}

fn do_run_command(cmd: &str, command: &mut Command) {
    match command.output() {
        Ok(output) => {
            if !output.status.success() {
                let error = std::str::from_utf8(&output.stderr).unwrap();
                panic!("Command '{}' failed: {}", cmd, error);
            }
        }
        Err(error) => {
            panic!("Error running command '{}': {:#}", cmd, error);
        }
    }
}
