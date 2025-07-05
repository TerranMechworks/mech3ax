pub trait CStruct {
    type FieldOffsets;
    fn __field_offsets(&self) -> &'static Self::FieldOffsets;
}

impl<T: CStruct> CStruct for &T {
    type FieldOffsets = T::FieldOffsets;

    #[inline]
    fn __field_offsets(&self) -> &'static Self::FieldOffsets {
        (*self).__field_offsets()
    }
}
