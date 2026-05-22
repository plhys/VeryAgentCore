//! Real-network e2e: prove `node npm-cli.js exec --prefix=<tmp>` actually
//! installs and runs a tiny npm package. Gated by `#[ignore]` so it never
//! runs in normal CI; surface via `just test-npm-e2e` recipe.

use std::process::Stdio;

#[tokio::test]
#[ignore = "real npm install — run with: cargo test -p aionui-runtime --test npm_exec_integration -- --ignored"]
async fn npm_exec_in_prefix_actually_works() {
    let node = aionui_runtime::resolve_node().expect("bundled node must be available");
    let cli = aionui_runtime::resolve_npm_cli_js().expect("npm-cli.js must be available");

    let prefix = tempfile::TempDir::new().unwrap();
    let cache = tempfile::TempDir::new().unwrap();

    let mut cmd = tokio::process::Command::new(node);
    cmd.arg(cli)
        .arg("exec")
        .arg(format!("--prefix={}", prefix.path().display()))
        .arg(format!("--cache={}", cache.path().display()))
        .arg("--yes")
        .arg("--")
        .arg("cowsay@1.6.0")
        .arg("hello")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let output = cmd.output().await.expect("spawn failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "npm exec failed: status={:?} stderr={}",
        output.status,
        stderr
    );
    assert!(stdout.contains("hello"), "expected cowsay 'hello' in stdout: {stdout}");
}
