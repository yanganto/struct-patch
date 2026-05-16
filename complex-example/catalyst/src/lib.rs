use serde::{Deserialize, Serialize};
use struct_patch::Catalyst;
use substrate::Base;

#[derive(Default, Catalyst)]
#[catalyst(bind = Base)]
// The Substrate has `#[serde(...)]` on fields , and catalyst keep_field_attribute
// so the complex should have the corresponding Serialize derive
#[catalyst(keep_field_attribute)]
#[complex(attribute(derive(Debug, Deserialize, Serialize)))]
#[allow(dead_code)]
struct Amyloid {
    pub extra_bool: bool,
    #[complex(attribute(serde(default = "default_str")))]
    pub extra_string: String,
    pub extra_option: Option<usize>,
}

fn default_str() -> String {
    "default".to_string()
}

#[derive(Catalyst)]
#[catalyst(bind = Base)]
#[complex(name = "SmallCpx")]
#[allow(dead_code)]
#[complex(attribute(derive(Default)))]
struct SmallAmyloid {
    pub extra_bool: bool,
}

#[allow(dead_code)]
impl SmallCpx {
    /// A reaction to change the substrate
    pub fn reaction(&mut self) {
        self.field_bool = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use struct_patch::Complex;

    #[test]
    fn complex_works() {
        let mut small_complex = SmallCpx::default();
        assert_eq!(small_complex.field_bool, false);
        assert_eq!(small_complex.field_string, String::new());
        assert_eq!(small_complex.field_option, None);
        assert_eq!(small_complex.extra_bool, false);

        small_complex.reaction();

        let (_cat, substrate) = small_complex.decouple();
        assert!(substrate.has_bool());

        let amyloid = Amyloid::default();
        let complex = amyloid.bind(substrate);
        assert_eq!(complex.field_bool, true);

        let toml_str = toml::to_string_pretty(&complex).unwrap();
        assert_eq!(
            toml_str,
            r#"field_bool = true
field_string = ""
extra_bool = false
extra_string = ""
"#
        );
        let toml_str = r#" field_bool = true
field_string = ""
extra_bool = true
    "#;
        let complex: AmyloidComplex = toml::from_str(toml_str).unwrap();
        assert_eq!(complex.extra_string, "default");
    }
}
