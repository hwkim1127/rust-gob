use std::io::Write;

use serde::ser::{self, Serialize};

use error::Error;
use internal::gob::Stream;
use internal::ser::{SerializationCtx, SerializeTupleValue};
use internal::types::TypeId;
use internal::utils::Bow;
use schema::Schema;

pub struct SerializeTuple<'t, W> {
    inner: SerializeTupleValue<Bow<'t, Schema>>,
    out: Stream<W>,
}

impl<'t, W: Write> SerializeTuple<'t, W> {
    pub(crate) fn homogeneous(
        type_id: TypeId,
        ctx: SerializationCtx<Bow<'t, Schema>>,
        out: Stream<W>,
    ) -> Result<Self, Error> {
        Ok(SerializeTuple {
            inner: SerializeTupleValue::homogeneous(ctx, type_id)?,
            out,
        })
    }
}

impl<'t, W: Write> ser::SerializeTuple for SerializeTuple<'t, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.inner.serialize_element(value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let type_id = self.inner.type_id();
        let mut ok = self.inner.end()?;
        ok.ctx.flush(type_id, self.out)
    }
}
