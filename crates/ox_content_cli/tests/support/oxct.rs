//! Hermetic `oxct` runner shared by the integration tests.
//!
//! Every invocation gets a private HOME/XDG tree and a PATH that only holds
//! the fake tools a test registers, so no test can reach the developer's
//! editors, package managers or user configuration.

#![allow(dead_code, reason = "Each test binary uses a different subset of the helpers.")]

use serde_json::Value;
use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

pub struct Project {
    _temporary: tempfile::TempDir,
    root: PathBuf,
}

pub struct Run {
    pub code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

impl Project {
    pub fn new() -> Self {
        let temporary = tempfile::tempdir().unwrap();
        // The CLI reports paths below the real working directory, not a symlinked temp root.
        let root = temporary.path().canonicalize().unwrap();
        for directory in ["work", "home", "bin"] {
            fs::create_dir(root.join(directory)).unwrap();
        }
        Self { _temporary: temporary, root }
    }

    /// The working directory commands run in.
    pub fn work(&self) -> PathBuf {
        self.root.join("work")
    }

    pub fn home(&self) -> PathBuf {
        self.root.join("home")
    }

    pub fn file(&self, path: &str) -> PathBuf {
        self.work().join(path)
    }

    pub fn write(&self, path: &str, content: &str) -> PathBuf {
        let file = self.file(path);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(&file, content).unwrap();
        file
    }

    pub fn read(&self, path: &str) -> String {
        fs::read_to_string(self.file(path)).unwrap()
    }

    pub fn exists(&self, path: &str) -> bool {
        self.file(path).exists()
    }

    /// Sorted names inside a directory relative to the working directory.
    pub fn list(&self, path: &str) -> Vec<String> {
        let mut names: Vec<_> = fs::read_dir(self.file(path))
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    /// Register an executable shell script as the only `name` on PATH.
    #[cfg(unix)]
    pub fn tool(&self, name: &str, script: &str) {
        use std::os::unix::fs::PermissionsExt;
        let file = self.root.join("bin").join(name);
        fs::write(&file, format!("#!/bin/sh\n{script}\n")).unwrap();
        fs::set_permissions(file, fs::Permissions::from_mode(0o755)).unwrap();
    }

    pub fn command(&self, args: &[&str]) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_oxct"));
        command
            .args(args)
            .current_dir(self.work())
            .env_clear()
            .env("PATH", self.root.join("bin"))
            .env("HOME", self.home())
            .env("USERPROFILE", self.home())
            .env("APPDATA", self.home().join("AppData"))
            .env("XDG_CONFIG_HOME", self.home().join(".config"))
            .env("XDG_DATA_HOME", self.home().join(".local/share"));
        // Windows resolves system libraries through these.
        for name in ["SystemRoot", "SYSTEMROOT", "TEMP", "TMP"] {
            if let Some(value) = std::env::var_os(name) {
                command.env(name, value);
            }
        }
        command
    }

    pub fn run(&self, args: &[&str]) -> Run {
        Run::of(self.command(args), None)
    }

    pub fn run_in(&self, directory: &str, args: &[&str]) -> Run {
        let mut command = self.command(args);
        command.current_dir(self.file(directory));
        Run::of(command, None)
    }

    pub fn run_with(&self, args: &[&str], input: &[u8]) -> Run {
        Run::of(self.command(args), Some(input))
    }
}

impl Run {
    pub fn of(mut command: Command, input: Option<&[u8]>) -> Self {
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        if let Some(input) = input {
            // A command that rejects its arguments may exit before reading.
            let _ = child.stdin.as_mut().unwrap().write_all(input);
        }
        drop(child.stdin.take());
        let output = child.wait_with_output().unwrap();
        Self {
            code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }
    }

    #[track_caller]
    pub fn json(&self) -> Value {
        serde_json::from_str(&self.stdout).unwrap_or_else(|error| {
            panic!("{error}\nstdout: {}\nstderr: {}", self.stdout, self.stderr)
        })
    }

    #[track_caller]
    pub fn success(&self) -> &Self {
        assert_eq!(self.code, Some(0), "stdout: {}\nstderr: {}", self.stdout, self.stderr);
        self
    }

    /// A rejected command exits 1, explains itself on stderr and prints no report.
    #[track_caller]
    pub fn rejected(&self, reason: &str) -> &Self {
        assert_eq!(self.code, Some(1), "stdout: {}\nstderr: {}", self.stdout, self.stderr);
        assert!(self.stderr.contains(reason), "Missing {reason:?} in stderr: {}", self.stderr);
        assert!(self.stdout.is_empty(), "Unexpected stdout: {}", self.stdout);
        self
    }

    /// `(file, ruleId, line, column)` for every reported diagnostic, in report order.
    #[track_caller]
    pub fn diagnostics(&self) -> Vec<(String, String, u64, u64)> {
        self.json()["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| {
                (
                    entry["file"].as_str().unwrap().to_string(),
                    entry["ruleId"].as_str().unwrap().to_string(),
                    entry["line"].as_u64().unwrap(),
                    entry["column"].as_u64().unwrap(),
                )
            })
            .collect()
    }

    /// Rule ids in report order.
    #[track_caller]
    pub fn rules(&self) -> Vec<String> {
        self.diagnostics().into_iter().map(|(_, rule, _, _)| rule).collect()
    }
}
