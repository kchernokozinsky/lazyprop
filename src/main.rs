use lazyprop::{
    encryption::{decrypt, encrypt},
    env::{Algorithm, Environment, State},
    state::AppState,
};

fn main() {
    let jar_path = "res/secure-properties-tool.jar";

    let mut state: AppState = AppState::new();
    let env = Environment::new(
        "MyEnv",
        Algorithm::AES,
        State::CBC,
        true,
        "secret1234567890",
    );

    state.add_env(env);
    state.set_temp_env(0);

    match encrypt("sasad", &state.temp_env.clone().unwrap(), jar_path) {
        Ok(e) => {
            println!("encrypt: {}", &e)
        }
        Err(e) => println!("{}", e),
    }

    match decrypt(
        "MhKI6FUNIB5KyP9QXN5x2Q==",
        &state.temp_env.unwrap(),
        jar_path,
    ) {
        Ok(e) => {
            println!("decrypt: {}", &e)
        }
        Err(e) => println!("{}", e),
    }
}
