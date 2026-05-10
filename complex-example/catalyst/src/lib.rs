use struct_patch::Catalyst;

#[derive(Catalyst)]
#[catalyst(bind = substrate::Base)]
#[allow(dead_code)]
struct Amyloid {
    extra_bool: bool,
    extra_string: String,
    extra_option: Option<usize>,
}

#[derive(Catalyst)]
#[catalyst(bind = substrate::Base)]
#[complex(name = "SmallCpx")]
#[allow(dead_code)]
#[complex(attribute(derive(Default)))]
struct SmallAmyloid {
    extra_bool: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complex_works() {
        let _complex = AmyloidComplex {
            field_bool: false,
            field_string: String::new(),
            field_option: None,
            extra_bool: false,
            extra_string: String::new(),
            extra_option: None,
        };

        let small_complex = SmallCpx::default();
        assert_eq!(small_complex.field_bool, false);
        assert_eq!(small_complex.field_string, String::new());
        assert_eq!(small_complex.field_option, None);
        assert_eq!(small_complex.extra_bool, false);
    }
}
