use std::fmt::{Display, Error, Formatter};
use std::str::FromStr;

/// **(internal)**  Definition and utility methods for `DatasetData` and `DatasetMetaData`.
mod _dataset_data;
/// **(internal)**  Definition and utility methods for `ChangeArgMonotoneData`
/// and `ChangeArgEssentialData`.
mod _fn_arg_change_data;
/// **(internal)**  Definition and utility methods for `ChangeIdData`.
mod _id_change_data;
/// **(internal)**  Definition and utility methods for `LayoutData`.
mod _layout_data;
/// **(internal)**  Definition and utility methods for `LayoutNodeData`.
mod _layout_node_data;
/// **(internal)**  Definition and utility methods for `ObservationData`.
mod _observation_data;
/// **(internal)** Definition and utility methods for `RegulationData`.
mod _regulation_data;
/// **(internal)**  Definition and utility methods for `UninterpretedFnData`.
mod _uninterpreted_fn_data;
/// **(internal)**  Definition and utility methods for `VariableData`.
mod _variable_data;

pub use _dataset_data::{DatasetData, DatasetMetaData};
pub use _fn_arg_change_data::{ChangeArgEssentialData, ChangeArgMonotoneData};
pub use _id_change_data::ChangeIdData;
pub use _layout_data::LayoutData;
pub use _layout_node_data::LayoutNodeData;
pub use _observation_data::ObservationData;
pub use _regulation_data::RegulationData;
pub use _uninterpreted_fn_data::UninterpretedFnData;
pub use _variable_data::VariableData;

/// Define a macro that implements Display and FromStr for all data structs.
/// All of them implement these two traits the same way, delegating it to [serde].
macro_rules! impl_display_fromstr_with_serde {
    ($($t:ty),*) => {
        $(
            impl Display for $t {
                /// Use json serialization for conversion to string.
                fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
                    write!(f, "{}", serde_json::to_string(self).unwrap())
                }
            }

            impl FromStr for $t {
                type Err = String;

                /// Use json de-serialization to construct data struct from string.
                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    serde_json::from_str(s).map_err(|e| e.to_string())
                }
            }
        )*
    };
}

// Use the macro to implement Display and FromStr for all data types
impl_display_fromstr_with_serde!(
    DatasetData,
    DatasetMetaData,
    ChangeArgEssentialData,
    ChangeArgMonotoneData,
    ChangeIdData,
    LayoutData,
    LayoutNodeData,
    ObservationData,
    RegulationData,
    UninterpretedFnData,
    VariableData
);
