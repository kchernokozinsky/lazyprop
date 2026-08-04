//! Renders the Home component into an in-memory backend to verify the layout
//! draws without panicking and shows the expected panes. Uses the bundled
//! `envs.yaml` (resolved relative to the crate root during `cargo test`).

use lazyprop::components::about::AboutScreen;
use lazyprop::components::home::Home;
use lazyprop::components::playground::PlaygroundScreen;
use lazyprop::components::Component;
use lazyprop::state::State;
use ratatui::{backend::TestBackend, Terminal};

/// A fixed environments file so tests do not depend on the user's real
/// `envs.yaml` (which the app itself can modify).
const FIXTURE: &str = "tests/fixtures/envs.yaml";

fn buffer_text(terminal: &Terminal<TestBackend>) -> String {
    let buf = terminal.backend().buffer();
    buf.content()
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>()
}

#[test]
fn home_renders_expected_panes() {
    let state = State::new(Some(FIXTURE.to_string()), None).expect("state should load fixture");
    let mut home = Home::new().expect("home should build");

    let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
    terminal
        .draw(|frame| {
            home.draw(frame, frame.area(), &state)
                .expect("draw should not error");
        })
        .unwrap();

    let text = buffer_text(&terminal);
    for expected in ["Environments", "Environment", "Value", "Result", "Status"] {
        assert!(text.contains(expected), "missing pane label: {expected}");
    }
    // The first environment from envs.yaml should be visible.
    assert!(text.contains("DefaultEnv"), "environment name not rendered");
}

#[test]
fn about_screen_renders_help_and_info() {
    let state = State::new(Some(FIXTURE.to_string()), None).expect("state should load fixture");
    let mut about = AboutScreen::new();
    // The keybindings section reads from the config, as the app wires it up.
    about
        .register_config_handler(lazyprop::config::Config::new().expect("config loads"))
        .unwrap();

    // Tall enough to fit the logo art plus all sections without scrolling.
    let mut terminal = Terminal::new(TestBackend::new(100, 60)).unwrap();
    terminal
        .draw(|frame| {
            about
                .draw(frame, frame.area(), &state)
                .expect("draw should not error");
        })
        .unwrap();

    let text = buffer_text(&terminal);
    for expected in ["About lazyprop", "Keybindings", "Version", "Author", "Repo"] {
        assert!(text.contains(expected), "about screen missing: {expected}");
    }
    // Keybindings show friendly descriptions, not raw action names.
    assert!(
        text.contains("Add environment"),
        "friendly descriptions missing"
    );
    assert!(
        !text.contains("AddEnv"),
        "raw action name should not appear"
    );
}

#[test]
fn playground_screen_renders_form() {
    let state = State::new(Some(FIXTURE.to_string()), None).expect("state should load fixture");
    let mut playground = PlaygroundScreen::new();

    let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
    terminal
        .draw(|frame| {
            playground
                .draw(frame, frame.area(), &state)
                .expect("draw should not error");
        })
        .unwrap();

    let text = buffer_text(&terminal);
    for expected in ["Operation", "Algorithm", "State", "Key", "Value", "Result"] {
        assert!(text.contains(expected), "playground missing: {expected}");
    }
}

#[test]
fn search_filters_the_environment_list() {
    let state = State::new(Some(FIXTURE.to_string()), None).expect("state should load fixture");
    // The fixture has DefaultEnv and BlowfishEnv.
    let all = state.filtered().len();
    assert_eq!(all, 2, "fixture should have two environments");

    let mut filtered = State::new(Some(FIXTURE.to_string()), None).unwrap();
    filtered.push_search('b'); // matches "BlowfishEnv" (case-insensitive)
    let matches: Vec<_> = filtered
        .filtered()
        .iter()
        .map(|&i| filtered.envs.environments[i].name.clone())
        .collect();
    assert!(matches.iter().all(|n| n.to_lowercase().contains('b')));
    assert!(matches.iter().any(|n| n == "BlowfishEnv"));
    assert!(!matches.iter().any(|n| n == "DefaultEnv"));
}

#[test]
fn env_form_overlay_renders() {
    let mut state = State::new(Some(FIXTURE.to_string()), None).expect("state should load fixture");
    let mut home = Home::new().expect("home should build");
    state.open_add_form();

    let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
    terminal
        .draw(|frame| {
            home.draw(frame, frame.area(), &state).unwrap();
        })
        .unwrap();
    let text = buffer_text(&terminal);
    assert!(text.contains("Add environment"), "form title not rendered");
    assert!(text.contains("Algorithm"), "form fields not rendered");
}

#[test]
fn yaml_screen_renders_tree() {
    use lazyprop::components::yaml::YamlScreen;

    let mut state = State::new(Some(FIXTURE.to_string()), None).expect("state should load fixture");
    let tmp = std::env::temp_dir().join(format!("lazyprop_render_{}.yaml", std::process::id()));
    std::fs::write(
        &tmp,
        "database:\n  host: localhost\n  password: \"![CIPHER]\"\nservers:\n  - host: one\n    port: 8081\n",
    )
    .unwrap();
    state
        .yaml
        .open_path(tmp.to_str().unwrap())
        .expect("open yaml");

    let mut screen = YamlScreen::new();
    let mut terminal = Terminal::new(TestBackend::new(100, 24)).unwrap();
    terminal
        .draw(|frame| {
            screen.draw(frame, frame.area(), &state).unwrap();
        })
        .unwrap();
    let text = buffer_text(&terminal);
    for expected in [
        "YAML",
        "Environments",
        "YAML tree",
        "database",
        "host",
        "servers",
    ] {
        assert!(text.contains(expected), "yaml screen missing: {expected}");
    }
    // Encrypted value is masked, not shown in the clear.
    assert!(text.contains("![••••••]"), "encrypted value not masked");
    assert!(!text.contains("![CIPHER]"), "ciphertext should be masked");
    let _ = std::fs::remove_file(&tmp);
}
