use struct_patch::Catalyst;
use substrate::Base;

#[derive(Catalyst)]
#[catalyst(bind = Base)]
#[allow(dead_code)]
struct Amyloid {
    pub extra_bool: bool,
    pub extra_string: String,
    pub extra_option: Option<usize>,
}

#[derive(Default, Catalyst)]
#[catalyst(bind = Base)]
#[complex(name = "SmallCpx")]
#[allow(dead_code)]
#[complex(attribute(derive(Default)))]
struct SmallAmyloid {
    pub extra_bool: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complex_works() {
        let small_complex = SmallCpx::default();
        assert_eq!(small_complex.field_bool, false);
        assert_eq!(small_complex.field_string, String::new());
        assert_eq!(small_complex.field_option, None);
        assert_eq!(small_complex.extra_bool, false);

        use struct_patch::Complex;
        let (_cat, mut substrate) = small_complex.decouple();

        substrate.field_bool = true;

        let _complex = AmyloidComplex {
            field_bool: false,
            field_string: String::new(),
            field_option: None,
            extra_bool: false,
            extra_string: String::new(),
            extra_option: None,
        };
    }
}
