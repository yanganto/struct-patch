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
}

impl Base {
    pub fn has_bool(&self) -> bool {
        self.field_bool
    }
}

#[derive(Deserialize, Default, Substrate)]
pub struct PrivateBase {
    private_bool: bool,
    private_string: String,
    private_option: Option<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expose_works() {
        assert_eq!(
            Base::expose_content(),
            r#"{"named":[{"attrs":[{"style":"outer","meta":{"list":{"path":{"segments":[{"ident":"serde"}]},"delimiter":"paren","tokens":[{"ident":"default"}]}}}],"vis":"pub","ident":"field_bool","colon_token":true,"ty":{"path":{"segments":[{"ident":"bool"}]}}},{"vis":"pub","ident":"field_string","colon_token":true,"ty":{"path":{"segments":[{"ident":"String"}]}}},{"vis":"pub","ident":"field_option","colon_token":true,"ty":{"path":{"segments":[{"ident":"Option","arguments":{"angle_bracketed":{"args":[{"type":{"path":{"segments":[{"ident":"usize"}]}}}]}}}]}}}]}"#
        );

        let _fields: syn::Fields = syn_serde::json::from_str(&Base::expose_content()).unwrap();
    }

    #[test]
    fn expose_private_works() {
        assert_eq!(
            PrivateBase::expose_content(),
              r#"{"named":[{"ident":"private_bool","colon_token":true,"ty":{"path":{"segments":[{"ident":"bool"}]}}},{"ident":"private_string","colon_token":true,"ty":{"path":{"segments":[{"ident":"String"}]}}},{"ident":"private_option","colon_token":true,"ty":{"path":{"segments":[{"ident":"Option","arguments":{"angle_bracketed":{"args":[{"type":{"path":{"segments":[{"ident":"usize"}]}}}]}}}]}}}]}"#);

        let _fields: syn::Fields = syn_serde::json::from_str(&PrivateBase::expose_content()).unwrap();
    }
}
