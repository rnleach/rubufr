use std::{
    error::Error,
    fmt::Display,
    io::{Read, Seek},
};

mod section0;
use section0::Section0;

mod section1;
use section1::Section1;

mod section2;
use section2::Section2;

mod section3;
use section3::Section3;

mod section4;
use section4::Section4;

mod section5;
use section5::Section5;

pub struct BufrMessage {
    section_0: Section0,
    section_1: Section1,
    section_2: Section2,
    section_3: Section3,
    section_4: Section4,
    section_5: Section5,
}

pub fn read_bufr_message(mut f: impl Read) -> Result<BufrMessage, Box<dyn Error>> {
    // Read section 0
    let section_0 = section0::read_section_0(&mut f)?;
    let section_1 = section1::read_section_1(&mut f)?;
    let section_2 = section2::read_section_2(&mut f, section_1.section_2_exists())?;
    let section_3 = section3::read_section_3(&mut f)?;
    let section_4 = section4::read_section_4(&mut f)?;
    let section_5 = section5::read_section_5(&mut f)?;

    Ok(BufrMessage {
        section_0,
        section_1,
        section_2,
        section_3,
        section_4,
        section_5,
    })
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

impl Display for BufrMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        writeln!(f, "Section 0:")?;
        writeln!(f, "{}", self.section_0)?;

        writeln!(f, "Section 1:")?;
        writeln!(f, "{}", self.section_1)?;

        writeln!(f, "Section 2:")?;
        writeln!(f, "{}", self.section_2)?;

        writeln!(f, "Section 3:")?;
        writeln!(f, "{}", self.section_3)?;

        writeln!(f, "Section 4:")?;
        writeln!(f, "{}", self.section_4)?;

        writeln!(f, "Section 5:")?;
        writeln!(f, "{}", self.section_5)?;

        Ok(())
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
