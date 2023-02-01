//! Optional module for table and tree sequence provenance tables.
//!
//! This module is enabled via the `"provenance"` feature and provides
//! the following:
//!
//! * [`crate::TableCollection::add_provenance`]
//! * [`crate::TreeSequence::add_provenance`]
//! * [`ProvenanceTable`].
//! * [`ProvenanceTableRow`], which is the value type returned by
//!   [`ProvenanceTable::iter`].
//!

use crate::bindings as ll_bindings;
use crate::sys;
use crate::SizeType;
use crate::{tsk_id_t, tsk_size_t, ProvenanceId};

#[derive(Eq, Debug)]
/// Row of a [`ProvenanceTable`].
pub struct ProvenanceTableRow {
    /// The row id
    pub id: ProvenanceId,
    /// ISO-formatted time stamp
    pub timestamp: String,
    /// The provenance record
    pub record: String,
}

impl PartialEq for ProvenanceTableRow {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.timestamp == other.timestamp && self.record == other.record
    }
}

impl std::fmt::Display for ProvenanceTableRow {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "id: {}, timestamp: {}, record: {}",
            self.id, self.timestamp, self.record,
        )
    }
}

fn make_provenance_row(table: &ProvenanceTable, pos: tsk_id_t) -> Option<ProvenanceTableRow> {
    Some(ProvenanceTableRow {
        id: pos.into(),
        timestamp: table.timestamp(pos)?.to_string(),
        record: table.record(pos)?.to_string(),
    })
}

type ProvenanceTableRefIterator<'a> = crate::table_iterator::TableIterator<&'a ProvenanceTable>;
type ProvenanceTableIterator = crate::table_iterator::TableIterator<ProvenanceTable>;

impl<'a> Iterator for ProvenanceTableRefIterator<'a> {
    type Item = ProvenanceTableRow;

    fn next(&mut self) -> Option<Self::Item> {
        let rv = make_provenance_row(self.table, self.pos);
        self.pos += 1;
        rv
    }
}

impl Iterator for ProvenanceTableIterator {
    type Item = ProvenanceTableRow;

    fn next(&mut self) -> Option<Self::Item> {
        let rv = make_provenance_row(&self.table, self.pos);
        self.pos += 1;
        rv
    }
}

#[derive(Debug)]
pub struct ProvenanceTableRowView<'a> {
    table: &'a ProvenanceTable,
    /// The row id
    pub id: ProvenanceId,
    /// ISO-formatted time stamp
    pub timestamp: &'a str,
    /// The provenance record
    pub record: &'a str,
}

impl<'a> ProvenanceTableRowView<'a> {
    fn new(table: &'a ProvenanceTable) -> Self {
        Self {
            table,
            id: ProvenanceId::NULL,
            timestamp: "",
            record: "",
        }
    }
}

impl<'a> PartialEq for ProvenanceTableRowView<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.timestamp == other.timestamp && self.record == other.record
    }
}

impl Eq for ProvenanceTableRowView<'_> {}

impl<'a> PartialEq<ProvenanceTableRow> for ProvenanceTableRowView<'a> {
    fn eq(&self, other: &ProvenanceTableRow) -> bool {
        self.id == other.id && self.timestamp == other.timestamp && self.record == other.record
    }
}

impl PartialEq<ProvenanceTableRowView<'_>> for ProvenanceTableRow {
    fn eq(&self, other: &ProvenanceTableRowView) -> bool {
        self.id == other.id && self.timestamp == other.timestamp && self.record == other.record
    }
}

impl<'a> streaming_iterator::StreamingIterator for ProvenanceTableRowView<'a> {
    type Item = Self;

    row_lending_iterator_get!();

    fn advance(&mut self) {
        self.id = (i32::from(self.id) + 1).into();
        self.record = self.table.record(self.id).unwrap_or("");
        self.timestamp = self.table.timestamp(self.id).unwrap_or("");
    }
}

/// A provenance table.
///
/// # Notes
///
/// * The type is enabled by the `"provenance"` feature.
///
/// # Examples
///
/// # Standalone tables
///
/// ```rust
/// # #[cfg(feature = "provenance")]
/// # #[cfg_attr(doc_cfg, doc(cfg(feature = "provenance")))]
/// {
/// use tskit::provenance::ProvenanceTable;
/// let mut provenances = ProvenanceTable::default();
/// let id = provenances.add_row("message").unwrap();
/// assert_eq!(id, 0);
/// assert_eq!(provenances.num_rows(), 1);
/// # }
/// ```
#[derive(Debug)]
#[repr(transparent)]
pub struct ProvenanceTable {
    table_: sys::LLProvenanceTable,
}

impl ProvenanceTable {
    pub(crate) fn as_ptr(&self) -> *const ll_bindings::tsk_provenance_table_t {
        self.table_.as_ptr()
    }

    pub(crate) fn as_mut_ptr(&mut self) -> *mut ll_bindings::tsk_provenance_table_t {
        self.table_.as_mut_ptr()
    }

    pub(crate) fn new_from_table(
        provenances: *mut ll_bindings::tsk_provenance_table_t,
    ) -> Result<Self, crate::TskitError> {
        let table_ = sys::LLProvenanceTable::new_non_owning(provenances)?;
        Ok(ProvenanceTable { table_ })
    }

    pub(crate) fn as_ref(&self) -> &ll_bindings::tsk_provenance_table_t {
        self.table_.as_ref()
    }

    pub fn new() -> Result<Self, crate::TskitError> {
        let table_ = sys::LLProvenanceTable::new_owning(0)?;
        Ok(Self { table_ })
    }

    /// Return the number of rows
    pub fn num_rows(&self) -> SizeType {
        self.as_ref().num_rows.into()
    }

    /// Get the ISO-formatted time stamp for row `row`.
    ///
    /// # Returns
    ///
    /// * `Some(String)` if `row` is valid.
    /// * `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    ///
    /// let mut tables = tskit::TableCollection::new(10.).unwrap();
    /// assert!(tables.add_provenance("foo").is_ok());
    /// if let Some(timestamp) = tables.provenances().timestamp(0) {
    ///  // then 0 is a valid row in the provenance table
    /// }
    /// # else {
    /// # panic!("Expected Some(timestamp)");
    /// # }
    /// ```
    pub fn timestamp<P: Into<ProvenanceId> + Copy>(&self, row: P) -> Option<&str> {
        let timestamp_slice = sys::tsk_ragged_column_access(
            row.into(),
            self.as_ref().timestamp,
            self.num_rows(),
            self.as_ref().timestamp_offset,
            self.as_ref().timestamp_length,
        );
        match timestamp_slice {
            Some(tstamp) => std::str::from_utf8(tstamp).ok(),
            None => None,
        }
    }

    /// Get the provenance record for row `row`.
    ///
    /// # Returns
    ///
    /// * `Some(String)` if `row` is valid.
    /// * `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    ///
    /// let mut tables = tskit::TableCollection::new(10.).unwrap();
    /// assert!(tables.add_provenance("foo").is_ok());
    /// if let Some(record) = tables.provenances().record(0) {
    ///  // then 0 is a valid row in the provenance table
    ///  # assert_eq!(record, "foo");
    /// }
    /// # else {
    /// # panic!("Expected Some(timestamp)");
    /// # }
    pub fn record<P: Into<ProvenanceId> + Copy>(&self, row: P) -> Option<&str> {
        let record_slice = sys::tsk_ragged_column_access(
            row.into(),
            self.as_ref().record,
            self.num_rows(),
            self.as_ref().record_offset,
            self.as_ref().record_length,
        );
        match record_slice {
            Some(rec) => std::str::from_utf8(rec).ok(),
            None => None,
        }
    }

    /// Obtain a [`ProvenanceTableRow`] for row `row`.
    ///
    /// # Returns
    ///
    /// * `Some(row)` if `r` is valid
    /// * `None` otherwise
    pub fn row<P: Into<ProvenanceId> + Copy>(&self, row: P) -> Option<ProvenanceTableRow> {
        make_provenance_row(self, row.into().into())
    }

    /// Obtain a [`ProvenanceTableRowView`] for row `row`.
    ///
    /// # Returns
    ///
    /// * `Some(row view)` if `r` is valid
    /// * `None` otherwise
    pub fn row_view<P: Into<ProvenanceId> + Copy>(&self, row: P) -> Option<ProvenanceTableRowView> {
        match row.into().to_usize() {
            Some(x) if (x as u64) < self.num_rows() => {
                let view = ProvenanceTableRowView {
                    table: self,
                    id: row.into(),
                    record: self.record(row)?,
                    timestamp: self.timestamp(row)?,
                };
                Some(view)
            }
            _ => None,
        }
    }

    /// Return an iterator over rows of the table.
    /// The value of the iterator is [`ProvenanceTableRow`].
    pub fn iter(&self) -> impl Iterator<Item = ProvenanceTableRow> + '_ {
        crate::table_iterator::make_table_iterator::<&ProvenanceTable>(self)
    }

    pub fn lending_iter(&self) -> ProvenanceTableRowView {
        ProvenanceTableRowView::new(self)
    }

    pub fn clear(&mut self) -> Result<(), crate::TskitError> {
        self.table_.clear().map_err(|e| e.into())
    }

    pub fn add_row(&mut self, record: &str) -> Result<crate::ProvenanceId, crate::TskitError> {
        if record.is_empty() {
            return Err(crate::TskitError::ValueError {
                got: "empty string".to_string(),
                expected: "provenance record".to_string(),
            });
        }
        let timestamp = humantime::format_rfc3339(std::time::SystemTime::now()).to_string();
        let rv = unsafe {
            crate::bindings::tsk_provenance_table_add_row(
                self.as_mut_ptr(),
                timestamp.as_ptr() as *mut i8,
                timestamp.len() as tsk_size_t,
                record.as_ptr() as *mut i8,
                record.len() as tsk_size_t,
            )
        };
        handle_tsk_return_value!(rv, rv.into())
    }
}

impl Default for ProvenanceTable {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

#[deprecated(since = "0.13.2", note = "use ProvenanceTable instead")]
pub type OwningProvenanceTable = ProvenanceTable;

#[cfg(test)]
mod test_provenances {
    use streaming_iterator::StreamingIterator;

    #[test]
    fn test_empty_record_string() {
        // check for tables...
        let mut tables = crate::TableCollection::new(10.).unwrap();
        let s = String::from("");
        assert!(tables.add_provenance(&s).is_err());

        // and for tree sequences...
        tables.build_index().unwrap();
        let mut ts = tables
            .tree_sequence(crate::TreeSequenceFlags::default())
            .unwrap();
        assert!(ts.add_provenance(&s).is_err())
    }

    #[test]
    fn test_add_rows() {
        let records = vec!["banana".to_string(), "split".to_string()];
        let mut tables = crate::TableCollection::new(10.).unwrap();
        for (i, r) in records.iter().enumerate() {
            let row_id = tables.add_provenance(r).unwrap();
            assert!(row_id == i as crate::tsk_id_t);
            assert_eq!(tables.provenances().record(row_id).unwrap(), *r);
        }
        assert_eq!(
            usize::try_from(tables.provenances().num_rows()).unwrap(),
            records.len()
        );
        for (i, row) in tables.provenances_iter().enumerate() {
            assert_eq!(records[i], row.record);
        }
        for (i, row) in tables.provenances().iter().enumerate() {
            assert_eq!(records[i], row.record);
        }

        assert!(tables.provenances().row(0).unwrap() == tables.provenances().row(0).unwrap());
        assert!(tables.provenances().row(0).unwrap() != tables.provenances().row(1).unwrap());

        let mut lending_iter = tables.provenances().lending_iter();
        for i in [0, 1] {
            if let Some(row) = lending_iter.next() {
                assert_eq!(row.record, &records[i]);
                let owned_row = tables.provenances().row(i as i32).unwrap();
                assert_eq!(row, &owned_row);
                assert_eq!(&owned_row, row);
            }
        }
    }
}
