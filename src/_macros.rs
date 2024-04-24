#![macro_use]

#[doc(hidden)]
macro_rules! handle_tsk_return_value {
    ($code: expr) => {{
        if $code < 0 {
            return Err($crate::error::TskitError::ErrorCode { code: $code });
        }
        Ok($code)
    }};
    ($code: expr, $return_value: expr) => {{
        if $code < 0 {
            return Err($crate::error::TskitError::ErrorCode { code: $code });
        }
        Ok($return_value)
    }};
}

macro_rules! panic_on_tskit_error {
    ($code: expr) => {
        if $code < 0 {
            let c_str =
                unsafe { std::ffi::CStr::from_ptr($crate::sys::bindings::tsk_strerror($code)) };
            let str_slice: &str = c_str.to_str().expect("failed to obtain &str from c_str");
            let message: String = str_slice.to_owned();
            panic!("{}", message);
        }
    };
}

macro_rules! decode_metadata_row {
    ($T: ty, $buffer: expr) => {
        <$T as $crate::metadata::MetadataRoundtrip>::decode($buffer)
    };
}

macro_rules! err_if_not_tracking_samples {
    ($flags: expr, $rv: expr) => {
        match $flags.contains($crate::TreeFlags::SAMPLE_LISTS) {
            false => Err(TskitError::NotTrackingSamples),
            true => Ok($rv),
        }
    };
}

// This macro assumes that table row access helper
// functions have a standard interface.
// Here, we convert the None type to an Error,
// as it implies $row is out of range.
macro_rules! table_row_access {
    ($row: expr, $table: expr, $row_fn: ident) => {
        $row_fn($table, $row)
    };
}

macro_rules! iterator_for_nodeiterator {
    ($ty: ty) => {
        impl Iterator for $ty {
            type Item = $crate::NodeId;
            fn next(&mut self) -> Option<Self::Item> {
                self.next_node();
                self.current_node()
            }
        }
    };
}

macro_rules! impl_id_traits {
    ($idtype: ty) => {
        impl $idtype {
            /// NULL value for this type.
            pub const NULL: $idtype = Self($crate::sys::TSK_NULL);

            /// Return `true` is `self == Self::NULL`
            pub fn is_null(&self) -> bool {
                *self == Self::NULL
            }

            /// Convenience function to convert to usize.
            ///
            /// Works via [`TryFrom`].
            ///
            /// # Returns
            ///
            /// * `None` if the underlying value is negative.
            /// * `Some(usize)` otherwise.
            pub fn to_usize(&self) -> Option<usize> {
                usize::try_from(*self).ok()
            }

            /// Convenience function to convert to usize.
            /// Implemented via `as`.
            /// Negative values with therefore wrap.
            pub fn as_usize(&self) -> usize {
                self.0 as usize
            }
        }

        impl std::fmt::Display for $idtype {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                match *self == Self::NULL {
                    false => write!(f, "{}", self.0),
                    true => write!(f, "NULL"),
                }
            }
        }

        impl From<$crate::sys::bindings::tsk_id_t> for $idtype {
            fn from(value: $crate::sys::bindings::tsk_id_t) -> Self {
                Self(value)
            }
        }

        impl TryFrom<$idtype> for usize {
            type Error = crate::TskitError;
            fn try_from(value: $idtype) -> Result<Self, Self::Error> {
                match value.0.try_into() {
                    Ok(value) => Ok(value),
                    Err(_) => Err(crate::TskitError::RangeError(format!(
                        "could not convert {:?} to usize",
                        value
                    ))),
                }
            }
        }

        impl From<$idtype> for $crate::sys::bindings::tsk_id_t {
            fn from(value: $idtype) -> Self {
                value.0
            }
        }

        impl TryFrom<$idtype> for $crate::SizeType {
            type Error = $crate::TskitError;

            fn try_from(value: $idtype) -> Result<Self, Self::Error> {
                $crate::SizeType::try_from(value.0)
            }
        }

        impl PartialEq<$crate::sys::bindings::tsk_id_t> for $idtype {
            fn eq(&self, other: &$crate::sys::bindings::tsk_id_t) -> bool {
                self.0 == *other
            }
        }

        impl PartialEq<$idtype> for $crate::sys::bindings::tsk_id_t {
            fn eq(&self, other: &$idtype) -> bool {
                *self == other.0
            }
        }

        impl PartialOrd<$crate::sys::bindings::tsk_id_t> for $idtype {
            fn partial_cmp(
                &self,
                other: &$crate::sys::bindings::tsk_id_t,
            ) -> Option<std::cmp::Ordering> {
                self.0.partial_cmp(other)
            }
        }

        impl PartialOrd<$idtype> for $crate::sys::bindings::tsk_id_t {
            fn partial_cmp(&self, other: &$idtype) -> Option<std::cmp::Ordering> {
                self.partial_cmp(&other.0)
            }
        }

        impl Default for $idtype {
            fn default() -> Self {
                Self::NULL
            }
        }
    };
}

macro_rules! impl_size_type_comparisons_for_row_ids {
    ($idtype: ty) => {
        impl PartialEq<$idtype> for $crate::SizeType {
            fn eq(&self, other: &$idtype) -> bool {
                self.0 == other.0 as $crate::sys::bindings::tsk_size_t
            }
        }

        impl PartialEq<$crate::SizeType> for $idtype {
            fn eq(&self, other: &$crate::SizeType) -> bool {
                (self.0 as $crate::sys::bindings::tsk_size_t) == other.0
            }
        }

        impl PartialOrd<$idtype> for $crate::SizeType {
            fn partial_cmp(&self, other: &$idtype) -> Option<std::cmp::Ordering> {
                self.0
                    .partial_cmp(&(other.0 as $crate::sys::bindings::tsk_size_t))
            }
        }

        impl PartialOrd<$crate::SizeType> for $idtype {
            fn partial_cmp(&self, other: &$crate::SizeType) -> Option<std::cmp::Ordering> {
                (self.0 as $crate::sys::bindings::tsk_size_t).partial_cmp(&other.0)
            }
        }
    };
}

macro_rules! impl_f64_newtypes {
    ($type: ty) => {
        impl std::fmt::Display for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl PartialEq<f64> for $type {
            fn eq(&self, other: &f64) -> bool {
                self.0.eq(other)
            }
        }

        impl PartialEq<$type> for f64 {
            fn eq(&self, other: &$type) -> bool {
                self.eq(&other.0)
            }
        }

        impl PartialOrd<f64> for $type {
            fn partial_cmp(&self, other: &f64) -> Option<std::cmp::Ordering> {
                self.0.partial_cmp(other)
            }
        }

        impl PartialOrd<$type> for f64 {
            fn partial_cmp(&self, other: &$type) -> Option<std::cmp::Ordering> {
                self.partial_cmp(&other.0)
            }
        }

        impl From<f64> for $type {
            fn from(value: f64) -> Self {
                Self(value)
            }
        }

        impl From<$type> for f64 {
            fn from(value: $type) -> Self {
                value.0
            }
        }

        impl std::ops::Sub for $type {
            type Output = Self;

            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.0 - rhs.0)
            }
        }

        impl std::ops::SubAssign for $type {
            fn sub_assign(&mut self, rhs: Self) {
                self.0 -= rhs.0
            }
        }

        impl std::ops::Add for $type {
            type Output = Self;

            fn add(self, rhs: Self) -> Self::Output {
                Self(self.0 + rhs.0)
            }
        }

        impl std::ops::AddAssign for $type {
            fn add_assign(&mut self, rhs: Self) {
                self.0 += rhs.0
            }
        }

        impl std::ops::Mul for $type {
            type Output = Self;

            fn mul(self, rhs: Self) -> Self::Output {
                Self(self.0 * rhs.0)
            }
        }

        impl std::ops::MulAssign for $type {
            fn mul_assign(&mut self, rhs: Self) {
                self.0.mul_assign(&rhs.0)
            }
        }

        impl std::ops::Div for $type {
            type Output = Self;

            fn div(self, rhs: Self) -> Self::Output {
                Self(self.0 / rhs.0)
            }
        }

        impl std::ops::DivAssign for $type {
            fn div_assign(&mut self, rhs: Self) {
                self.0.div_assign(&rhs.0)
            }
        }
    };
}

macro_rules! impl_time_position_arithmetic {
    ($lhs: ty, $rhs:ty) => {
        impl std::ops::Mul<$rhs> for $lhs {
            type Output = $lhs;

            fn mul(self, other: $rhs) -> Self {
                Self(self.0.mul(&other.0))
            }
        }

        impl std::ops::MulAssign<$rhs> for $lhs {
            fn mul_assign(&mut self, other: $rhs) {
                self.0.mul_assign(&other.0)
            }
        }

        impl std::ops::Div<$rhs> for $lhs {
            type Output = $lhs;

            fn div(self, other: $rhs) -> Self {
                Self(self.0.div(&other.0))
            }
        }

        impl std::ops::DivAssign<$rhs> for $lhs {
            fn div_assign(&mut self, other: $rhs) {
                self.0.div_assign(&other.0)
            }
        }
    };
}

/// Convenience macro to handle implementing
/// [`crate::metadata::MetadataRoundtrip`]
#[macro_export]
macro_rules! handle_metadata_return {
    ($e: expr) => {
        match $e {
            Ok(x) => Ok(x),
            Err(e) => Err($crate::metadata::MetadataError::RoundtripError { value: Box::new(e) }),
        }
    };
}

macro_rules! raw_metadata_getter_for_tables {
    ($idtype: ty) => {
        fn raw_metadata<I: Into<$idtype>>(&self, row: I) -> Option<&[u8]> {
            $crate::sys::tsk_ragged_column_access::<'_, u8, $idtype, _, _>(
                row.into(),
                self.as_ref().metadata,
                self.num_rows(),
                self.as_ref().metadata_offset,
                self.as_ref().metadata_length,
            )
        }
    };
}

macro_rules! row_lending_iterator_get {
    () => {
        fn get(&self) -> Option<&Self::Item> {
            if crate::SizeType::try_from(self.id).ok()? < self.table.num_rows() {
                Some(self)
            } else {
                None
            }
        }
    };
}

macro_rules! optional_container_comparison {
    ($lhs: expr, $rhs: expr) => {
        if let Some(value) = &$lhs {
            if let Some(ovalue) = &$rhs {
                if value.len() != ovalue.len() {
                    return false;
                }
                if value.iter().zip(ovalue.iter()).any(|(a, b)| a != b) {
                    false
                } else {
                    true
                }
            } else {
                false
            }
        } else if $rhs.is_some() {
            false
        } else {
            true
        }
    };
}

macro_rules! build_table_column_slice_getter {
    ($(#[$attr:meta])* => $column: ident, $name: ident, $cast: ty) => {
        $(#[$attr])*
        pub fn $name(&self) -> &[$cast] {
            $crate::sys::generate_slice(self.as_ref().$column, self.num_rows())
        }
    };
}

macro_rules! build_table_column_slice_mut_getter {
    ($(#[$attr:meta])* => $column: ident, $name: ident, $cast: ty) => {
        $(#[$attr])*
        pub fn $name(&mut self) -> &mut [$cast] {
            $crate::sys::generate_slice_mut(self.as_ref().$column, self.num_rows())
        }
    };
}

macro_rules! delegate_table_view_api {
    () => {
        delegate::delegate! {
            to self.views {
                /// Get reference to the [``EdgeTable``](crate::EdgeTable).
                pub fn edges(&self) -> &crate::EdgeTable;
                /// Get reference to the [``IndividualTable``](crate::IndividualTable).
                pub fn individuals(&self) -> &crate::IndividualTable;
                /// Get reference to the [``MigrationTable``](crate::MigrationTable).
                pub fn migrations(&self) -> &crate::MigrationTable;
                /// Get reference to the [``MutationTable``](crate::MutationTable).
                pub fn mutations(&self) -> &crate::MutationTable;
                /// Get reference to the [``NodeTable``](crate::NodeTable).
                pub fn nodes(&self) -> &crate::NodeTable;
                /// Get reference to the [``PopulationTable``](crate::PopulationTable).
                pub fn populations(&self) -> &crate::PopulationTable;
                /// Get reference to the [``SiteTable``](crate::SiteTable).
                pub fn sites(&self) -> &crate::SiteTable;

                #[cfg(feature = "provenance")]
                #[cfg_attr(doc_cfg, doc(cfg(feature = "provenance")))]
                /// Get reference to the [``ProvenanceTable``](crate::provenance::ProvenanceTable)
                pub fn provenances(&self) -> &crate::provenance::ProvenanceTable ;

                /// Return an iterator over the individuals.
                pub fn individuals_iter(&self) -> impl Iterator<Item = crate::IndividualTableRow> + '_;
                /// Return an iterator over the nodes.
                pub fn nodes_iter(&self) -> impl Iterator<Item = crate::NodeTableRow> + '_;
                /// Return an iterator over the edges.
                pub fn edges_iter(&self) -> impl Iterator<Item = crate::EdgeTableRow> + '_;
                /// Return an iterator over the migrations.
                pub fn migrations_iter(&self) -> impl Iterator<Item = crate::MigrationTableRow> + '_;
                /// Return an iterator over the mutations.
                pub fn mutations_iter(&self) -> impl Iterator<Item = crate::MutationTableRow> + '_;
                /// Return an iterator over the populations.
                pub fn populations_iter(&self) -> impl Iterator<Item = crate::PopulationTableRow> + '_;
                /// Return an iterator over the sites.
                pub fn sites_iter(&self) -> impl Iterator<Item = crate::SiteTableRow> + '_;

                #[cfg(feature = "provenance")]
                #[cfg_attr(doc_cfg, doc(cfg(feature = "provenance")))]
                /// Return an iterator over provenances
                pub fn provenances_iter(&self,) -> impl Iterator<Item = crate::provenance::ProvenanceTableRow> + '_;

                /// Obtain a vector containing the indexes ("ids")
                /// of all nodes for which [`crate::NodeFlags::is_sample`]
                /// is `true`.
                ///
                /// The provided implementation dispatches to
                /// [`crate::NodeTable::samples_as_vector`].
                pub fn samples_as_vector(&self) -> Vec<crate::NodeId>;

                /// Obtain a vector containing the indexes ("ids") of all nodes
                /// satisfying a certain criterion.
                ///
                /// The provided implementation dispatches to
                /// [`crate::NodeTable::create_node_id_vector`].
                ///
                /// # Parameters
                ///
                /// * `f`: a function.  The function is passed the current table
                ///    collection and each [`crate::node_table::NodeTableRow`].
                ///    If `f` returns `true`, the index of that row is included
                ///    in the return value.
                ///
                /// # Examples
                ///
                /// Get all nodes with time > 0.0:
                ///
                /// ```
                /// let mut tables = tskit::TableCollection::new(100.).unwrap();
                /// tables
                ///     .add_node(tskit::NodeFlags::new_sample(), 0.0, tskit::PopulationId::NULL,
                ///     tskit::IndividualId::NULL)
                ///     .unwrap();
                /// tables
                ///     .add_node(tskit::NodeFlags::new_sample(), 1.0, tskit::PopulationId::NULL,
                ///     tskit::IndividualId::NULL)
                ///     .unwrap();
                /// let samples = tables.create_node_id_vector(
                ///     |row: &tskit::NodeTableRow| row.time > 0.,
                /// );
                /// assert_eq!(samples[0], 1);
                /// ```
                pub fn create_node_id_vector(&self, f: impl FnMut(&crate::NodeTableRow) -> bool) -> Vec<crate::NodeId>;
            }
        }
    };
}

#[cfg(test)]
mod test {
    use crate::error::TskitError;
    use crate::TskReturnValue;

    #[test]
    #[should_panic]
    fn test_tskit_panic() {
        panic_on_tskit_error!(-202); // "Node out of bounds"
    }

    fn return_value_mock(rv: i32) -> TskReturnValue {
        handle_tsk_return_value!(rv)
    }

    fn must_not_error(x: TskReturnValue) -> bool {
        x.map_or_else(|_: TskitError| false, |_| true)
    }

    fn must_error(x: TskReturnValue) -> bool {
        x.map_or_else(|_: TskitError| true, |_| false)
    }

    #[test]
    fn test_handle_good_return_value() {
        assert!(must_not_error(return_value_mock(0)));
        assert!(must_not_error(return_value_mock(1)));
    }

    #[test]
    fn test_handle_return_value_test_panic() {
        assert!(must_error(return_value_mock(-207)));
    }
}
