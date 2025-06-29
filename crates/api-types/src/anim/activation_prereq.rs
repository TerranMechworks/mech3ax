use crate::{api, sum};

api! {
    struct PrerequisiteAnimation: Val {
        name: String,
        required: bool,
    }
}

api! {
    struct PrerequisiteObject: Val {
        name: String,
        required: bool,
        active: bool,
        ptr: u32,
    }
}

api! {
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
