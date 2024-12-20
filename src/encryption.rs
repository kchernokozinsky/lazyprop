const ENCRYPT_ACTION: &str = "encrypt";

const DECRPYPT_ACTION: &str = "decrypt";

pub fn encrypt(input: &str, env: &Environment, jar_path: PathBuf) -> Result<String, String> {
    invoke_jar(
        jar_path,
        ENCRYPT_ACTION,
        input,
        &format!("{:?}", env.algorithm),
        &format!("{:?}", env.state),
        env.use_random_ivs,
        &env.key,
    )
}

pub fn decrypt(input: &str, env: &Environment, jar_path: PathBuf) -> Result<String, String> {
    invoke_jar(
        jar_path,
        DECRPYPT_ACTION,
        input,
        &format!("{:?}", env.algorithm),
        &format!("{:?}", env.state),
        env.use_random_ivs,
        &env.key,
    )
}
use std::path::PathBuf;

use crate::env::Environment;

pub fn invoke_jar(
    jar_path: PathBuf,
    action: &str,
    input: &str,
    algorithm: &str,
    mode: &str,
    _random_iv: bool,
    key: &str,
) -> Result<String, String> {
    let output = std::process::Command::new("java")
        .arg("-cp")
        .arg(jar_path)
        .arg("com.mulesoft.tools.SecurePropertiesTool")
        .arg("string")
        .arg(action)
        .arg(algorithm)
        .arg(mode)
        .arg(key.trim())
        .arg(input.trim())
        .output()
        .map_err(|e| format!("Failed to invoke JAR file: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}
