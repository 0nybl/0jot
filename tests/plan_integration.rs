use std::fs;
use std::process::Command;

#[test]
fn plan_creates_and_closes() {
    let repo = tempfile::tempdir().unwrap();
    let work = tempfile::tempdir().unwrap();

    fs::create_dir_all(repo.path().join("src")).unwrap();
    fs::write(repo.path().join("src/x.rs"), "// @todo: keep me\n").unwrap();

    let kept_fp = jot::fingerprint::fingerprint("keep me");
    let issues = work.path().join("issues.json");
    fs::write(
        &issues,
        format!(
            r#"[{{"number": 1, "body": "<!-- 0jot: {kept} -->"}},
                {{"number": 2, "body": "<!-- 0jot: deadbeef0000 -->"}}]"#,
            kept = kept_fp
        ),
    )
    .unwrap();

    let out = work.path().join("actions.json");
    let status = Command::new(env!("CARGO_BIN_EXE_0jot"))
        .args([
            "plan",
            "--repo",
            repo.path().to_str().unwrap(),
            "--issues",
            issues.to_str().unwrap(),
            "--out",
            out.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(status.success());

    let body = fs::read_to_string(&out).unwrap();
    let v: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(v["create"].as_array().unwrap().len(), 0);
    assert_eq!(v["close"].as_array().unwrap().len(), 1);
    assert_eq!(v["close"][0]["number"], 2);
}
