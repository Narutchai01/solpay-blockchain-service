use dotenvy::dotenv;
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub app_port: i32,
    pub environment: String,
    pub mq_url: String,
}

impl Config {
    pub fn load() -> Self {
        if dotenv().is_err() {
            println!("Note: .env file not found. Using system environment variables instead.");
        }

        Config {
            app_port: Self::get_env("APP_PORT", Some("8080"))
                .parse::<i32>()
                .expect("APP_PORT must be a valid integer"),
            environment: Self::get_env("ENVIRONMENT", Some("development")),
            mq_url: Self::get_env("MQ_URL", Some("amqp://guest:guest@localhost:5672/")),
        }
    }

    fn get_env(key: &str, fallback: Option<&str>) -> String {
        match env::var(key) {
            Ok(val) => val,
            Err(_) => {
                if let Some(default_val) = fallback {
                    default_val.to_string()
                } else {
                    panic!("Environment variable {} is required but not set", key);
                }
            }
        }
    }
}
