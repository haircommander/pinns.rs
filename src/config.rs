//! Configuration related structures
use anyhow::{bail, Result};
use clap::{AppSettings, Clap};
use getset::{CopyGetters, Getters};
use lazy_static::lazy_static;
use log::{debug, LevelFilter};
use nix::sched::CloneFlags;
use std::{env::temp_dir, fs::metadata, path::PathBuf};

lazy_static! {
    static ref TEMP_DIR: String = temp_dir().display().to_string();
}

#[derive(Clap, Getters, CopyGetters)]
#[clap(
    after_help("More info at: https://github.com/saschagrunert/pinns.rs"),
    global_setting(AppSettings::ColoredHelp),
    version(env!("VERSION"))
)]
/// A simple utility to pin Linux namespaces
pub struct Config {
    #[get_copy = "pub"]
    #[clap(
        default_value("info"),
        long("log-level"),
        possible_values(&["trace", "debug", "info", "warn", "error", "off"]),
        short("l"),
        value_name("LEVEL")
    )]
    /// The logging level of the application
    log_level: LevelFilter,

    #[get = "pub"]
    #[clap(default_value(&TEMP_DIR), long("dir"), short("d"), value_name("DIRECTORY"))]
    /// The directory for the pinned namespaces
    dir: PathBuf,

    #[clap(long("cgroup"), short("c"))]
    /// Pin the cgroup namespace
    cgroup: bool,

    #[clap(long("ipc"), short("i"))]
    /// Pin the IPC namespace
    ipc: bool,

    #[clap(long("net"), short("n"))]
    /// Pin the network namespace
    net: bool,

    #[clap(long("pid"), short("p"))]
    /// Pin the PID namespace
    pid: bool,

    #[clap(long("uts"), short("u"))]
    /// Pin the UTS namespace
    uts: bool,

    #[get = "pub"]
    #[clap(skip)]
    namespaces: Namespaces,
}

#[derive(Getters)]
pub struct Namespaces {
    #[get = "pub"]
    cgroup: Namespace,

    #[get = "pub"]
    ipc: Namespace,

    #[get = "pub"]
    net: Namespace,

    #[get = "pub"]
    pid: Namespace,

    #[get = "pub"]
    uts: Namespace,
}

impl Default for Namespaces {
    fn default() -> Self {
        Namespaces {
            cgroup: Namespace {
                name: "cgroup",
                clone_flag: CloneFlags::CLONE_NEWCGROUP,
                enabled: false,
            },
            ipc: Namespace {
                name: "ipc",
                clone_flag: CloneFlags::CLONE_NEWIPC,
                enabled: false,
            },
            net: Namespace {
                name: "net",
                clone_flag: CloneFlags::CLONE_NEWNET,
                enabled: false,
            },
            pid: Namespace {
                name: "pid",
                clone_flag: CloneFlags::CLONE_NEWPID,
                enabled: false,
            },
            uts: Namespace {
                name: "uts",
                clone_flag: CloneFlags::CLONE_NEWUTS,
                enabled: false,
            },
        }
    }
}

impl IntoIterator for &Namespaces {
    type Item = Namespace;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        vec![self.cgroup, self.ipc, self.net, self.pid, self.uts].into_iter()
    }
}

#[derive(Clone, Copy, Getters, CopyGetters)]
pub struct Namespace {
    #[get = "pub"]
    name: &'static str,

    #[get_copy = "pub"]
    enabled: bool,

    #[get_copy = "pub"]
    clone_flag: CloneFlags,
}

impl Config {
    /// Validate the configuration in their parameters
    pub fn validate(&mut self) -> Result<()> {
        self.namespaces.cgroup.enabled = self.cgroup;
        self.namespaces.ipc.enabled = self.ipc;
        self.namespaces.net.enabled = self.net;
        self.namespaces.pid.enabled = self.pid;
        self.namespaces.uts.enabled = self.uts;

        if self.namespaces().into_iter().all(|x| !x.enabled()) {
            bail!("no namespace specified for pinning")
        }

        if !self.dir().exists() {
            bail!("pin path {} does not exist", self.dir().display())
        }

        if !metadata(self.dir())?.is_dir() {
            bail!("pin path {} is not a directory", self.dir().display())
        }

        debug!("CLI provided config is valid");
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::parse()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn validate_success() -> Result<()> {
        let mut c = Config::default();
        c.cgroup = true;
        c.validate()
    }

    #[test]
    fn validate_failed_no_namespaces() {
        let c = Config::default();
        assert!(c.validate().is_err())
    }

    #[test]
    fn validate_failed_not_existing_path() {
        let mut c = Config::default();
        c.dir = PathBuf::from("/not/existing/path");
        assert!(c.validate().is_err())
    }

    #[test]
    fn validate_failed_path_not_dir() -> Result<()> {
        let mut c = Config::default();
        c.dir = NamedTempFile::new()?.path().into();
        assert!(c.validate().is_err());
        Ok(())
    }
}
