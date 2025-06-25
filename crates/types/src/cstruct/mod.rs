pub trait CStruct {
    type FieldOffsets;
    fn __field_offsets(&self) -> &'static Self::FieldOffsets;
}
