//! End-to-end test of add / edit / delete persisting to the environments file,
//! driven through the same `State` methods the TUI calls.

use lazyprop::environment::Environments;
use lazyprop::state::State;
use lazyprop::text_field::TextField;

fn temp_envs_file() -> String {
    let path = std::env::temp_dir().join(format!(
        "lazyprop_crud_{}_{}.yaml",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::copy("tests/fixtures/envs.yaml", &path).expect("copy fixture envs.yaml");
    path.to_string_lossy().to_string()
}

#[test]
fn add_edit_delete_persist_to_file() {
    let file = temp_envs_file();
    let mut state = State::new(Some(file.clone()), None).expect("state loads");
    let start = state.envs.len();

    // --- Add ---
    state.open_add_form();
    {
        let form = state.form.as_mut().unwrap();
        form.name = TextField::from_text("StagingEnv");
        form.key = TextField::from_text("stagingkey123456");
    }
    state.submit_form().expect("add should succeed and save");
    assert!(state.form.is_none(), "form should close on success");

    let reloaded = Environments::new(&file).unwrap();
    assert_eq!(reloaded.len(), start + 1);
    assert!(reloaded.environments.iter().any(|e| e.name == "StagingEnv"));

    // --- Duplicate add is rejected and keeps the form open with an error ---
    state.open_add_form();
    state.form.as_mut().unwrap().name = TextField::from_text("StagingEnv");
    state.form.as_mut().unwrap().key = TextField::from_text("whatever00000000");
    assert!(
        state.submit_form().is_err(),
        "duplicate name must be rejected"
    );
    state.cancel_form();

    // --- Edit the newly added env (it is now selected) ---
    state.open_edit_form();
    state.form.as_mut().unwrap().key = TextField::from_text("rotatedkey000000");
    state.submit_form().expect("edit should succeed and save");
    let reloaded = Environments::new(&file).unwrap();
    let edited = reloaded
        .environments
        .iter()
        .find(|e| e.name == "StagingEnv")
        .unwrap();
    assert_eq!(edited.key, "rotatedkey000000");

    // --- Delete it ---
    state.request_delete();
    assert!(state.pending_delete.is_some());
    state
        .confirm_delete()
        .expect("delete should succeed and save");
    let reloaded = Environments::new(&file).unwrap();
    assert_eq!(reloaded.len(), start);
    assert!(!reloaded.environments.iter().any(|e| e.name == "StagingEnv"));

    let _ = std::fs::remove_file(&file);
}
