/// **(internal)** Implementation of the safe identifier generating.
mod _impl_id_generating;
/// **(internal)** Methods for safely constructing or mutating instances of `RegulationsState`.
mod _impl_mutator_methods;
/// **(internal)** Methods for observing instances of `RegulationsState` (various getters, etc.).
mod _impl_observer_methods;
/// **(internal)** Minor traits like [PartialEq] or [Display].
mod _impl_other_traits;
/// **(internal)** Methods for converting between `RegulationsState` and `RegulatoryGraph`.
mod _impl_reg_graph_conversion;
/// **(internal)** Implementation of serialization traits [Serialize] and [Deserialize].
mod _impl_serde;
/// **(internal)** Implementation of the [SessionState] trait.
mod _impl_session_state;
