#[allow(clippy::octal_escapes)]
const RC_ANIM_NAMES: &[(&[u8; 32], &str)] = &[
    (
        b"bft_exhaust\0l\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        "bft_exhaust",
    ),
    (
        b"bftsplash\0lt\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        "bftsplash",
    ),
    (
        b"blueparts\0ts.flt\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        "blueparts",
    ),
    (
        b"bluesparks\0s.flt\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        "bluesparks",
    ),
    (
        b"bubble1\0flt\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        "bubble1",
    ),
    (
        b"chutes\0flt\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        "chutes",
    ),
    (
        b"csinwave\0locator\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        "csinwave",
    ),
    (
        b"fire_40\0k.flt\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        "fire_40",
    ),
    (
        b"fire_bft\0flt\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        "fire_bft",
    ),
    (
        b"force_off\0d\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        "force_off",
    ),
    (
        b"fring01\0.flt\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        "fring01",
    ),
    (
        b"greenparts\0.flt\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        "greenparts",
    ),
    (
        b"hitboom\0m\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        "hitboom",
    ),
    (
        b"litening\0flt\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        "litening",
    ),
    (
        b"nbfrc_field\0e\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        "nbfrc_field",
    ),
    (
        b"op_beam\0ghts\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        "op_beam",
    ),
    (
        b"orangeparts\0ts.flt\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        "orangeparts",
    ),
    (
        b"powvoice\0n\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        "powvoice",
    ),
    (
        b"rebel_vw\0el\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        "rebel_vw",
    ),
    (
        b"red_light\0lt\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        "red_light",
    ),
    (
        b"redparts\0ts.flt\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        "redparts",
    ),
    (
        b"ring_fire\0lt\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        "ring_fire",
    ),
    (
        b"ring_red.flt\0lt\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        "ring_red.flt",
    ),
    (
        b"siren1\0de\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        "siren1",
    ),
    (
        b"siren2\0de\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        "siren2",
    ),
    (
        b"siren3\0de\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        "siren3",
    ),
    (
        b"siren4\0de\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        "siren4",
    ),
    (
        b"smoke_destroy1\0lt\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        "smoke_destroy1",
    ),
    (
        b"smoke_destroy\0flt\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        "smoke_destroy",
    ),
    (
        b"smorangeparts\0ts.flt\0\0\0\0\0\0\0\0\0\0\0\0",
        "smorangeparts",
    ),
    (
        b"snake_loop\0gn\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        "snake_loop",
    ),
    (
        b"subf_hint\0d\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        "subf_hint",
    ),
    (
        b"switch01\0h_01\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        "switch01",
    ),
    (
        b"switch10\0h_01\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        "switch10",
    ),
    (
        b"yellowparts\0flt\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        "yellowparts",
    ),
];
