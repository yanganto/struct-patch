#[cfg(feature = "option")]
use struct_patch::Patch;

#[cfg(all(
    feature = "option",
    not(feature = "keep_none"),
    not(feature = "none_as_default")
))]
fn pure_none_feature() {
    #[derive(Debug, PartialEq, Patch)]
    #[patch(attribute(derive(Debug)))]
    struct User {
        name: String,
        #[patch(name = "Option<AddressPatch>")]
        address: Option<Address>,
    }

    #[derive(Debug, PartialEq, Patch)]
    #[patch(attribute(derive(Debug)))]
    struct Address {
        street: Option<String>,
        country: String,
    }

    // NOTE: we need impl the From trait.
    // When patch on None, the patch will convert into the instance base on From implementation
    impl From<AddressPatch> for Address {
        fn from(patch: AddressPatch) -> Self {
            let mut address = Address {
                street: None,
                country: "France".to_string(),
            };
            address.apply(patch);
            address
        }
    }

    let user = User {
        name: String::from("Thomas"),
        address: None,
    };
    let mut patch: UserPatch = User::new_empty_patch();

    patch.address = Some(Some(AddressPatch {
        street: Some(Some("Av. Gustave Eiffel, 75007 Paris".to_string())),
        country: None,
    }));

    let mut next_patch: UserPatch = User::new_empty_patch();
    next_patch.address = Some(Some(AddressPatch {
        street: Some(Some("New Address".to_string())),
        country: None,
    }));

    let patched_user = user << patch << next_patch;

    assert_eq!(
        patched_user,
        User {
            name: String::from("Thomas"),
            address: Some(Address {
                street: Some(String::from("New Address")),
                country: String::from("France"),
            }),
        }
    );
}

#[cfg(feature = "none_as_default")]
fn none_as_default_feature() {
    #[derive(Debug, PartialEq, Patch)]
    #[patch(attribute(derive(Debug)))]
    struct User {
        name: String,
        #[patch(name = "Option<AddressPatch>")]
        address: Option<Address>,
    }

    #[derive(Debug, PartialEq, Patch)]
    #[patch(attribute(derive(Debug)))]
    struct Address {
        street: Option<String>,
        country: String,
    }

    // NOTE: we need impl the Default trait
    // When patch on None, the patch will patch on a Default instance
    impl Default for Address {
        fn default() -> Self {
            Self {
                country: "France".to_string(),
                street: None,
            }
        }
    }

    let user = User {
        name: String::from("Thomas"),
        address: None,
    };
    let mut patch: UserPatch = User::new_empty_patch();

    patch.address = Some(Some(AddressPatch {
        street: Some(Some("Av. Gustave Eiffel, 75007 Paris".to_string())),
        country: None,
    }));

    let mut next_patch: UserPatch = User::new_empty_patch();
    next_patch.address = Some(Some(AddressPatch {
        street: Some(Some("New Address".to_string())),
        country: None,
    }));

    let patched_user = user << patch << next_patch;

    assert_eq!(
        patched_user,
        User {
            name: String::from("Thomas"),
            address: Some(Address {
                street: Some(String::from("New Address")),
                country: String::from("France"),
            }),
        }
    );
}

#[cfg(feature = "keep_none")]
fn keep_none_feature() {
    #[derive(Debug, PartialEq, Patch)]
    #[patch(attribute(derive(Debug)))]
    struct User {
        name: String,
        #[patch(name = "Option<AddressPatch>")]
        address: Option<Address>,
    }

    #[derive(Debug, PartialEq, Patch)]
    #[patch(attribute(derive(Debug)))]
    struct Address {
        street: Option<String>,
        country: String,
    }

    let mut user = User {
        name: String::from("Thomas"),
        address: None,
    };
    let mut patch: UserPatch = User::new_empty_patch();

    patch.address = Some(Some(AddressPatch {
        street: Some(Some("Av. Gustave Eiffel, 75007 Paris".to_string())),
        country: None,
    }));

    user.apply(patch);

    assert_eq!(
        user,
        User {
            name: String::from("Thomas"),
            address: None
        }
    );
}

#[cfg(feature = "option")]
fn main() {
    // NOTE:
    // The `pure_none_feature` and `none_as_default_feature` are the same logic,
    // but the former uses `From` trait and the later uses `Default` trait.
    // You can base on your need to use `option` feature or `none_as_default` feature
    #[cfg(all(not(feature = "keep_none"), not(feature = "none_as_default")))]
    pure_none_feature();
    #[cfg(feature = "none_as_default")]
    none_as_default_feature();

    // NOTE:
    // In the feature, the patch do not allow to apply on None
    #[cfg(feature = "keep_none")]
    keep_none_feature();
}

#[cfg(not(feature = "option"))]
fn main() {
    println!("Please enable the 'option' feature to run this example");
}
