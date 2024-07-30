use config::{Config, File};

pub fn cfg() -> Config {
    Config::builder()
        .add_source(File::with_name(&format!(
            "{}/../Settings.toml",
            std::env::var("CARGO_MANIFEST_DIR").unwrap()
        )))
        .build()
        .unwrap()
}
