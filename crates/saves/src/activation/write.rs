use super::{AnimActivationC, VALUES_SIZE};
use mech3ax_api_types::saves::{ActivationType, AnimActivation};
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_common::{assert_with_msg, Result};
use mech3ax_types::Ascii;
use std::convert::TryInto;
use std::io::Write;

pub fn write_activation(
    write: &mut CountingWriter<impl Write>,
    activation: &AnimActivation,
) -> Result<()> {
    let type_ = match activation.type_ {
        ActivationType::One => 1,
        ActivationType::Two(_) => 2,
        ActivationType::Five(_) => 5,
    };
    let node_index = activation.node_index.unwrap_or(-1);
    let status = activation.status.maybe();
    let (unk80, unk86) = match activation.ptr {
        Some(ptr) => (ptr.into(), 25),
        None => (0, 0),
    };
    let count = activation.node_states.len().try_into().map_err(|_| {
        assert_with_msg!(
            "Expected activation to have {} node states or fewer, but was {}",
            u8::MAX,
            activation.node_states.len()
        )
    })?;
    let name = Ascii::from_str_padded(&activation.name);

    let mut activ = AnimActivationC {
        type_,
        unk04: 0,
        name,
        node_index,
        values: [0u8; VALUES_SIZE],
        unk80,
        status,
        count,
        unk86,
        unk87: 0,
    };

    match activation.type_ {
        ActivationType::One | ActivationType::Two(None) | ActivationType::Five(None) => {
            // nothing to do, values is already zero
        }
        ActivationType::Two(Some(ref values)) => {
            let src: &[u8; 6 * 4] = &values.0[..].try_into().map_err(|_| {
                assert_with_msg!(
                    "Expected activation type to have exactly {} bytes, but was {}",
                    6 * 4,
                    values.0.len()
                )
            })?;
            let (_one, two) = activ.values.split_at_mut(3 * 4);
            two.copy_from_slice(src);
        }
        ActivationType::Five(Some(ref values)) => {
            let src: &[u8; VALUES_SIZE] = &values.0[..].try_into().map_err(|_| {
                assert_with_msg!(
                    "Expected activation type to have exactly {} bytes, but was {}",
                    VALUES_SIZE,
                    values.0.len()
                )
            })?;
            activ.values.copy_from_slice(src);
        }
    }

    write.write_struct(&activ)?;
    for node_state in &activation.node_states {
        write.write_all(node_state)?;
    }
    Ok(())
}
