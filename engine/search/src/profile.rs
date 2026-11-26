//! The browser's profile directory of a run that names none (`--profile`): a directory of the
//! run's own under the system's temp directory, removed when the run ends.
//!
//! A run may start in any working directory, a checkout of another repository among them, so it
//! leaves nothing there but the files its command names. The temp directory is
//! `std::env::temp_dir()`: `TMPDIR` where it is set.

use std::path::{Path, PathBuf};

/// A profile directory of the run's own. Dropping it removes the directory and what the browser
/// left in it.
#[derive(Debug)]
pub struct Temporary {
    path: PathBuf,
}

/// How many names are tried before the run gives up: each one is taken when it is there.
const TRIES: u32 = 64;

fn create(path: &Path) -> std::io::Result<()> {
    let mut builder = std::fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(path)
}

impl Temporary {
    /// Makes `ux-search-profile-<pid>-<n>` in the system's temp directory, for the first `n`
    /// no entry has. The directory is made here and by nobody else: a name that is there, as a
    /// directory or as a link, is not used.
    pub fn new() -> Result<Temporary, String> {
        Temporary::under(&std::env::temp_dir())
    }

    /// The same under `parent`.
    pub fn under(parent: &Path) -> Result<Temporary, String> {
        for n in 0..TRIES {
            let path = parent.join(format!("ux-search-profile-{}-{n}", std::process::id()));
            match create(&path) {
                Ok(()) => return Ok(Temporary { path }),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(error) => {
                    return Err(format!(
                        "{}: {error}: the browser's profile is made in the system's temp \
                         directory (TMPDIR) unless --profile names a directory",
                        path.display()
                    ));
                }
            }
        }
        Err(format!(
            "{}: {TRIES} profile directories of this process are there already; name one with \
             --profile",
            parent.display()
        ))
    }

    /// The directory.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for Temporary {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

/// The profile directory of a run: the one `--profile` names, which the run does not own beyond
/// what the browser does with it, or a [`Temporary`] one.
#[derive(Debug)]
pub enum Profile {
    /// `--profile <dir>`.
    Named(PathBuf),
    /// No `--profile`.
    Own(Temporary),
}

impl Profile {
    /// The profile of a run whose `--profile` is `named`.
    pub fn of(named: Option<&str>) -> Result<Profile, String> {
        match named {
            Some(directory) => Ok(Profile::Named(PathBuf::from(directory))),
            None => Temporary::new().map(Profile::Own),
        }
    }

    /// The directory.
    pub fn path(&self) -> &Path {
        match self {
            Profile::Named(path) => path,
            Profile::Own(own) => own.path(),
        }
    }
}
