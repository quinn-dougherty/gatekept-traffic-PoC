use config::{Config, File};

pub fn cfg() -> Config {
    Config::builder()
        .add_source(File::with_name("./Settings"))
        .build()
        .unwrap()
}
