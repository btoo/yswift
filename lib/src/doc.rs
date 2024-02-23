use crate::array::YrsArray;
use crate::error::CodingError;
use crate::map::YrsMap;
use crate::text::YrsText;
use crate::transaction::YrsTransaction;
use std::sync::Arc;
use std::{borrow::Borrow, cell::RefCell};
use yrs::{updates::decoder::Decode, ArrayRef, Doc, OffsetKind, Options, StateVector, Transact, Origin};
use yrs::{MapRef, ReadTxn};
use crate::{UniffiCustomTypeConverter, YrsSharedRef};
use crate::undo::YrsUndoManager;

pub(crate) struct YrsDoc(RefCell<Doc>);

unsafe impl Send for YrsDoc {}
unsafe impl Sync for YrsDoc {}

impl YrsDoc {
    pub(crate) fn new() -> Self {
        let mut options = Options::default();
        options.offset_kind = OffsetKind::Utf16;
        let doc = yrs::Doc::with_options(options);

        Self(RefCell::from(doc))
    }

    pub(crate) fn encode_diff_v1(
        &self,
        transaction: &YrsTransaction,
        state_vector: Vec<u8>,
    ) -> Result<Vec<u8>, CodingError> {
        let mut tx = transaction.transaction();
        let tx = tx.as_mut().unwrap();

        StateVector::decode_v1(state_vector.borrow())
            .map_err(|_e| CodingError::DecodingError)
            .map(|sv| tx.encode_diff_v1(&sv))
    }

    pub(crate) fn get_text(&self, name: String) -> Arc<YrsText> {
        let text_ref = self.0.borrow().get_or_insert_text(name.as_str());
        Arc::from(YrsText::from(text_ref))
    }

    pub(crate) fn get_array(&self, name: String) -> Arc<YrsArray> {
        let array_ref: ArrayRef = self.0.borrow().get_or_insert_array(name.as_str()).into();
        Arc::from(YrsArray::from(array_ref))
    }

    pub(crate) fn get_map(&self, name: String) -> Arc<YrsMap> {
        let map_ref: MapRef = self.0.borrow().get_or_insert_map(name.as_str()).into();
        Arc::from(YrsMap::from(map_ref))
    }

    pub(crate) fn transact<'doc>(&self, origin: Option<YrsOrigin>) -> Arc<YrsTransaction> {
        let tx = self.0.borrow();
        let tx = tx.transact_mut();
        Arc::from(YrsTransaction::from(tx))
    }

    pub(crate) fn undo_manager(&self, tracked_refs: Vec<Arc<dyn YrsSharedRef>>) -> Arc<YrsUndoManager> {
        todo!()
    }
}

#[derive(Clone)]
pub struct YrsOrigin(Arc<[u8]>);

impl From<Origin> for YrsOrigin {
    fn from(value: Origin) -> Self {
        YrsOrigin(Arc::from(value.as_ref()))
    }
}

impl Into<Origin> for YrsOrigin {
    fn into(self) -> Origin {
        Origin::from(self.0.as_ref())
    }
}

impl UniffiCustomTypeConverter for YrsOrigin {
    type Builtin = Vec<u8>;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> where Self: Sized {
        Ok(YrsOrigin(val.into()))
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.0.to_vec()
    }
}