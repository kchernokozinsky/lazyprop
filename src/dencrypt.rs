use std::path::Path;

use crate::environment::Environment;

const MAIN_CLASS: &str = "com.mulesoft.tools.SecurePropertiesTool";
const ENCRYPT_ACTION: &str = "encrypt";
const DECRYPT_ACTION: &str = "decrypt";

/// Encrypt `input` using the algorithm, mode and key of the given environment.
pub fn encrypt(input: &str, env: &Environment, jar_path: &Path) -> Result<String, String> {
    invoke_jar(jar_path, ENCRYPT_ACTION, input, env)
}

/// Decrypt `input` using the algorithm, mode and key of the given environment.
pub fn decrypt(input: &str, env: &Environment, jar_path: &Path) -> Result<String, String> {
    invoke_jar(jar_path, DECRYPT_ACTION, input, env)
}

/// Build the argument list passed to `java`, matching the CLI of the
/// MuleSoft Secure Properties Tool:
///
/// ```text
/// java -cp <jar> com.mulesoft.tools.SecurePropertiesTool \
///     string <encrypt|decrypt> <algorithm> <mode> <key> <value> [--use-random-iv]
/// ```
fn build_args(jar_path: &Path, action: &str, input: &str, env: &Environment) -> Vec<String> {
    let mut args = vec![
        "-cp".to_string(),
        jar_path.to_string_lossy().to_string(),
        MAIN_CLASS.to_string(),
        "string".to_string(),
        action.to_string(),
        format!("{:?}", env.algorithm),
        format!("{:?}", env.state),
        env.key.trim().to_string(),
        input.trim().to_string(),
    ];
    if env.use_random_ivs {
        args.push("--use-random-iv".to_string());
    }
    args
}

fn invoke_jar(
    jar_path: &Path,
    action: &str,
    input: &str,
    env: &Environment,
) -> Result<String, String> {
    let args = build_args(jar_path, action, input, env);
    let output = std::process::Command::new("java")
        .args(&args)
        .output()
        .map_err(|e| format!("Failed to run `java` (is a JRE installed and on PATH?): {e}"))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let message = if !stderr.trim().is_empty() {
            stderr.trim().to_string()
        } else {
            stdout.trim().to_string()
        };
        Err(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environment::{Algorithm, State};
    use std::path::PathBuf;

    fn env(use_random_ivs: bool) -> Environment {
        Environment::new(
            "Test",
            Algorithm::AES,
            State::CBC,
            use_random_ivs,
            "secret1234567890",
        )
    }

    #[test]
    fn build_args_encrypt_without_iv() {
        let jar = PathBuf::from("tool.jar");
        let args = build_args(&jar, ENCRYPT_ACTION, " hello ", &env(false));
        assert_eq!(
            args,
            vec![
                "-cp",
                "tool.jar",
                MAIN_CLASS,
                "string",
                "encrypt",
                "AES",
                "CBC",
                "secret1234567890",
                "hello",
            ]
        );
    }

    #[test]
    fn build_args_appends_random_iv_flag() {
        let jar = PathBuf::from("tool.jar");
        let args = build_args(&jar, DECRYPT_ACTION, "cipher", &env(true));
        assert_eq!(args.last().map(String::as_str), Some("--use-random-iv"));
        assert!(args.contains(&"decrypt".to_string()));
    }
}
