use crate::{event_type, EventHeaderC, EventType};
use mech3ax_api_types::anim::events::{EventStart, StartOffset};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, assert_with_msg, Result};
use mech3ax_types::AsBytes as _;
use std::io::{Read, Write};

// pub(crate) fn read_event_header(
//     read: &mut CountingReader<impl Read>,
// ) -> Result<(EventType, u32, Option<EventStart>)> {
//     let header: EventHeaderC = read.read_struct()?;

//     let event_type =
//         assert_that!("event header type", enum EventType => header.event_type, read.prev + 0)?;
//     let start_offset =
//         assert_that!("event start offset", enum StartOffset => header.start_offset, read.prev + 1)?;
//     assert_that!("event header field 02", header.pad == 0, read.prev + 2)?;

//     let data_size = header.size.checked_sub(EventHeaderC::SIZE).ok_or_else(|| {
//         assert_with_msg!(
//             "Expected event size > {}, but was {} (at {})",
//             EventHeaderC::SIZE,
//             header.size,
//             read.prev + 4,
//         )
//     })?;

//     let start = if start_offset == StartOffset::Animation && header.start_time == 0.0 {
//         None
//     } else {
//         Some(EventStart {
//             offset: start_offset,
//             time: header.start_time,
//         })
//     };

//     Ok((event_type, data_size, start))
// }
