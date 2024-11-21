use eyre::{bail, Result};
use mech3ax_common::assert_that;
use mech3ax_common::io_ext::CountingReader;
use std::io::{Read, Seek, SeekFrom};

const RIFF_CHUNK_ID: &[u8; 4] = b"RIFF";
const FMT_CHUNK_ID: &[u8; 4] = b"fmt ";
const DATA_CHUNK_ID: &[u8; 4] = b"data";
const FORM_TYPE_WAVE: &[u8; 4] = b"WAVE";
const WAVE_FORMAT_PCM: u16 = 1;

pub struct WaveFile {
    pub channels: i32,
    pub frequency: i32,
    pub samples: Vec<f32>,
}

fn read_chunk_header(read: &mut CountingReader<impl Read>) -> Result<([u8; 4], u32)> {
    let mut chunk_id = [0u8; 4];
    read.read_exact(&mut chunk_id)?;
    let chunk_size = read.read_u32()?;
    Ok((chunk_id, chunk_size))
}

fn read_riff_chunk(read: &mut CountingReader<impl Read>) -> Result<()> {
    let (chunk_id, _chunk_size) = read_chunk_header(read)?;
    assert_that!("RIFF chunk ID", chunk_id == *RIFF_CHUNK_ID, read.offset - 8)?;
    // TODO: validate chunk_size against the length of data - 8?

    let mut form_type = [0u8; 4];
    read.read_exact(&mut form_type)?;
    assert_that!("RIFF form type", form_type == *FORM_TYPE_WAVE, read.prev)?;
    Ok(())
}

struct Format {
    bits_per_sample: u32,
    channels: i32,
    frequency: i32,
}

fn read_fmt_chunk(read: &mut CountingReader<impl Read>, chunk_size: u32) -> Result<Format> {
    let format_tag = read.read_u16()?;
    assert_that!("format tag", format_tag == WAVE_FORMAT_PCM, read.prev)?;
    // this is only valid for PCM files
    assert_that!("format chunk size", chunk_size in [16, 18], read.prev - 4)?;
    // Cast safety: i32 > u16
    let channels = read.read_u16()? as i32;
    // aka. SamplesPerSec, sample rate, sampling rate. Unity calls this frequency
    // this should be an unsigned, but Unity doesn't support that
    let frequency = read.read_i32()?;
    // PCM: Channels * bitsPerSecond * (bitsPerSample / 8)
    let _avg_bytes_per_sec = read.read_u32()?;
    // PCM: Channels * (bitsPerSample / 8)
    let _block_align = read.read_u16()?;
    // aka. sample size
    let bits_per_sample = read.read_u16()?.into();
    if chunk_size != 16 {
        let _extra_param_size = read.read_u16()?;
    }
    Ok(Format {
        channels,
        frequency,
        bits_per_sample,
    })
}

fn read_samples_8bit(
    read: &mut CountingReader<impl Read>,
    sample_count: usize,
    format: Format,
) -> Result<WaveFile> {
    // 8 bit WAV files are unsigned, from 0 to 255 (mid is 128).
    // We want to shift the values so that the midpoint is 0, which makes the
    // minimum -128 and maximum 127. To avoid clipping, divide by 128.
    let max_value = (i8::MAX as f32) + 1.0;
    let mut data = vec![0u8; sample_count];
    read.read_exact(&mut data)?;
    let samples = data
        .into_iter()
        .map(|sample| ((sample as f32) - max_value) / max_value)
        .collect();
    Ok(WaveFile {
        channels: format.channels,
        frequency: format.frequency,
        samples,
    })
}

fn read_samples_16bit(
    read: &mut CountingReader<impl Read>,
    sample_count: usize,
    format: Format,
) -> Result<WaveFile> {
    // 16 bit WAV files are signed, from -32768 to 32767 (mid is 0).
    // This means no shifting of the values is needed. To avoid clipping,
    // divide by 32768.
    let max_value = (i16::MAX as f32) + 1.0;
    let mut data = vec![0i16; sample_count];
    {
        // TODO: likely unsafe/UB
        let len = data.len() * std::mem::size_of::<i16>();
        let buf = unsafe { std::slice::from_raw_parts_mut(data.as_mut_ptr() as _, len) };
        read.read_exact(buf)?;
    }
    let samples = data
        .into_iter()
        .map(|sample| (sample as f32) / max_value)
        .collect();
    Ok(WaveFile {
        channels: format.channels,
        frequency: format.frequency,
        samples,
    })
}

fn read_data_chunk(
    read: &mut CountingReader<impl Read>,
    chunk_size: u32,
    format: Format,
) -> Result<WaveFile> {
    // It's possible to have bps values not integer multiples of 8,
    // e.g. 12 bits is specifically mentioned in the spec. This logic
    // rounds up to the nearest 8 bits; however scaling non-integer
    // multiples isn't implemented, and will throw an error.
    let bytes_per_sample = (format.bits_per_sample + 7) / 8;
    // Unity seems to want samples, i.e. channels are left interleaved.
    let sample_count = (chunk_size / bytes_per_sample) as _;
    match format.bits_per_sample {
        8 => read_samples_8bit(read, sample_count, format),
        16 => read_samples_16bit(read, sample_count, format),
        bps => bail!("Unsupported bit depth {}", bps),
    }
}

fn read_wav_file(read: &mut CountingReader<impl Read + Seek>) -> Result<WaveFile> {
    // the RIFF chunk must be first
    read_riff_chunk(read)?;

    let mut fmt = None;
    loop {
        let (chunk_id, chunk_size) = read_chunk_header(read)?;
        match &chunk_id {
            FMT_CHUNK_ID => {
                fmt = Some(read_fmt_chunk(read, chunk_size)?);
            }
            DATA_CHUNK_ID => match fmt {
                Some(format) => return read_data_chunk(read, chunk_size, format),
                None => bail!("Data chunk before format chunk (at {})", read.offset),
            },
            _ => {
                // skip unknown chunks
                // if the chunk is an odd size, it has a pad byte
                let skip = if (chunk_size & 1) == 1 {
                    chunk_size + 1
                } else {
                    chunk_size
                };
                // Cast safety: i64 > u32
                read.seek(SeekFrom::Current(skip as i64))?;
            }
        }
    }
}

impl WaveFile {
    pub fn new(read: &mut CountingReader<impl Read + Seek>) -> Result<WaveFile> {
        read_wav_file(read)
    }
}
