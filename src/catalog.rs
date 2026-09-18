use std::{
    collections::HashMap,
    env, fs,
    io::{self, Read},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

const HELP_TIMEOUT: Duration = Duration::from_millis(800);
const HELP_LIMIT: u64 = 32 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CommandEntry {
    pub(crate) name: String,
    pub(crate) path: PathBuf,
}

pub(crate) fn scan_installed_commands() -> Vec<CommandEntry> {
    env::var_os("PATH")
        .map(|path| scan_path(&path))
        .unwrap_or_default()
}

pub(crate) fn inspect_help(path: &Path) -> io::Result<String> {
    let mut child = Command::new(path)
        .arg("--help")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| io::Error::other("help stdout was unavailable"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| io::Error::other("help stderr was unavailable"))?;
    let stdout_reader = thread::spawn(move || read_limited(stdout));
    let stderr_reader = thread::spawn(move || read_limited(stderr));
    let deadline = Instant::now() + HELP_TIMEOUT;
    let timed_out = loop {
        if child.try_wait()?.is_some() {
            break false;
        }
        if Instant::now() >= deadline {
            child.kill()?;
            child.wait()?;
            break true;
        }
        thread::sleep(Duration::from_millis(10));
    };

    let (mut output, stdout_truncated) = join_reader(stdout_reader)?;
    let (stderr, stderr_truncated) = join_reader(stderr_reader)?;
    if !stderr.is_empty() {
        if !output.is_empty() {
            output.push(b'\n');
        }
        output.extend(stderr);
    }

    let mut help = String::from_utf8_lossy(&output).trim().to_owned();
    if help.is_empty() {
        return Err(io::Error::other(if timed_out {
            "help command timed out"
        } else {
            "help command returned no text"
        }));
    }
    if timed_out {
        help.push_str("\n\n[Stopped after 800 ms]");
    } else if stdout_truncated || stderr_truncated {
        help.push_str("\n\n[Output truncated at 64 KiB]");
    }
    Ok(help)
}

fn read_limited(reader: impl Read) -> io::Result<(Vec<u8>, bool)> {
    let mut output = Vec::new();
    reader.take(HELP_LIMIT + 1).read_to_end(&mut output)?;
    let truncated = output.len() as u64 > HELP_LIMIT;
    output.truncate(HELP_LIMIT as usize);
    Ok((output, truncated))
}

fn join_reader(
    reader: thread::JoinHandle<io::Result<(Vec<u8>, bool)>>,
) -> io::Result<(Vec<u8>, bool)> {
    reader
        .join()
        .map_err(|_| io::Error::other("help reader stopped unexpectedly"))?
}

fn scan_path(path: &std::ffi::OsStr) -> Vec<CommandEntry> {
    let mut commands = HashMap::new();

    for directory in env::split_paths(path) {
        let directory = if directory.as_os_str().is_empty() {
            Path::new(".")
        } else {
            directory.as_path()
        };
        for (name, path) in executable_files(directory) {
            commands.entry(name).or_insert(path);
        }
    }

    let mut commands = commands
        .into_iter()
        .map(|(name, path)| CommandEntry { name, path })
        .collect::<Vec<_>>();
    commands.sort_unstable_by(|left, right| left.name.cmp(&right.name));
    commands
}

fn executable_files(directory: &Path) -> Vec<(String, PathBuf)> {
    let Ok(entries) = fs::read_dir(directory) else {
        return Vec::new();
    };
    let mut commands = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let metadata = entry.metadata().ok()?;
            let name = entry.file_name().into_string().ok()?;
            (metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
                .then(|| (name, entry.path()))
        })
        .collect::<Vec<_>>();
    commands.sort_unstable_by(|left, right| left.0.cmp(&right.0));
    commands
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use super::*;

    #[test]
    fn scans_executables_and_keeps_first_path_match() -> io::Result<()> {
        let root = env::temp_dir().join(format!(
            "kepler-catalog-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let first = root.join("first");
        let second = root.join("second");
        fs::create_dir_all(&first)?;
        fs::create_dir_all(&second)?;
        make_file(&first.join("alpha"), 0o755)?;
        make_file(&second.join("alpha"), 0o755)?;
        make_file(&second.join("beta"), 0o755)?;
        make_file(&second.join("ignored"), 0o644)?;

        let path = env::join_paths([&first, &second]).unwrap();
        let commands = scan_path(&path);

        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0].path, first.join("alpha"));
        assert_eq!(commands[1].name, "beta");
        fs::remove_dir_all(root)
    }

    #[test]
    fn inspects_bounded_help_output() -> io::Result<()> {
        let help = inspect_help(&env::current_exe()?)?;

        assert!(help.contains("Usage"));
        Ok(())
    }

    fn make_file(path: &Path, mode: u32) -> io::Result<()> {
        fs::write(path, [])?;
        fs::set_permissions(path, fs::Permissions::from_mode(mode))
    }
}
