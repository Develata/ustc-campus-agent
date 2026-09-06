//! PR80 P1: exercise production route layers over real HTTP, including JSON escaping.
use super::*;
use serde_json::Value;
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "uca-campus-body-limits-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("test directory");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).expect("private directory");
        Self(path)
    }
    fn composition(&self) -> AffairsComposition {
        assert!(
            std::env::var_os("USTC_SOURCE_REVIEW_MANIFEST").is_none(),
            "run this controlled fixture without an external source manifest"
        );
        let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("workspace");
        let composition = AffairsComposition::open(
            &workspace.join("fixtures/affairs/proc-011-reviewed.json"),
            &self.0.join("records.json"),
            &self.0.join("idempotency.json"),
            &self.0.join("sessions.json"),
        )
        .expect("composition");
        let manifest = composition
            .conversation_store_path
            .with_extension("sources")
            .join("sources-reviewed.json");
        crate::durable_path::ensure_secure_parent(&manifest, true).expect("source directory");
        fs::write(manifest, serde_json::to_vec(&json!({"schema":"source-review-manifest/v1","sources":[{
            "source_id":"body-limit-controlled", "title":"Controlled body-limit fixture", "url":"https://www.ustc.edu.cn/",
            "reviewer":"controlled-test", "permission_evidence":"Synthetic imports only; no network or real-source license assertion",
            "review_evidence":"Controlled HTTP body-limit regression", "minimum_interval_seconds":60
        }]})).expect("manifest JSON")).expect("manifest");
        composition
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

async fn post(
    client: &reqwest::Client,
    base: &str,
    path: &str,
    body: String,
) -> (StatusCode, Value) {
    let response = client
        .post(format!("{base}{path}"))
        .header("Origin", base)
        .header("Content-Type", "application/json")
        .header(CLIENT_PROTOCOL_MAJOR_HEADER, "1")
        .header(ADMINISTRATOR_DEMO_HEADER, ADMINISTRATOR_DEMO_CONFIRMATION)
        .body(body)
        .send()
        .await
        .expect("HTTP response");
    let status = response.status();
    (status, response.json().await.expect("JSON response"))
}
fn padded(mut body: String, limit: usize) -> String {
    assert!(body.len() <= limit);
    body.extend(std::iter::repeat_n(' ', limit + 1 - body.len()));
    body
}
fn course_request() -> Value {
    let courses = (0..64)
        .map(|index| {
            let code = format!("CONTROLLED{index}");
            let mut excerpt = format!("{code} Controlled course\n");
            excerpt.extend(std::iter::repeat_n('\u{0001}', 8192 - excerpt.len()));
            json!({"code":code,"title":"Controlled course","credits_tenths":1,
            "prerequisites":[],"tags":[],"meetings":[],"source_url":"https://www.ustc.edu.cn/",
            "source_excerpt":excerpt,"observed_at":"2026-09-06T00:00:00Z"})
        })
        .collect::<Vec<_>>();
    json!({"schema":"personal-course-request/v1","consent_this_request":true,"courses":courses,
        "completed_courses":[],"interests":[],"free_slots":[],"min_credits_tenths":1,"max_credits_tenths":2})
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn body_limits_preserve_source_course_and_skill_capacity_over_real_http() {
    let fixture = Fixture::new();
    let router = web_router_with_provider(
        Arc::new(Mutex::new(fixture.composition())),
        ChatProvider::deterministic_mock(),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("listen");
    let base = format!("http://{}", listener.local_addr().expect("address"));
    let server = tokio::spawn(async move {
        axum::serve(listener, router).await.expect("serve");
    });
    let client = reqwest::Client::builder()
        .no_proxy()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("client");

    // Each control byte expands into six JSON bytes. This exercises the legal
    // decoded 128 KiB ceiling instead of testing an oversized invalid string.
    let mut text = "controlled\n".to_owned();
    text.extend(std::iter::repeat_n('\u{0001}', 128 * 1024 - text.len()));
    let import = json!({"schema":"source-import-text/v1","text":text});
    let import_body = serde_json::to_string(&import).expect("JSON");
    assert!(import_body.len() > 16 * 1024 && import_body.len() < SOURCE_IMPORT_BODY_LIMIT);
    let (status, result) = post(
        &client,
        &base,
        "/api/v1/sources/body-limit-controlled/import",
        import_body.clone(),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{result}");
    assert_eq!(result["text"], import["text"]);

    let (status, result) = post(
        &client,
        &base,
        "/api/v1/sources/body-limit-controlled/import",
        padded(import_body, SOURCE_IMPORT_BODY_LIMIT),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(result["error"], "source_import_invalid");
    let oversized_text = "x".repeat(128 * 1024 + 1);
    let (status, result) = post(
        &client,
        &base,
        "/api/v1/sources/body-limit-controlled/import",
        serde_json::to_string(&json!({"schema":"source-import-text/v1","text":oversized_text}))
            .expect("JSON"),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(result["error"], "invalid_source_observation");

    let course_request = course_request();
    let course_body = serde_json::to_string(&course_request).expect("JSON");
    assert!(course_body.len() > 16 * 1024 && course_body.len() < COURSE_PLAN_BODY_LIMIT);
    let (status, result) = post(&client, &base, "/api/v1/courses/plan", course_body.clone()).await;
    assert_eq!(status, StatusCode::OK, "{result}");
    assert!(
        !result["candidates"]
            .as_array()
            .expect("candidates")
            .is_empty()
    );
    let (status, result) = post(
        &client,
        &base,
        "/api/v1/courses/plan",
        padded(course_body, COURSE_PLAN_BODY_LIMIT),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(result["error"], "course_request_invalid");
    let mut oversized_course = course_request;
    oversized_course["courses"][0]["source_excerpt"] =
        Value::String(format!("CONTROLLED0 Controlled course{}", "x".repeat(8192)));
    let (status, result) = post(
        &client,
        &base,
        "/api/v1/courses/plan",
        serde_json::to_string(&oversized_course).expect("JSON"),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(result["error"], "course_evidence_invalid");

    // Import conversion admits no package and must fit the existing 64 KiB
    // ParsedSkill resource bound, including six-byte JSON escapes in its body.
    let before_plugins: Value = client
        .get(format!("{base}/api/v1/plugins"))
        .header(CLIENT_PROTOCOL_MAJOR_HEADER, "1")
        .send()
        .await
        .expect("plugin catalog")
        .error_for_status()
        .expect("catalog status")
        .json()
        .await
        .expect("catalog JSON");
    let mut skill =
        "---\nname: body-limit-guide\ndescription: Controlled import preview.\n---\n".to_owned();
    skill.extend(std::iter::repeat_n('\u{0001}', 64 * 1024 - skill.len()));
    let preview = json!({"schema":"plugin-import-preview/v1", "package_id":"community.body-limit-guide",
        "version":"0.1.0", "display_name":"Controlled large Skill", "source":"Synthetic HTTP body-limit fixture; no external input",
        "skill":skill, "mcp":null});
    let preview_body = serde_json::to_string(&preview).expect("preview JSON");
    assert!(
        preview_body.len() > 16 * 1024
            && preview_body.len() < plugin_routes::IMPORT_PREVIEW_BODY_LIMIT
    );
    let (status, result) = post(
        &client,
        &base,
        "/api/v1/plugins/import-preview",
        preview_body.clone(),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{result}");
    assert_eq!(result["admitted"], false);
    assert_eq!(
        result["files"]["skills/body-limit-guide/SKILL.md"],
        preview["skill"]
    );
    assert!(result["files"]["package.json"].is_string());
    assert!(result["files"]["runtime.json"].is_string());
    assert!(result["files"]["configuration.json"].is_string());
    let after_plugins: Value = client
        .get(format!("{base}/api/v1/plugins"))
        .header(CLIENT_PROTOCOL_MAJOR_HEADER, "1")
        .send()
        .await
        .expect("plugin catalog")
        .error_for_status()
        .expect("catalog status")
        .json()
        .await
        .expect("catalog JSON");
    assert_eq!(
        after_plugins["packages"], before_plugins["packages"],
        "preview changed catalog or installations"
    );
    let (status, result) = post(
        &client,
        &base,
        "/api/v1/plugins/import-preview",
        padded(preview_body, plugin_routes::IMPORT_PREVIEW_BODY_LIMIT),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(result["error"], "invalid_plugin_request");
    let mut oversized_skill = preview;
    oversized_skill["skill"] = Value::String(format!(
        "{}x",
        oversized_skill["skill"].as_str().expect("Skill")
    ));
    let (status, result) = post(
        &client,
        &base,
        "/api/v1/plugins/import-preview",
        serde_json::to_string(&oversized_skill).expect("JSON"),
    )
    .await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(result["error"], "plugin_capacity_exceeded");
    // An unrelated route still inherits the existing 16 KiB transport cap.
    let search =
        serde_json::to_string(&json!({"schema":"source-search/v1","query":"","source_id":null}))
            .expect("JSON");
    let (status, result) = post(
        &client,
        &base,
        "/api/v1/sources/search",
        padded(search, 16 * 1024),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(result["error"], "source_search_invalid");
    server.abort();
}
