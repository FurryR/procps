// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use uutests::new_ucmd;
use uutests::util::TestScenario;
use uutests::util_name;

#[test]
fn test_no_args() {
    new_ucmd!().fails().code_is(1).no_output();
}

#[test]
fn test_invalid_arg() {
    new_ucmd!().arg("--definitely-invalid").fails().code_is(1);
}

#[test]
#[cfg(target_os = "linux")]
#[ignore = "fails in CI"]
fn test_find_init() {
    new_ucmd!().arg("init").succeeds();
}

#[test]
#[cfg(target_os = "linux")]
fn test_find_kthreadd_only_with_w_flag() {
    new_ucmd!().arg("kthreadd").fails();
    new_ucmd!().arg("-w").arg("kthreadd").succeeds();
}

#[test]
#[cfg(target_os = "linux")]
fn test_no_pid_found() {
    new_ucmd!().arg("NO_THIS_PROGRAM").fails().code_is(1);
}

#[test]
#[cfg(target_os = "linux")]
fn test_quiet() {
    new_ucmd!()
        .arg("kthreadd")
        .arg("-q")
        .arg("-w")
        .succeeds()
        .no_output();
}

#[test]
#[cfg(target_os = "linux")]
fn test_check_root_accepted() {
    new_ucmd!()
        .arg("-w")
        .arg("--check-root")
        .arg("kthreadd")
        .succeeds();
}

#[test]
#[cfg(target_os = "linux")]
fn test_single_shot() {
    for arg in ["-s", "--single-shot"] {
        let binding = new_ucmd!()
            .args(&[arg, "-w", "kthreadd", "kthreadd", "kthreadd"])
            .succeeds();
        let output = binding.stdout_str().trim_end();

        let pids = output.split(' ').collect::<Vec<_>>();
        let first = pids[0];

        let result = pids.iter().all(|it| *it == first);

        assert!(result);
        assert_eq!(pids.len(), 3);
    }
}

#[test]
#[cfg(target_os = "linux")]
fn test_omit_pid() {
    for arg in ["-o=1000", "--omit-pid=1000"] {
        new_ucmd!().arg(arg).arg("-w").arg("kthreadd").succeeds();
    }
}

#[test]
#[cfg(target_os = "linux")]
fn test_separator() {
    use regex::Regex;

    let re = &Regex::new("^[1-9][0-9]*separator[1-9][0-9]*\n$").unwrap();

    for arg in ["-S", "-d", "--separator"] {
        new_ucmd!()
            .args(&[arg, "separator", "-w", "kthreadd", "kthreadd"])
            .succeeds()
            .stdout_matches(re);
    }
}

#[test]
#[cfg(target_os = "linux")]
fn test_threads() {
    let main_tid = unsafe { uucore::libc::gettid() };
    std::thread::spawn(move || {
        let argv0 = std::env::args().next().unwrap();
        let our_name = std::path::Path::new(argv0.as_str())
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();

        let new_thread_tid = unsafe { uucore::libc::gettid() };
        new_ucmd!()
            .arg("-t")
            .arg(our_name)
            .succeeds()
            .stdout_contains(main_tid.to_string())
            .stdout_contains(new_thread_tid.to_string());
    })
    .join()
    .unwrap();
}

#[test]
#[cfg(target_os = "linux")]
fn test_script() {
    use std::os::unix::fs::PermissionsExt;

    let temp_dir = tempfile::tempdir().unwrap();
    let script_path = temp_dir
        .path()
        .join("dummy_test_script_with_very_long_name");
    std::fs::write(&script_path, "#!/bin/sh\nsleep 2").unwrap();
    std::fs::set_permissions(&script_path, std::fs::Permissions::from_mode(0o755)).unwrap();

    let mut directly_executed_child = std::process::Command::new(&script_path).spawn().unwrap();
    new_ucmd!()
        .arg("dummy_test_script_with_very_long_name")
        .fails();
    new_ucmd!()
        .args(&["-x", "dummy_test_script_with_very_long_name"])
        .succeeds()
        .stdout_contains(directly_executed_child.id().to_string());
    directly_executed_child.kill().unwrap();
    directly_executed_child.wait().unwrap();

    let mut executed_via_sh_child = std::process::Command::new("/bin/sh")
        .arg(&script_path)
        .spawn()
        .unwrap();
    new_ucmd!()
        .arg("dummy_test_script_with_very_long_name")
        .fails();
    new_ucmd!()
        .args(&["-x", "dummy_test_script_with_very_long_name"])
        .fails();
    executed_via_sh_child.kill().unwrap();
    executed_via_sh_child.wait().unwrap();
}
