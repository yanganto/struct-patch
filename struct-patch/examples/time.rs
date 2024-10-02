use serde::{Deserialize};
use std::time::Duration;
use struct_patch::Patch;

#[derive(Deserialize, Clone, Debug, Patch)]
#[patch(name = "FileConfig", attribute(derive(Deserialize, Debug)))]
struct Config {
    #[serde(with = "humantime_serde")]
    #[patch(attribute(serde(with = "humantime_serde", default)))]
    // NOTE:
    // We need extra default parameter for Option<T>.
    // https://github.com/jean-airoldie/humantime-serde/issues/13#issuecomment-2388437558
    time: Duration,
}

fn main() {
    let config = Config {
        time: Duration::from_millis(500),
    };

    let patch: FileConfig = toml::from_str("time = \"200ms\"").unwrap();

    let mut patched = config.clone();
    patched.apply(patch);
    assert_eq!(patched.time, Duration::from_millis(200));

    let empty_patch: FileConfig = toml::from_str("").unwrap();

    let mut patched = config.clone();
    patched.apply(empty_patch);
    assert_eq!(patched.time, Duration::from_millis(500));
}
