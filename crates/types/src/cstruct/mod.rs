pub trait CStruct {
    type FieldOffsets: 'static;
    const __NAME: &'static str;
    const __FIELD_OFFSETS: &'static Self::FieldOffsets;
}

impl<T: CStruct> CStruct for &T {
    type FieldOffsets = T::FieldOffsets;
    const __NAME: &'static str = T::__NAME;
    const __FIELD_OFFSETS: &'static Self::FieldOffsets = T::__FIELD_OFFSETS;
}
