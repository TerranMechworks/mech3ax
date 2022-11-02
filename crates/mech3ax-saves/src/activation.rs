use log::debug;
use mech3ax_api_types::saves::{ActivationStatus, ActivationType, AnimActivation};
use mech3ax_api_types::static_assert_size;
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::string::{str_from_c_padded, str_to_c_padded};
use mech3ax_common::{assert_that, assert_with_msg, Result};
use num_traits::FromPrimitive;
use std::convert::TryInto;
use std::io::{Cursor, Read, Write};
use std::num::NonZeroU32;

const VALUES_SIZE: usize = 9 * 4;

#[derive(Debug)]
#[repr(C)]
struct AnimActivationC {
    pub type_: i32,                // 00
    pub unk04: i32,                // 04
    pub name: [u8; 32],            // 08
    pub node_index: i32,           // 40
    pub values: [u8; VALUES_SIZE], // 44
    pub unk80: u32,                // 80
    pub status: u8,                // 84
    pub count: u8,                 // 85
    pub unk86: u8,                 // 86
    pub unk87: u8,                 // 87
}
static_assert_size!(AnimActivationC, 88);

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
        str_from_c_padded(&activation.name)
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

    let status: ActivationStatus = FromPrimitive::from_u8(activation.status).ok_or_else(|| {
        assert_with_msg!(
            "Expected valid anim activation status, but was {} (at {})",
            activation.status,
            read.prev,
        )
    })?;

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
                assert_that!(
                    format!("anim activation type 1 value {0}", i),
                    value == 0.0,
                    values.prev
                )?;
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
                ActivationType::Two(Some(unzeroed))
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
                ActivationType::Five(Some(unzeroed))
            }
        }
        _ => unreachable!(),
    };

    let node_states = (0..activation.count)
        .map(|i| {
            debug!("Reading node state {} ({}) at {}", i, 68, read.offset);
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
    let status = activation.status as u8;
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

    let mut activ = AnimActivationC {
        type_,
        unk04: 0,
        name: [0u8; 32],
        node_index,
        values: [0u8; VALUES_SIZE],
        unk80,
        status,
        count,
        unk86,
        unk87: 0,
    };

    str_to_c_padded(&activation.name, &mut activ.name);
    match activation.type_ {
        ActivationType::One | ActivationType::Two(None) | ActivationType::Five(None) => {
            // nothing to do, values is already zero
        }
        ActivationType::Two(Some(ref values)) => {
            let src: &[u8; 6 * 4] = &values[..].try_into().map_err(|_| {
                assert_with_msg!(
                    "Expected activation type to have exactly {} bytes, but was {}",
                    6 * 4,
                    values.len()
                )
            })?;
            let (_one, two) = activ.values.split_at_mut(3 * 4);
            two.copy_from_slice(src);
        }
        ActivationType::Five(Some(ref values)) => {
            let src: &[u8; VALUES_SIZE] = &values[..].try_into().map_err(|_| {
                assert_with_msg!(
                    "Expected activation type to have exactly {} bytes, but was {}",
                    VALUES_SIZE,
                    values.len()
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
