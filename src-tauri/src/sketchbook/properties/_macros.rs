/// Generate two enums - one where each variant contains inner data, and the other
/// (with completely same variants) without the inner data. Also generates `From` trait for
/// converting the complex one into the simpler one.
///
/// This can be used for static and dynamic properties. One of the datatypes will carry the
/// whole property data, while the other will be used in cases when inner data are not needed.
#[macro_export]
macro_rules! generate_property_enums {
    (
        $original_enum:ident, $simple_enum:ident, {
            $($variant:ident($inner:ty)),*
        }
    ) => {
        #[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
        pub enum $original_enum {
            $($variant($inner)),*
        }

        #[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
        pub enum $simple_enum {
            $($variant),*
        }

        impl From<$original_enum> for $simple_enum {
            fn from(value: $original_enum) -> Self {
                match value {
                    $($original_enum::$variant(_) => $simple_enum::$variant),*
                }
            }
        }
    }
}
