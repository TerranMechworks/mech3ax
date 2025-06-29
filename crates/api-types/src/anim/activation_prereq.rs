use crate::{fld, sum};

fld! {
    struct PrerequisiteAnimation: Val {
        name: String,
        required: bool,
    }
}

fld! {
    struct PrerequisiteObject: Val {
        name: String,
        required: bool,
        active: bool,
        ptr: u32,
    }
}

fld! {
    struct PrerequisiteParent: Val {
        name: String,
        required: bool,
        active: bool,
        ptr: u32,
    }
}

sum! {
    enum ActivationPrerequisite {
        Animation(PrerequisiteAnimation),
        Parent(PrerequisiteParent),
        Object(PrerequisiteObject),
    }
}
