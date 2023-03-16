use crate::{read_1_octet_u8, read_2_octet_u16, read_3_octet_usize, types::BufrMessageBuilder};
use std::{error::Error, fmt::Display, io::Read};

pub struct Section3 {
    section_size: usize,
    num_datasets: u16,
    observed_data: bool,
    compressed_data: bool,
    descriptors: Vec<Descriptor>,
}

impl Display for Section3 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        writeln!(f, "        Section Size: {}", self.section_size)?;
        writeln!(f, "  Number of Datasets: {}", self.num_datasets)?;
        writeln!(f, "       Observed Data: {}", self.observed_data)?;
        writeln!(f, "     Compressed Data: {}", self.compressed_data)?;

        writeln!(f, "         Descriptors:")?;
        for desc in &self.descriptors {
            write!(f, "{}", desc)?;
        }
        writeln!(f)?;

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Hash)]
pub struct Descriptor {
    f: u8,
    x: u8,
    y: u8,
}

#[rustfmt::skip]
impl Display for Descriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {

        match self.f {
            0 => writeln!(f, "Element Descriptor (Table B): Class = {} Entry = {}", self.x, self.y)?,
            1 => writeln!(f, "Replication Descriptor: Operator = {} Number = {}", self.x, self.y)?,
            2 => writeln!(f, "Operator Descriptor (Table C): Operation = {} Y = {}", self.x, self.y)?,
            3 => writeln!(f, "Sequence Descriptor (Table D): X = {} Y = {}", self.x, self.y)?,
            x => panic!("NOT A VALID DESCRIPTOR TYPE {}", x),
        }

        Ok(())
    }
}

impl Descriptor {
    pub fn string_form(&self) -> String {
        format!("{:01}{:02}{:03}", self.f, self.x, self.y)
    }

    pub fn from_string_form(s: &str) -> Self {
        let fs = &s[0..1];
        let xs = &s[1..3];
        let ys = &s[3..];

        let f = u8::from_str_radix(fs, 10).unwrap();
        let x = u8::from_str_radix(xs, 10).unwrap();
        let y = u8::from_str_radix(ys, 10).unwrap();

        Descriptor { f, x, y }
    }

    pub fn decode_binary_descriptor(desc: u16) -> Self {
        let fx: u8 = (desc >> 8) as u8;
        let f = (fx & 0b1100_0000u8) >> 6;
        let x = fx & 0b0011_1111u8;
        let y = (desc & 0b0000_0000_1111_1111u16) as u8;

        Descriptor { f, x, y }
    }

    pub fn f_value(&self) -> u8 {
        self.f
    }

    pub fn x_value(&self) -> u8 {
        self.x
    }

    pub fn y_value(&self) -> u8 {
        self.y
    }
}

#[rustfmt::skip]
pub(super) fn read_section_3(mut f: impl Read, builder: &mut BufrMessageBuilder) -> Result<Vec<Descriptor>, Box<dyn Error>> {

    let mut octets_read: usize = 0;

    let section_size = read_3_octet_usize(&mut f)?;
    octets_read += 3;

    // TODO: Check if this is zero and return an error if it isn't
    let reserved = read_1_octet_u8(&mut f)?;
    octets_read += 1;
    debug_assert_eq!(reserved, 0, "Section 3: Octet 4 must be zero.");

    let num_datasets = read_2_octet_u16(&mut f)?;
    builder.num_datasets(num_datasets);
    octets_read += 2;

    let d_flags = read_1_octet_u8(&mut f)?;
    octets_read += 1;

    let observed_data = (d_flags & 0b1000_0000u8) > 0;
    let compressed_data = (d_flags & 0b0100_0000u8) > 0;
    let bits_3_to_8 = d_flags & 0b0011_1111u8;
    // TODO: Check if this is zero and return an error if it isn't
    debug_assert_eq!(bits_3_to_8, 0, "Section 3: Bits 3-8 nonzero in octet 7.");

    // FIXME: Learn to handle compressed data
    assert!(!compressed_data, "Compressed data handling not implemented.");

    builder.observed_data(observed_data).compressed_data(compressed_data);

    let num_descriptors = (section_size - 7) / 2;

    let mut descriptors = Vec::with_capacity(num_descriptors);
    for _ in 0..num_descriptors {
        let desc = read_2_octet_u16(&mut f)?;
        octets_read += 2;
        descriptors.push(Descriptor::decode_binary_descriptor(desc));
    }

    // Push the stream ahead to the end of the section
    for _ in 0..(section_size - octets_read) {
        let _v = read_1_octet_u8(&mut f)?;
    }

    Ok(descriptors)
}
