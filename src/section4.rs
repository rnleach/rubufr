use super::{read_1_octet_u8, read_3_octet_usize};
use crate::{
    bit_buffer::BitBuffer,
    section3::{Descriptor, Section3},
    table_b, table_d,
};
use std::{collections::HashMap, error::Error, fmt::Display, io::Read};

pub struct Section4 {
    section_size: usize,
    float_vals: HashMap<&'static str, f64>,
    float_arrays: HashMap<&'static str, Vec<f64>>,
}

impl Display for Section4 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        writeln!(f, "Section Size: {}", self.section_size)?;
        writeln!(f)?;

        writeln!(f, "Float Values:")?;
        for (k, v) in self.float_vals.iter() {
            writeln!(f, "    {} : {}", k, v)?;
        }
        writeln!(f)?;

        writeln!(f, "Float Arrays:")?;
        for (k, v) in self.float_arrays.iter() {
            writeln!(f, "    {}: {} values", k, v.len())?;
        }
        writeln!(f)?;
        Ok(())
    }
}

pub struct TableBEntry {
    pub(crate) width_bits: usize,
    pub(crate) element_name: &'static str,
    pub(crate) units: &'static str,
    pub(crate) reference_val: i64,
    pub(crate) scale_val: i32,
}

fn read_element_descriptor(
    f: &mut BitBuffer,
    desc: &Descriptor,
    float_vals: &mut HashMap<&'static str, f64>,
    float_arrays: &mut HashMap<&'static str, Vec<f64>>,
    is_array_val: bool,
) -> Result<(), Box<dyn Error>> {
    //dbg!("read_element_descriptor");

    let desc = table_b::TABLE_B.get(&desc.string_form() as &str).unwrap();
    let bits = desc.width_bits;
    let name = desc.element_name;
    //dbg!(name, desc.units, bits, desc.reference_val, desc.scale_val);

    match desc.units {
        "CCITT IA5" => {
            let _val = f.read_text(bits)?;
            //dbg!(val, "-+-+-+-+-+-+-+-+-+-+-+-+-+-");
        }
        "Numeric" => {
            let _val = f.read_i64(bits, desc.reference_val)?;
            //dbg!(val, "-*-*-*-*-*-*-*-*-*-*-*-*-*-");
        }
        "Code table" => {
            let _val = f.read_i64(bits, desc.reference_val)?;
            //dbg!(val, "-^-^-^-^-^-^-^-^-^-^-^-^-^-");
        }
        _ => {
            let val = f.read_f64(bits, desc.reference_val, desc.scale_val)?;
            //dbg!(val, "-x-x-x-x-x-x-x-x-x-x-x-x-x-");
            if is_array_val {
                let vec = float_arrays.entry(name).or_insert(vec![]);
                vec.push(val.unwrap_or(f64::NAN));
            } else {
                let val = val.unwrap_or(f64::NAN);
                let v = float_vals.insert(name, val);
                debug_assert!(v.is_none(), "overwriting float val {}", name);
            }
        }
    }

    Ok(())
}

fn read_replication_descriptor<'a>(
    f: &mut BitBuffer,
    desc: &Descriptor,
    iter: &mut dyn Iterator<Item = &'a Descriptor>,
    float_vals: &mut HashMap<&'static str, f64>,
    float_arrays: &mut HashMap<&'static str, Vec<f64>>,
) -> Result<(), Box<dyn Error>> {
    //dbg!("read_replication_descriptor");

    debug_assert_eq!(
        desc.f_value(),
        1,
        "Not a replication descriptor, f={}",
        desc.f_value()
    );

    let num_descriptors = desc.x_value();
    //dbg!(num_descriptors);

    let mut num_repititions: usize = desc.y_value() as usize;
    //dbg!(num_repititions);

    if num_repititions == 0 {
        let reps = iter.next().unwrap();
        debug_assert_eq!(reps.f_value(), 0);
        debug_assert_eq!(reps.x_value(), 31);
        let bits = match reps.y_value() {
            1 => 8,
            2 => 16,
            _ => panic!(
                "unimplemented replication descriptor: {}",
                reps.string_form()
            ),
        };

        num_repititions = f.read_usize(bits)?.unwrap();
    }

    let mut descriptors = Vec::with_capacity(num_descriptors as usize);

    for _ in 0..num_descriptors {
        match iter.next() {
            Some(desc) => descriptors.push(desc),
            None => panic!("Ran out of descriptors in replication!"),
        }
    }
    let descriptors = descriptors;

    for _ in 0..num_repititions {
        for desc in &descriptors {
            match desc.f_value() {
                0 => read_element_descriptor(f, &desc, float_vals, float_arrays, true)?,
                1 => read_replication_descriptor(f, &desc, iter, float_vals, float_arrays)?,
                2 => panic!("Operator descriptors not supported at this time."),
                3 => read_sequence_descriptor(f, &desc, float_vals, float_arrays, true)?,
                _ => panic!("Unknown descriptor type."),
            }
        }
    }

    Ok(())
}

fn read_sequence_descriptor<'a>(
    f: &mut BitBuffer,
    desc: &Descriptor,
    float_vals: &mut HashMap<&'static str, f64>,
    float_arrays: &mut HashMap<&'static str, Vec<f64>>,
    in_array: bool,
) -> Result<(), Box<dyn Error>> {
    let sequence = table_d::TABLE_D.get(&desc.string_form() as &str).unwrap();
    let sequence: Vec<_> = sequence
        .iter()
        .map(|d| Descriptor::from_string_form(d))
        .collect();

    let mut desc_iter = sequence.iter();

    loop {
        let desc = match desc_iter.next() {
            Some(desc) => desc,
            None => break,
        };

        //dbg!("read_sequence_descriptor loop", desc.string_form(), "-------------------");

        match desc.f_value() {
            0 => read_element_descriptor(f, &desc, float_vals, float_arrays, in_array)?,
            1 => read_replication_descriptor(f, &desc, &mut desc_iter, float_vals, float_arrays)?,
            2 => panic!("Operator descriptors not supported at this time."),
            3 => read_sequence_descriptor(f, &desc, float_vals, float_arrays, in_array)?,
            _ => panic!("Unknown descriptor type."),
        }
    }
    Ok(())
}

pub(super) fn read_section_4(
    mut f: impl Read,
    sec3: &Section3,
) -> Result<Section4, Box<dyn Error>> {
    let mut octets_read: usize = 0;
    let mut float_vals = HashMap::new();
    let mut float_arrays = HashMap::new();

    let section_size = read_3_octet_usize(&mut f)?;
    octets_read += 3;

    // TODO: Check if this is zero and return an error if it isn't
    let reserved = read_1_octet_u8(&mut f)?;
    octets_read += 1;
    debug_assert_eq!(reserved, 0, "Section 4: Octet 4 must be zero.");

    let descriptors = sec3.descriptors();
    assert!(!descriptors.is_empty());

    let mut desc_iter = descriptors.iter();

    let bytes_left_in_section = section_size - octets_read;
    let mut bit_buffer = BitBuffer::new(&mut f, bytes_left_in_section);

    loop {
        let descriptor = match desc_iter.next() {
            Some(desc) => desc,
            None => break,
        };
        //dbg!(descriptor.string_form());

        match descriptor.f_value() {
            0 => read_element_descriptor(
                &mut bit_buffer,
                descriptor,
                &mut float_vals,
                &mut float_arrays,
                false,
            )?,
            1 => read_replication_descriptor(
                &mut bit_buffer,
                descriptor,
                &mut desc_iter,
                &mut float_vals,
                &mut float_arrays,
            )?,
            2 => panic!("Operator descriptors not supported at this time."),
            3 => read_sequence_descriptor(
                &mut bit_buffer,
                descriptor,
                &mut float_vals,
                &mut float_arrays,
                false,
            )?,
            _ => panic!("Unknown descriptor type."),
        }
    }

    octets_read += bit_buffer.bytes_read();

    // Push the stream ahead to the end of the section
    //dbg!(section_size - octets_read);
    for _ in 0..(section_size - octets_read) {
        let _v = read_1_octet_u8(&mut f)?;
    }

    Ok(Section4 {
        section_size,
        float_vals,
        float_arrays,
    })
}
