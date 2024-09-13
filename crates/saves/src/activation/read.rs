use super::{AnimActivationC, VALUES_SIZE};
use log::trace;
use mech3ax_api_types::saves::{ActivationType, AnimActivation};
use mech3ax_api_types::Bytes;
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::{assert_that, assert_with_msg, Result};
use std::io::{Cursor, Read};
use std::num::NonZeroU32;

pub fn read_activation(read: &mut CountingReader<impl Read>) -> Result<AnimActivation> {
    let activation: AnimActivationC = read.read_struct()?;
    // [1, 2, 3, 4, 5]
    assert_that!("anim activation type", activation.type_ in [1, 2, 5], read.prev + 0)?;
    assert_that!(
        "anim activation unk04",
        activation.unk04 == 0,
        read.prev + 4
    )?;

    let name = assert_utf8("anim activation name", read.prev + 8, || {
        activation.name.to_str_padded()
    })?;

    let node_index = if activation.node_index < 0 {
        assert_that!(
            "anim activation node index",
            activation.node_index == -1,
            read.prev + 40
        )?;
        None
    } else {
        assert_that!(
            "anim activation node index",
            activation.node_index < 20000,
            read.prev + 40
        )?;
        Some(activation.node_index)
    };

    let status = assert_that!("anim activation status", enum activation.status, read.prev + 0)?;

    let ptr = match activation.unk86 {
        0 => {
            assert_that!(
                "anim activation unk80",
                activation.unk80 == 0,
                read.prev + 80
            )?;
            None
        }
        25 => {
            assert_that!(
                "anim activation unk80",
                activation.unk80 != 0,
                read.prev + 80
            )?;
            NonZeroU32::new(activation.unk80)
        }
        _ => {
            return Err(assert_with_msg!(
                "Expected anim activation unk86 to be either 0 or 25, but was {0} (at {1})",
                activation.unk86,
                read.prev + 86
            ));
        }
    };

    assert_that!(
        "anim activation unk87",
        activation.unk87 == 0,
        read.prev + 87
    )?;

    let mut values = CountingReader::new(Cursor::new(activation.values));
    values.offset = read.prev + 44;

    let type_ = match activation.type_ {
        1 => {
            for i in 0..9 {
                let value = values.read_f32()?;
                let name = format!("anim activation type 1 value {0}", i);
                assert_that!(&name, value == 0.0, values.prev)?;
            }
            values.assert_end()?;
            ActivationType::One
        }
        2 => {
            let value = values.read_f32()?;
            assert_that!("anim activation type 1 value 1", value == 0.0, values.prev)?;
            let value = values.read_f32()?;
            assert_that!("anim activation type 1 value 2", value == 0.0, values.prev)?;
            let value = values.read_f32()?;
            assert_that!("anim activation type 1 value 3", value == 0.0, values.prev)?;

            let mut unzeroed = vec![0u8; 6 * 4];
            values.read_exact(&mut unzeroed[..])?;
            values.assert_end()?;

            if unzeroed.iter().all(|&v| v == 0) {
                ActivationType::Two(None)
            } else {
                ActivationType::Two(Some(Bytes(unzeroed)))
            }
        }
        5 => {
            assert_that!(
                "anim activation count",
                activation.count == 0,
                read.prev + 85
            )?;

            let mut unzeroed = vec![0u8; VALUES_SIZE];
            values.read_exact(&mut unzeroed[..])?;
            values.assert_end()?;

            if unzeroed.iter().all(|&v| v == 0) {
                ActivationType::Five(None)
            } else {
                ActivationType::Five(Some(Bytes(unzeroed)))
            }
        }
        _ => unreachable!(),
    };

    let node_states = (0..activation.count)
        .map(|i| {
            trace!("Reading node state {} ({}) at {}", i, 68, read.offset);
            let mut buf = vec![0u8; 68];
            read.read_exact(&mut buf)?;
            Ok(buf)
        })
        .collect::<Result<_>>()?;

    Ok(AnimActivation {
        name,
        node_index,
        status,
        type_,
        node_states,
        ptr,
    })
}
