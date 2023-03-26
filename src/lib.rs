use std::{
    error::Error,
    io::{Read, Seek},
};

mod bit_buffer;
mod section0;
mod section1;
mod section2;
mod section3;
mod section4;
mod section5;
mod tables;
mod types;

pub use types::BufrMessage;

pub const MAX_BUFR_TABLE_VERSION_SUPPORTED: u8 = 39;

pub fn read_bufr_message(mut f: impl Read) -> Result<BufrMessage, Box<dyn Error>> {
    let mut builder = types::BufrMessageBuilder::new();

    section0::read_section_0(&mut f, &mut builder)?;
    let section_2_exists = section1::read_section_1(&mut f, &mut builder)?;

    if section_2_exists {
        section2::read_section_2(&mut f, &mut builder)?;
    }

    let descriptors = section3::read_section_3(&mut f, &mut builder)?;
    section4::read_section_4(&mut f, descriptors, &mut builder)?;
    section5::read_section_5(&mut f)?;

    Ok(builder.build())
}

pub fn scan_to_bufr_start(mut f: impl Seek + Read) -> Result<(), Box<dyn Error>> {
    let mut position: usize = 0;

    let mut buffer: [u8; 24] = [0; 24];
    loop {
        let num_read = f.read(&mut buffer)?;

        if num_read == 0 {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Not a bufr file",
            )));
        }

        let mut scan_start = 0;
        if buffer[0] == 'B' as u8
            && buffer[1] == 'U' as u8
            && buffer[2] == 'F' as u8
            && buffer[3] == 'R' as u8
        {
            f.seek(std::io::SeekFrom::Start(position as u64))?;
            return Ok(());
        } else if buffer[0] == 'B' as u8 {
            scan_start = 1;
            position += 1;
        }

        for i in scan_start..num_read {
            if buffer[i] == 'B' as u8 {
                f.seek(std::io::SeekFrom::Start(position as u64))?;
                break;
            }
            position += 1;
        }
    }
}

fn read_1_octet_u8(mut f: impl Read) -> Result<u8, Box<dyn Error>> {
    let mut value: [u8; 1] = [0; 1];
    f.read_exact(&mut value)?;

    Ok(value[0])
}

fn read_2_octet_u16(mut f: impl Read) -> Result<u16, Box<dyn Error>> {
    let mut value: [u8; 2] = [0; 2];
    f.read_exact(&mut value)?;
    let value = u16::from_be_bytes(value);

    Ok(value)
}

fn read_3_octet_usize(mut f: impl Read) -> Result<usize, Box<dyn Error>> {
    let mut message_size: [u8; 3] = [0; 3];
    f.read_exact(&mut message_size)?;
    let message_size: [u8; 8] = [
        0,
        0,
        0,
        0,
        0,
        message_size[0],
        message_size[1],
        message_size[2],
    ];
    let message_size: u64 = u64::from_be_bytes(message_size);
    let message_size: usize = usize::try_from(message_size)?;

    Ok(message_size)
}
