use std::ptr::NonNull;

use super::bindings::tsk_flags_t;
use super::bindings::tsk_id_t;
use super::bindings::tsk_individual_table_add_row;
use super::bindings::tsk_individual_table_clear;
use super::bindings::tsk_individual_table_init;
use super::bindings::tsk_individual_table_t;
use super::tskbox::TskBox;
use super::Error;

#[derive(Debug)]
pub struct IndividualTable(TskBox<tsk_individual_table_t>);

impl IndividualTable {
    pub fn new(options: u32) -> Result<Self, Error> {
        let tsk = TskBox::new(|e: *mut tsk_individual_table_t| unsafe {
            tsk_individual_table_init(e, options)
        })?;
        Ok(Self(tsk))
    }

    pub unsafe fn new_borrowed(ptr: NonNull<tsk_individual_table_t>) -> Self {
        let tsk = TskBox::new_init_from_ptr(ptr);
        Self(tsk)
    }

    pub fn as_ref(&self) -> &tsk_individual_table_t {
        self.0.as_ref()
    }

    pub fn as_mut(&mut self) -> &mut tsk_individual_table_t {
        self.0.as_mut()
    }

    pub fn clear(&mut self) -> i32 {
        unsafe { tsk_individual_table_clear(self.as_mut()) }
    }

    pub fn add_row(
        &mut self,
        flags: tsk_flags_t,
        location: &[f64],
        parents: &[tsk_id_t],
    ) -> Result<tsk_id_t, Error> {
        self.add_row_with_metadata(flags, location, parents, &[])
    }

    pub fn add_row_with_metadata(
        &mut self,
        flags: tsk_flags_t,
        location: &[f64],
        parents: &[tsk_id_t],
        metadata: &[u8],
    ) -> Result<tsk_id_t, Error> {
        unsafe {
            Ok(tsk_individual_table_add_row(
                self.as_mut(),
                flags,
                location.as_ptr(),
                location.len() as u64,
                parents.as_ptr(),
                parents.len() as u64,
                metadata.as_ptr().cast::<i8>(),
                metadata.len() as u64,
            ))
        }
    }
}

impl Default for IndividualTable {
    fn default() -> Self {
        Self::new(0).unwrap()
    }
}
