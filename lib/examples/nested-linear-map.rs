use struct_patch::Patch;
use linear_map::{linear_map, LinearMap};

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum Key {
    TEST1 = 1,
    TEST2 = 2

}

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum KeyB {
    B1 = 1,
    B2 = 2
}

#[derive(Debug, Clone, PartialEq, Eq, Patch)]
#[patch(attribute(derive(Debug, Default)))]
pub struct ExampleB {
    pub foo: Option<u8>,
}

#[derive(Debug, Clone, Default, PartialEq, Patch)]
#[patch(attribute(derive(Debug, Default )))]
pub struct InnerConfig {
    pub example_a: LinearMap<Key, u8>,
    pub example_b: LinearMap<KeyB, ExampleB>,
}

#[cfg(feature = "nesting")]
#[derive(Debug, Clone, Default, PartialEq, Patch)]
#[patch(attribute(derive(Debug, Default)))]
pub struct Config {
    #[patch(nesting)]
    pub inner: InnerConfig,
}

#[cfg(not(feature = "nesting"))]
fn main() {}

#[cfg(feature = "nesting")]
fn main() {
    let config_a = Config::default();
    let config_b = Config {
        inner: InnerConfig {
            example_a: linear_map!{
                Key::TEST1 => 1,
                Key::TEST2 => 2,
            },
            ..Default::default()
        },
    };

    let patch: ConfigPatch = config_b.clone().into_patch_by_diff(config_a);
    assert_eq!(
        format!("{patch:?}"),
        "ConfigPatch { inner: InnerConfigPatch { example_a: Some({TEST1: 1, TEST2: 2}), example_b: None } }"
    );

    let mut config = Config::default();
    config.apply(patch);

    assert_eq!(config, config_b);

    // --- test nesting with ExampleB --- 

    let patch_b: ConfigPatch = ConfigPatch {
        inner: InnerConfigPatch {
            example_b: Some(
               linear_map!{
                    KeyB::B1 => ExampleB {
                        foo: Some(7),
                    },
                }
            ),
            ..Default::default()
        },
    };
    config.apply(patch_b);
    assert_eq!(
        format!("{config:?}"),
        "Config { inner: InnerConfig { example_a: {TEST1: 1, TEST2: 2}, example_b: {B1: ExampleB { foo: Some(7) }} } }"
    );

    let _patch_under_example_b = Some(
       linear_map!{
            KeyB::B2 => ExampleB {
                foo: Some(100),
            },
        }
    );

    // TODO
    // nested map feature
    config.apply(_patch_under_example_b);
}
