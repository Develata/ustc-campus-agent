use super::*;
use ustc_campus_agent_core::invocation::Sha256Digest;

fn document(header: &str, body: &str) -> Vec<u8> {
    format!("---\n{header}\n---\n{body}").into_bytes()
}
fn parse(header: &str) -> Result<ParsedSkill, SkillError> {
    ParsedSkill::parse(
        "campus-help",
        &document(header, "# 指引\nLower-trust guidance."),
    )
}

#[test]
fn standard_multiline_metadata_and_readonly_body() {
    let source = document(
        "name: campus-help\ndescription: >-\n  Explain campus\n  procedures.\nlicense: MIT\ncompatibility: 'Requires a campus directory'\nmetadata:\n  author: 'Campus students'\n  version: \"1.0\"\nallowed-tools: 'Bash(git:*) Read'",
        "# 指引\r\nDo not infer a grant.\n---\nStill Markdown.",
    );
    let skill = ParsedSkill::parse("campus-help", &source).expect("valid test fixture");
    assert_eq!(skill.name(), "campus-help");
    assert_eq!(skill.description(), "Explain campus procedures.");
    assert_eq!(skill.license(), Some("MIT"));
    assert_eq!(skill.compatibility(), Some("Requires a campus directory"));
    assert_eq!(
        skill.metadata().get("version").expect("valid test fixture"),
        "1.0"
    );
    assert_eq!(skill.advisory_allowed_tools(), Some("Bash(git:*) Read"));
    assert_eq!(
        skill.body(),
        "# 指引\r\nDo not infer a grant.\n---\nStill Markdown."
    );
    assert!(!format!("{skill:?}").contains("infer"));
}

#[test]
fn crlf_literal_and_quoted_yaml_values_are_supported() {
    let text = "---\r\nname: campus-help\r\ndescription: |\r\n  Line one: # literal\r\n  第二行\r\nmetadata: {author: 'It''s valid'}\r\n---\r\nBody\r\n";
    let skill = ParsedSkill::parse("campus-help", text.as_bytes()).expect("valid test fixture");
    assert_eq!(skill.description(), "Line one: # literal\n第二行\n");
    assert_eq!(
        skill.metadata().get("author").expect("valid test fixture"),
        "It's valid"
    );
    assert_eq!(skill.body(), "Body\r\n");
}

#[test]
fn invalid_names_and_directory_mismatch_reject() {
    for name in [
        "",
        "Upper",
        "-start",
        "end-",
        "two--hyphens",
        "two words",
        "a/b",
    ] {
        assert_eq!(
            parse(&format!("name: '{name}'\ndescription: help"))
                .expect_err("hostile fixture must reject"),
            SkillError::InvalidName
        );
    }
    assert_eq!(
        parse("name: other\ndescription: help").expect_err("hostile fixture must reject"),
        SkillError::DirectoryMismatch
    );
    assert_eq!(
        parse(&format!("name: {}\ndescription: help", "a".repeat(65)))
            .expect_err("hostile fixture must reject"),
        SkillError::InvalidName
    );
}

#[test]
fn malformed_unknown_duplicate_aliases_and_includes_reject_without_echo() {
    for header in [
        "name: campus-help\ndescription: help\nname: campus-help",
        "name: campus-help\ndescription: help\nmetadata: {author: a, author: b}",
        "name: campus-help\ndescription: &secret REDACTED-SENTINEL\nlicense: *secret",
        "name: campus-help\ndescription: [REDACTED-SENTINEL",
        "name: campus-help\ndescription: !include REDACTED-SENTINEL",
        "name: campus-help\ndescription: !env REDACTED-SENTINEL",
        "name: campus-help\ndescription: help\nexecutable: REDACTED-SENTINEL",
        "name: campus-help\ndescription: help\nmetadata: {<<: {author: a}}",
        "name: campus-help\ndescription: help\nmetadata: {author: [a]}",
        "name: campus-help\ndescription: help\nmetadata: {author: 42}",
        "name: campus-help\ndescription: help\ncompatibility: null",
        "name: campus-help\ndescription: help\nallowed-tools: null",
        "name: campus-help\ndescription: help\n...\nname: REDACTED-SENTINEL",
    ] {
        let error = parse(header).expect_err("hostile fixture must reject");
        assert_eq!(error, SkillError::InvalidFrontmatter);
        assert!(!format!("{error:?} {error}").contains("REDACTED-SENTINEL"));
    }
}

#[test]
fn utf8_frontmatter_description_compatibility_and_metadata_limits() {
    assert_eq!(
        ParsedSkill::parse("campus-help", &[0xff]).expect_err("hostile fixture must reject"),
        SkillError::InvalidUtf8
    );
    assert_eq!(
        ParsedSkill::parse("campus-help", b"name: campus-help")
            .expect_err("hostile fixture must reject"),
        SkillError::MissingFrontmatter
    );
    assert_eq!(
        ParsedSkill::parse("campus-help", &vec![b'a'; MAX_SKILL_BYTES + 1])
            .expect_err("hostile fixture must reject"),
        SkillError::TooLarge
    );
    assert_eq!(
        parse(&format!(
            "name: campus-help\ndescription: help\n#{}",
            "a".repeat(MAX_FRONTMATTER_BYTES)
        ))
        .expect_err("hostile fixture must reject"),
        SkillError::TooLarge
    );
    for description in [String::new(), " ".into(), "字".repeat(1025)] {
        assert_eq!(
            parse(&format!("name: campus-help\ndescription: '{description}'"))
                .expect_err("hostile fixture must reject"),
            SkillError::InvalidDescription
        );
    }
    assert!(
        parse(&format!(
            "name: campus-help\ndescription: '{}'",
            "字".repeat(1024)
        ))
        .is_ok()
    );
    assert_eq!(
        parse(&format!(
            "name: campus-help\ndescription: help\ncompatibility: '{}'",
            "字".repeat(501)
        ))
        .expect_err("hostile fixture must reject"),
        SkillError::InvalidCompatibility
    );
    let metadata = (0..33)
        .map(|index| format!("  key{index}: 'value'\n"))
        .collect::<String>();
    assert_eq!(
        parse(&format!(
            "name: campus-help\ndescription: help\nmetadata:\n{metadata}"
        ))
        .expect_err("hostile fixture must reject"),
        SkillError::TooManyMetadataEntries
    );
}

#[test]
fn hostile_resource_paths_reject_before_io() {
    for path in [
        "",
        "/etc/passwd",
        "../secret",
        "a/../secret",
        "./file",
        "a//file",
        "https://host/file",
        "C:\\secret",
        "\\\\host\\file",
        "a\0b",
        "a\nb",
    ] {
        assert_eq!(
            DeclaredTextResource::new(path, Sha256Digest::from_bytes(b"test"))
                .expect_err("hostile fixture must reject"),
            SkillError::InvalidResourcePath
        );
    }
}

#[cfg(unix)]
mod filesystem {
    use super::*;
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };
    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Sandbox(PathBuf);
    impl Sandbox {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!(
                "uca-skills-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&root).expect("valid test fixture");
            Self(root)
        }
        fn declare(&self, path: &str, bytes: &[u8]) -> DeclaredTextResource {
            let target = self.0.join(path);
            fs::create_dir_all(target.parent().expect("valid test fixture"))
                .expect("valid test fixture");
            fs::write(target, bytes).expect("valid test fixture");
            DeclaredTextResource::new(path, Sha256Digest::from_bytes(bytes))
                .expect("valid test fixture")
        }
    }
    impl Drop for Sandbox {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).expect("valid test fixture");
        }
    }

    #[test]
    fn declared_skill_and_lazy_reference_are_digest_checked() {
        let root = Sandbox::new();
        let skill = root.declare(
            "campus-help/SKILL.md",
            &document(
                "name: campus-help\ndescription: help",
                "See references/guide.md",
            ),
        );
        let reference = root.declare("campus-help/references/guide.md", "申办指南".as_bytes());
        let declarations = [skill, reference];
        assert_eq!(
            load_declared_skill(&root.0, &declarations, declarations[0].path())
                .expect("valid test fixture")
                .name(),
            "campus-help"
        );
        let loaded = load_declared_text_resource(&root.0, &declarations, declarations[1].path())
            .expect("valid test fixture");
        assert_eq!(loaded.text(), "申办指南");
        assert_eq!(loaded.digest(), declarations[1].digest());
        assert!(!format!("{loaded:?}").contains("申办"));
        fs::write(root.0.join(declarations[1].path()), "changed").expect("valid test fixture");
        assert_eq!(
            load_declared_text_resource(&root.0, &declarations, declarations[1].path())
                .expect_err("hostile fixture must reject"),
            SkillError::DigestMismatch
        );
        assert_eq!(
            load_declared_text_resource(&root.0, &declarations, "unlisted.md")
                .expect_err("hostile fixture must reject"),
            SkillError::UndeclaredResource
        );
        assert_eq!(
            load_declared_text_resource(
                &root.0,
                &[declarations[0].clone(), declarations[0].clone()],
                declarations[0].path()
            )
            .expect_err("hostile fixture must reject"),
            SkillError::DuplicateResource
        );
    }

    #[test]
    fn oversized_invalid_utf8_symlinks_directories_and_fifo_reject() {
        use std::os::unix::fs::symlink;
        let root = Sandbox::new();
        let big = root.declare("big.md", &vec![b'a'; MAX_RESOURCE_BYTES + 1]);
        assert_eq!(
            load_declared_text_resource(&root.0, &[big], "big.md")
                .expect_err("hostile fixture must reject"),
            SkillError::TooLarge
        );
        let invalid = root.declare("invalid.md", &[0xff]);
        assert_eq!(
            load_declared_text_resource(&root.0, &[invalid], "invalid.md")
                .expect_err("hostile fixture must reject"),
            SkillError::InvalidUtf8
        );
        let file = root.declare("real.md", b"safe");
        symlink(root.0.join("real.md"), root.0.join("link.md")).expect("valid test fixture");
        let link = DeclaredTextResource::new("link.md", file.digest().clone())
            .expect("valid test fixture");
        assert_eq!(
            load_declared_text_resource(&root.0, &[link], "link.md")
                .expect_err("hostile fixture must reject"),
            SkillError::UnsafeResource
        );
        fs::create_dir(root.0.join("dir")).expect("valid test fixture");
        symlink(root.0.join("dir"), root.0.join("linkdir")).expect("valid test fixture");
        let nested = DeclaredTextResource::new("linkdir/missing.md", file.digest().clone())
            .expect("valid test fixture");
        assert_eq!(
            load_declared_text_resource(&root.0, &[nested], "linkdir/missing.md")
                .expect_err("hostile fixture must reject"),
            SkillError::UnsafeResource
        );
        let directory =
            DeclaredTextResource::new("dir", file.digest().clone()).expect("valid test fixture");
        assert_eq!(
            load_declared_text_resource(&root.0, &[directory], "dir")
                .expect_err("hostile fixture must reject"),
            SkillError::UnsafeResource
        );
        rustix::fs::mknodat(
            rustix::fs::CWD,
            root.0.join("fifo"),
            rustix::fs::FileType::Fifo,
            rustix::fs::Mode::RUSR,
            0,
        )
        .expect("valid test fixture");
        let fifo =
            DeclaredTextResource::new("fifo", file.digest().clone()).expect("valid test fixture");
        assert_eq!(
            load_declared_text_resource(&root.0, &[fifo], "fifo")
                .expect_err("hostile fixture must reject"),
            SkillError::UnsafeResource
        );
    }
}
