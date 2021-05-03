use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct Configure {
    project_name: String,
    cmd: Command,
}

impl Configure {
    pub fn new<S: AsRef<str>, P: AsRef<Path>>(project_name: S, src_dir: P) -> Self {
        let mut cmd = Command::new("./configure");
        cmd.current_dir(src_dir);
        Self {
            project_name: project_name.as_ref().to_string(),
            cmd,
        }
    }

    pub fn src_dir<P: AsRef<Path>>(&mut self, src_dir: P) {
        self.cmd.current_dir(src_dir);
    }

    pub fn prefix<P: AsRef<Path>>(&mut self, path: P) {
        self.cmd
            .arg(&format!("--prefix={}", path.as_ref().display()));
    }

    pub fn with_pkg_prefix<S: AsRef<str>, P: AsRef<Path>>(&mut self, pkg: S, path: P) {
        self.cmd.arg(&format!(
            "--with-{}-prefix={}",
            pkg.as_ref(),
            path.as_ref().display()
        ));
    }

    pub fn enable(&mut self, feature: &str) {
        self.cmd.arg(&format!("--enable-{}", feature));
    }

    pub fn disable(&mut self, feature: &str) {
        self.cmd.arg(&format!("--disable-{}", feature));
    }

    pub fn configure(self) {
        run_command(self.cmd, &format!("configuring {}", self.project_name));
    }
}

pub struct Project {
    project_name: String,
    src_dir: PathBuf,
}

impl Project {
    pub fn new<S: AsRef<str>, P: AsRef<Path>>(project_name: S, src_dir: P) -> Self {
        Self {
            project_name: project_name.as_ref().to_string(),
            src_dir: src_dir.as_ref().to_path_buf(),
        }
    }

    pub fn cp_src<P: AsRef<Path>>(&mut self, new_src_dir: P) {
        if new_src_dir.as_ref().exists() {
            fs::remove_dir_all(new_src_dir.as_ref()).unwrap();
        }
        fs::create_dir_all(&new_src_dir).unwrap();
        cp_r(&self.src_dir, new_src_dir.as_ref());
        self.src_dir = new_src_dir.as_ref().to_path_buf();
    }

    pub fn mv_src<P: AsRef<Path>>(&mut self, new_src_dir: P) {
        let old_src_dir = self.src_dir.clone();
        self.cp_src(new_src_dir);
        fs::remove_dir_all(&old_src_dir).unwrap();
    }

    pub fn autogen(&self) {
        let mut cmd = Command::new("./autogen.sh");
        cmd.current_dir(&self.src_dir);
        run_command(
            cmd,
            &format!(
                "generating the configure script for {} using autogen.sh",
                self.project_name
            ),
        );
    }

    pub fn configure(&self) -> Configure {
        Configure::new(&self.project_name, &self.src_dir)
    }

    pub fn make(&self) {
        let mut cmd = Command::new("make");
        cmd.arg(&format!("-j{}", num_cpus::get()));
        cmd.current_dir(&self.src_dir);
        run_command(cmd, &format!("building {}", self.project_name));
    }

    pub fn check(&self) {
        let mut cmd = Command::new("make");
        cmd.arg("check");
        cmd.current_dir(&self.src_dir);
        run_command(cmd, &format!("checking {}", self.project_name));
    }

    pub fn install(&self) {
        let mut cmd = Command::new("make");
        cmd.arg("install");
        cmd.current_dir(&self.src_dir);
        run_command(cmd, &format!("installing {}", self.project_name));
    }
}

pub struct Artifacts {
    pub install_dir: PathBuf,
    pub bin_dir: PathBuf,
    pub include_dir: PathBuf,
    pub lib_dir: PathBuf,
    pub libs: Vec<&'static str>,
}

impl Artifacts {
    pub fn print_cargo_metadata(&self) {
        println!("cargo:rustc-link-search=native={}", self.lib_dir.display());
        for lib in self.libs.iter() {
            println!("cargo:rustc-link-lib=static={}", lib);
        }
        println!("cargo:include={}", self.include_dir.display());
        println!("cargo:lib={}", self.lib_dir.display());
    }
}

pub fn run_command(mut command: Command, desc: &str) {
    println!("running '{:?}'", command);
    let status = command.status().unwrap();
    if !status.success() {
        panic!(
            "\n\nError: {}\n  Command: {:?}\n  Exit status: {}\n\n",
            desc, command, status
        );
    }
}

fn cp_r(src: &Path, dst: &Path) {
    for f in fs::read_dir(src).unwrap() {
        let f = f.unwrap();
        let path = f.path();
        let name = path.file_name().unwrap();

        // Skip git metadata as it's been known to cause issues and
        // otherwise shouldn't be required
        if name.to_str() == Some(".git") {
            continue;
        }

        let dst = dst.join(name);
        if f.file_type().unwrap().is_dir() {
            fs::create_dir_all(&dst).unwrap();
            cp_r(&path, &dst);
        } else {
            let _ = fs::remove_file(&dst);
            fs::copy(&path, &dst).unwrap();
        }
    }
}
