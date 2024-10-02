use serde::{Deserialize};
use std::time::Duration;
use struct_patch::Patch;

#[derive(Deserialize, Clone, Debug, Patch)]
#[patch(name = "FileConfig", attribute(derive(Deserialize, Debug)))]
struct Config {
    #[serde(with = "humantime_serde")]
    #[patch(attribute(serde(with = "humantime_serde")))]
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

    // NOTE
    // Following code does not work, because `humantime_serde` does not allow `Option<>` field
    // anymore.
    // https://github.com/jean-airoldie/humantime-serde/issues/13
    //
    // let empty_patch: FileConfig = toml::from_str("").unwrap();

    // let mut patched = config.clone();
    // patched.apply(empty_patch);
    // assert_eq!(patched.time, Duration::from_millis(500));
}
