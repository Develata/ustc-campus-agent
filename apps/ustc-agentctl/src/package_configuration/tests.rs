use super::{directory::PackageDirectory, prepare};
use std::{
    fs,
    os::unix::fs::symlink,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "uca-package-preparation-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).expect("new isolated fixture");
        Self(root)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        if self.0.parent() == Some(std::env::temp_dir().as_path())
            && self.0.file_name().is_some_and(|name| {
                name.to_string_lossy()
                    .starts_with("uca-package-preparation-")
            })
        {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
}
fn skill_package(root: &Path) {
    fs::create_dir_all(root.join("skills/campus-guide")).expect("package directory");
    fs::write(
        root.join("package.json"),
        include_bytes!("../../../../market/packages/ustc.campus-guide/package.json"),
    )
    .expect("manifest");
    fs::write(
        root.join("skills/campus-guide/SKILL.md"),
        include_bytes!(
            "../../../../market/packages/ustc.campus-guide/skills/campus-guide/SKILL.md"
        ),
    )
    .expect("skill");
}
#[test]
fn prepares_checked_skill_config_without_overwrite_or_execution() {
    let fixture = Fixture::new();
    let root = fixture.0.join("package");
    skill_package(&root);
    prepare(&root).expect("prepare");
    let bytes = fs::read(root.join("configuration.json")).expect("output");
    assert!(prepare(&root).is_err());
    assert_eq!(
        bytes,
        fs::read(root.join("configuration.json")).expect("unchanged")
    );
}
#[test]
fn rejects_root_ancestor_resource_and_output_symlinks() {
    let fixture = Fixture::new();
    let root = fixture.0.join("holder/package");
    skill_package(&root);
    let root_link = fixture.0.join("root-link");
    symlink(&root, &root_link).expect("root alias");
    assert!(prepare(&root_link).is_err());
    let ancestor = fixture.0.join("ancestor-link");
    symlink(fixture.0.join("holder"), &ancestor).expect("ancestor alias");
    assert!(prepare(&ancestor.join("package")).is_err());
    assert!(prepare(&root.join("../package")).is_err());
    let source = root.join("skills/campus-guide/SKILL.md");
    let actual = fixture.0.join("source.md");
    fs::rename(&source, &actual).expect("move fixture source");
    symlink(&actual, &source).expect("leaf alias");
    assert!(prepare(&root).is_err());
    fs::remove_file(&source).expect("unlink alias");
    fs::rename(&actual, &source).expect("restore fixture");
    let skills = root.join("skills");
    let real_skills = root.join("real-skills");
    fs::rename(&skills, &real_skills).expect("move fixture directory");
    symlink(&real_skills, &skills).expect("resource directory alias");
    assert!(prepare(&root).is_err());
    fs::remove_file(&skills).expect("unlink directory alias");
    fs::rename(&real_skills, &skills).expect("restore directory");
    let victim = fixture.0.join("existing.json");
    fs::write(&victim, b"unchanged synthetic data").expect("victim fixture");
    symlink(&victim, root.join("configuration.json")).expect("output alias");
    assert!(prepare(&root).is_err());
    assert_eq!(
        fs::read(victim).expect("victim"),
        b"unchanged synthetic data"
    );
}
#[test]
fn fifo_reads_reject_without_waiting_for_a_writer() {
    let fixture = Fixture::new();
    let root = fixture.0.join("package");
    fs::create_dir(&root).expect("directory");
    rustix::fs::mknodat(
        rustix::fs::CWD,
        root.join("fifo"),
        rustix::fs::FileType::Fifo,
        rustix::fs::Mode::RUSR,
        0,
    )
    .expect("FIFO fixture");
    let directory = PackageDirectory::open(&root).expect("opened root");
    let (sender, receiver) = std::sync::mpsc::channel();
    let worker = std::thread::spawn(move || {
        sender
            .send(directory.read("fifo", 1024))
            .expect("test result");
    });
    assert!(
        receiver
            .recv_timeout(Duration::from_secs(2))
            .expect("FIFO open must not block")
            .is_err()
    );
    worker.join().expect("worker");
}
#[test]
fn a_replaced_root_path_cannot_redirect_reads_or_output() {
    let fixture = Fixture::new();
    let root = fixture.0.join("package");
    fs::create_dir(&root).expect("directory");
    fs::write(root.join("package.json"), b"opened directory").expect("fixture input");
    let directory = PackageDirectory::open(&root).expect("opened root");
    let moved = fixture.0.join("moved");
    fs::rename(&root, &moved).expect("replace pathname");
    let other = fixture.0.join("other");
    fs::create_dir(&other).expect("other directory");
    fs::write(other.join("package.json"), b"other directory").expect("other input");
    symlink(&other, &root).expect("replacement alias");
    assert_eq!(
        directory.read("package.json", 1024).expect("fixed read"),
        b"opened directory"
    );
    directory
        .create_sidecar(b"checked sidecar")
        .expect("fixed output");
    assert_eq!(
        fs::read(moved.join("configuration.json")).expect("original output"),
        b"checked sidecar\n"
    );
    assert!(!other.join("configuration.json").exists());
}
