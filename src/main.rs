use lazyprop::{
    config::{app::AppConfig, env::EnvironmentsConfig},
    encryption::{decrypt, encrypt},
    state::AppState,
};

const CONFIG: &str = "conf.yaml";

fn main() -> anyhow::Result<()> {
    let config = AppConfig::new(CONFIG).unwrap_or_else(|e| {
        std::process::exit(1);
    });

    let mut envs =
        EnvironmentsConfig::new(config.envs_path.display().to_string()).unwrap_or_else(|e| {
            std::process::exit(1);
        });

    let mut state: AppState = AppState::new(&mut envs);

    match encrypt("sasad", state.curr_env()?, config.jar_path.clone()) {
        Ok(e) => {
            println!("encrypt: {}", &e)
        }
        Err(e) => println!("{}", e),
    }

    match decrypt(
        "MhKI6FUNIB5KyP9QXN5x2Q==",
        state.curr_env()?,
        config.jar_path.clone(),
    ) {
        Ok(e) => {
            println!("decrypt: {}", &e)
        }
        Err(e) => println!("{}", e),
    }

    Ok(())
}
