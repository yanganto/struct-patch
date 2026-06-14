#![allow(unused)]
use serde::Deserialize;
use struct_patch::Substrate;

// TODO verify no rename on a struct using Substrate
#[derive(Deserialize, Default, Substrate)]
pub struct Base {
    #[serde(default)]
    pub field_bool: bool,
    pub field_string: String,
    pub field_option: Option<usize>,
    private_number: u8,
}

impl Base {
    pub fn has_bool(&self) -> bool {
        self.field_bool
    }

    pub fn get_private_number(&self) -> u8 {
        self.private_number
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expose_works() {
        assert_eq!(
            Base::expose_content(),
            r#"{"named":[{"attrs":[{"style":"outer","meta":{"list":{"path":{"segments":[{"ident":"serde"}]},"delimiter":"paren","tokens":[{"ident":"default"}]}}}],"vis":"pub","ident":"field_bool","colon_token":true,"ty":{"path":{"segments":[{"ident":"bool"}]}}},{"vis":"pub","ident":"field_string","colon_token":true,"ty":{"path":{"segments":[{"ident":"String"}]}}},{"vis":"pub","ident":"field_option","colon_token":true,"ty":{"path":{"segments":[{"ident":"Option","arguments":{"angle_bracketed":{"args":[{"type":{"path":{"segments":[{"ident":"usize"}]}}}]}}}]}}},{"ident":"private_number","colon_token":true,"ty":{"path":{"segments":[{"ident":"u8"}]}}}]}"#
        );

        let _fields: syn::Fields = syn_serde::json::from_str(&Base::expose_content()).unwrap();
    }

    #[test]
    fn substrate_new_works() {
        let b = Base::__substrate_new(
            true,
            "test".to_string(),
            Some(100),
            7u8,
        );
        assert_eq!(b.field_bool, true);
        assert_eq!(b.field_string, "test");
        assert_eq!(b.field_option, Some(100));
        assert_eq!(b.get_private_number(), 7);
    }

    #[test]
    fn substrate_unpack_works() {
        let b = Base::__substrate_new(
            true,
            "test".to_string(),
            Some(100),
            7u8,
        );
        let (field_bool, field_string, field_option, private_number) = b.__substrate_unpack();

        assert_eq!(field_bool, true);
        assert_eq!(field_string, "test");
        assert_eq!(field_option, Some(100));
        assert_eq!(private_number, 7);
    }
}
