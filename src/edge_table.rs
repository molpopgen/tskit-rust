use crate::bindings as ll_bindings;
use crate::metadata;
use crate::sys;
use crate::Position;
use crate::{tsk_id_t, TskitError};
use crate::{EdgeId, NodeId};

/// Row of an [`EdgeTable`]
#[derive(Debug)]
pub struct EdgeTableRow {
    pub id: EdgeId,
    pub left: Position,
    pub right: Position,
    pub parent: NodeId,
    pub child: NodeId,
    pub metadata: Option<Vec<u8>>,
}

impl PartialEq for EdgeTableRow {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.parent == other.parent
            && self.child == other.child
            && crate::util::partial_cmp_equal(&self.left, &other.left)
            && crate::util::partial_cmp_equal(&self.right, &other.right)
            && self.metadata == other.metadata
    }
}

fn make_edge_table_row(table: &EdgeTable, pos: tsk_id_t) -> Option<EdgeTableRow> {
    Some(EdgeTableRow {
        id: pos.into(),
        left: table.left(pos)?,
        right: table.right(pos)?,
        parent: table.parent(pos)?,
        child: table.child(pos)?,
        metadata: table.raw_metadata(pos).map(|m| m.to_vec()),
    })
}

pub(crate) type EdgeTableRefIterator<'a> = crate::table_iterator::TableIterator<&'a EdgeTable>;
pub(crate) type EdgeTableIterator = crate::table_iterator::TableIterator<EdgeTable>;

impl<'a> Iterator for EdgeTableRefIterator<'a> {
    type Item = EdgeTableRow;

    fn next(&mut self) -> Option<Self::Item> {
        let rv = make_edge_table_row(self.table, self.pos);
        self.pos += 1;
        rv
    }
}

impl Iterator for EdgeTableIterator {
    type Item = EdgeTableRow;

    fn next(&mut self) -> Option<Self::Item> {
        let rv = make_edge_table_row(&self.table, self.pos);
        self.pos += 1;
        rv
    }
}

/// Row of an [`EdgeTable`]
#[derive(Debug)]
pub struct EdgeTableRowView<'a> {
    table: &'a EdgeTable,
    pub id: EdgeId,
    pub left: Position,
    pub right: Position,
    pub parent: NodeId,
    pub child: NodeId,
    pub metadata: Option<&'a [u8]>,
}

impl<'a> EdgeTableRowView<'a> {
    fn new(table: &'a EdgeTable) -> Self {
        Self {
            table,
            id: (-1).into(),
            left: f64::NAN.into(),
            right: f64::NAN.into(),
            parent: NodeId::NULL,
            child: NodeId::NULL,
            metadata: None,
        }
    }
}

impl<'a> PartialEq for EdgeTableRowView<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.parent == other.parent
            && self.child == other.child
            && crate::util::partial_cmp_equal(&self.left, &other.left)
            && crate::util::partial_cmp_equal(&self.right, &other.right)
            && self.metadata == other.metadata
    }
}

impl<'a> Eq for EdgeTableRowView<'a> {}

impl<'a> PartialEq<EdgeTableRow> for EdgeTableRowView<'a> {
    fn eq(&self, other: &EdgeTableRow) -> bool {
        self.id == other.id
            && self.parent == other.parent
            && self.child == other.child
            && crate::util::partial_cmp_equal(&self.left, &other.left)
            && crate::util::partial_cmp_equal(&self.right, &other.right)
            && optional_container_comparison!(self.metadata, other.metadata)
    }
}

impl PartialEq<EdgeTableRowView<'_>> for EdgeTableRow {
    fn eq(&self, other: &EdgeTableRowView) -> bool {
        self.id == other.id
            && self.parent == other.parent
            && self.child == other.child
            && crate::util::partial_cmp_equal(&self.left, &other.left)
            && crate::util::partial_cmp_equal(&self.right, &other.right)
            && optional_container_comparison!(self.metadata, other.metadata)
    }
}

impl<'a> streaming_iterator::StreamingIterator for EdgeTableRowView<'a> {
    type Item = Self;

    row_lending_iterator_get!();

    fn advance(&mut self) {
        self.id = (i32::from(self.id) + 1).into();
        self.left = self.table.left(self.id).unwrap_or_else(|| f64::NAN.into());
        self.right = self.table.right(self.id).unwrap_or_else(|| f64::NAN.into());
        self.parent = self.table.parent(self.id).unwrap_or(NodeId::NULL);
        self.child = self.table.child(self.id).unwrap_or(NodeId::NULL);
        self.metadata = self.table.raw_metadata(self.id);
    }
}

/// An edge table.
///
/// # Examples
///
/// ## Standalone tables
///
/// ```
/// use tskit::EdgeTable;
///
/// let mut edges = EdgeTable::default();
/// let rowid = edges.add_row(1., 2., 0, 1).unwrap();
/// assert_eq!(rowid, 0);
/// assert_eq!(edges.num_rows(), 1);
///
/// edges.clear().unwrap();
/// assert_eq!(edges.num_rows(), 0);
/// ```
///
/// An example with metadata.
/// This requires the cargo feature `"derive"` for `tskit`.
///
/// ```
/// # #[cfg(any(feature="doc", feature="derive"))] {
/// use tskit::EdgeTable;
///
/// #[derive(serde::Serialize,
///          serde::Deserialize,
///          tskit::metadata::EdgeMetadata)]
/// #[serializer("serde_json")]
/// struct EdgeMetadata {
///     value: i32,
/// }
///
/// let metadata = EdgeMetadata{value: 42};
///
/// let mut edges = EdgeTable::default();
///
/// let rowid = edges.add_row_with_metadata(0., 1., 5, 10, &metadata).unwrap();
/// assert_eq!(rowid, 0);
///
/// match edges.metadata::<EdgeMetadata>(rowid) {
///     // rowid is in range, decoding succeeded
///     Some(Ok(decoded)) => assert_eq!(decoded.value, 42),
///     // rowid is in range, decoding failed
///     Some(Err(e)) => panic!("error decoding metadata: {:?}", e),
///     None => panic!("row id out of range")
/// }
/// # }
/// ```
#[repr(transparent)]
#[derive(Debug)]
pub struct EdgeTable {
    table_: sys::LLEdgeTable,
}

impl EdgeTable {
    pub(crate) fn as_ptr(&self) -> *const ll_bindings::tsk_edge_table_t {
        self.table_.as_ptr()
    }

    pub(crate) fn as_mut_ptr(&mut self) -> *mut ll_bindings::tsk_edge_table_t {
        self.table_.as_mut_ptr()
    }

    pub(crate) fn new_from_table(
        edges: *mut ll_bindings::tsk_edge_table_t,
    ) -> Result<Self, TskitError> {
        let table_ = sys::LLEdgeTable::new_non_owning(edges)?;
        Ok(EdgeTable { table_ })
    }

    pub(crate) fn as_ref(&self) -> &ll_bindings::tsk_edge_table_t {
        self.table_.as_ref()
    }

    pub fn new() -> Result<Self, TskitError> {
        let table_ = sys::LLEdgeTable::new_owning(0)?;
        Ok(Self { table_ })
    }

    /// Return the number of rows
    pub fn num_rows(&self) -> crate::SizeType {
        self.as_ref().num_rows.into()
    }

    raw_metadata_getter_for_tables!(EdgeId);

    /// Return the ``parent`` value from row ``row`` of the table.
    ///
    /// # Returns
    ///
    /// * `Some(parent)` if `u` is valid.
    /// * `None` otherwise.
    pub fn parent<E: Into<EdgeId> + Copy>(&self, row: E) -> Option<NodeId> {
        sys::tsk_column_access::<NodeId, _, _, _>(row.into(), self.as_ref().parent, self.num_rows())
    }

    /// Return the ``child`` value from row ``row`` of the table.
    ///
    /// # Returns
    ///
    /// * `Some(child)` if `u` is valid.
    /// * `None` otherwise.
    pub fn child<E: Into<EdgeId> + Copy>(&self, row: E) -> Option<NodeId> {
        sys::tsk_column_access::<NodeId, _, _, _>(row.into(), self.as_ref().child, self.num_rows())
    }

    /// Return the ``left`` value from row ``row`` of the table.
    ///
    /// # Returns
    ///
    /// * `Some(position)` if `u` is valid.
    /// * `None` otherwise.
    pub fn left<E: Into<EdgeId> + Copy>(&self, row: E) -> Option<Position> {
        sys::tsk_column_access::<Position, _, _, _>(row.into(), self.as_ref().left, self.num_rows())
    }

    /// Return the ``right`` value from row ``row`` of the table.
    ///
    /// # Returns
    ///
    /// * `Some(position)` if `u` is valid.
    /// * `None` otherwise.
    pub fn right<E: Into<EdgeId> + Copy>(&self, row: E) -> Option<Position> {
        sys::tsk_column_access::<Position, _, _, _>(
            row.into(),
            self.as_ref().right,
            self.num_rows(),
        )
    }

    /// Retrieve decoded metadata for a `row`.
    ///
    /// # Returns
    ///
    /// * `Some(Ok(T))` if `row` is valid and decoding succeeded.
    /// * `Some(Err(_))` if `row` is not valid and decoding failed.
    /// * `None` if `row` is not valid.
    ///
    /// # Errors
    ///
    /// * [`TskitError::MetadataError`] if decoding fails.
    ///
    /// # Examples.
    ///
    /// The big-picture semantics are the same for all table types.
    /// See [`crate::IndividualTable::metadata`] for examples.
    pub fn metadata<T: metadata::EdgeMetadata>(
        &self,
        row: EdgeId,
    ) -> Option<Result<T, TskitError>> {
        let buffer = self.raw_metadata(row)?;
        Some(decode_metadata_row!(T, buffer).map_err(|e| e.into()))
    }

    /// Return an iterator over rows of the table.
    /// The value of the iterator is [`EdgeTableRow`].
    ///
    pub fn iter(&self) -> impl Iterator<Item = EdgeTableRow> + '_ {
        crate::table_iterator::make_table_iterator::<&EdgeTable>(self)
    }

    pub fn lending_iter(&self) -> EdgeTableRowView {
        EdgeTableRowView::new(self)
    }

    /// Return row `r` of the table.
    ///
    /// # Parameters
    ///
    /// * `r`: the row id.
    ///
    /// # Returns
    ///
    /// * `Some(row)` if `r` is valid
    /// * `None` otherwise
    pub fn row<E: Into<EdgeId> + Copy>(&self, r: E) -> Option<EdgeTableRow> {
        table_row_access!(r.into().into(), self, make_edge_table_row)
    }

    /// Return a view of row `r` of the table.
    ///
    /// # Parameters
    ///
    /// * `r`: the row id.
    ///
    /// # Returns
    ///
    /// * `Some(row_view)` if `r` is valid
    /// * `None` otherwise
    pub fn row_view<E: Into<EdgeId> + Copy>(&self, r: E) -> Option<EdgeTableRowView> {
        let view = EdgeTableRowView {
            table: self,
            id: r.into(),
            left: self.left(r)?,
            right: self.right(r)?,
            parent: self.parent(r)?,
            child: self.child(r)?,
            metadata: self.raw_metadata(r.into()),
        };
        Some(view)
    }

    pub fn clear(&mut self) -> Result<(), TskitError> {
        self.table_.clear().map_err(|e| e.into())
    }

    fn add_row_details<L, R, P, C>(
        &mut self,
        left: L,
        right: R,
        parent: P,
        child: C,
        metadata: *const i8,
        metadata_len: u64,
    ) -> Result<crate::EdgeId, crate::TskitError>
    where
        L: Into<crate::Position>,
        R: Into<crate::Position>,
        P: Into<crate::NodeId>,
        C: Into<crate::NodeId>,
    {
        let rv = unsafe {
            crate::bindings::tsk_edge_table_add_row(
                self.as_mut_ptr(),
                left.into().into(),
                right.into().into(),
                parent.into().into(),
                child.into().into(),
                metadata,
                metadata_len,
            )
        };
        handle_tsk_return_value!(rv, rv.into())
    }

    pub fn add_row<L, R, P, C>(
        &mut self,
        left: L,
        right: R,
        parent: P,
        child: C,
    ) -> Result<crate::EdgeId, crate::TskitError>
    where
        L: Into<crate::Position>,
        R: Into<crate::Position>,
        P: Into<crate::NodeId>,
        C: Into<crate::NodeId>,
    {
        self.add_row_details(left, right, parent, child, std::ptr::null(), 0)
    }

    pub fn add_row_with_metadata<L, R, P, C, M>(
        &mut self,
        left: L,
        right: R,
        parent: P,
        child: C,
        metadata: &M,
    ) -> Result<crate::EdgeId, crate::TskitError>
    where
        L: Into<crate::Position>,
        R: Into<crate::Position>,
        P: Into<crate::NodeId>,
        C: Into<crate::NodeId>,
        M: crate::metadata::EdgeMetadata,
    {
        let md = crate::metadata::EncodedMetadata::new(metadata)?;
        let mdlen = md.len()?;
        self.add_row_details(left, right, parent, child, md.as_ptr(), mdlen.into())
    }

    build_table_column_slice_getter!(
        /// Get the left column as a slice
        => left, left_slice, Position);
    build_table_column_slice_getter!(
        /// Get the left column as a slice of [`f64`] 
        => left, left_slice_raw, f64);
    build_table_column_slice_getter!(
        /// Get the right column as a slice
        => right, right_slice, Position);
    build_table_column_slice_getter!(
        /// Get the left column as a slice of [`f64`] 
        => right, right_slice_raw, f64);
    build_table_column_slice_getter!(
        /// Get the parent column as a slice
        => parent, parent_slice, NodeId);
    build_table_column_slice_getter!(
        /// Get the parent column as a slice of [`crate::bindings::tsk_id_t`]
        => parent, parent_slice_raw, ll_bindings::tsk_id_t);
    build_table_column_slice_getter!(
        /// Get the child column as a slice
        => child, child_slice, NodeId);
    build_table_column_slice_getter!(
        /// Get the child column as a slice of [`crate::bindings::tsk_id_t`]
        => child, child_slice_raw, ll_bindings::tsk_id_t);
}

impl Default for EdgeTable {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

#[deprecated(since = "0.13.2", note = "use EdgeTable instead")]
pub type OwningEdgeTable = EdgeTable;
