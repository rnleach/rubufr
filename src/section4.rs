use crate::{
    bit_buffer::BitBuffer,
    read_1_octet_u8, read_3_octet_usize,
    section3::Descriptor,
    tables::{table_b, table_d},
    types::{BufrMessageBuilder, Element, Group, Replication, Structure, Value},
};
use std::{error::Error, io::Read};

fn read_element_descriptor(
    f: &mut BitBuffer,
    desc: &Descriptor,
) -> Result<Element, Box<dyn Error>> {
    let desc = table_b::TABLE_B.get(&desc.string_form() as &str).ok_or(std::io::Error::other("Invalid Table B Entry"))?;
    let bits = desc.width_bits;
    let name = desc.element_name;

    let value = match desc.units {
        "CCITT IA5" => Value::Str(f.read_text(bits)?),

        "Numeric" | "a" | "mon" | "d" | "h" | "min" | "s" => f
            .read_i64(bits, desc.reference_val)?
            .map(Value::Numeric)
            .unwrap_or(Value::Missing),

        "Code table" | "Flag table" => f.read_u64(bits)?.map(Value::Code).unwrap_or(Value::Missing),

        _ => f
            .read_f64(bits, desc.reference_val, desc.scale_val)?
            .map(Value::Float)
            .unwrap_or(Value::Missing),
    };

    Ok(Element::new(value, desc.units, name, desc.fxy))
}

fn read_replication_descriptor<'a>(
    f: &mut BitBuffer,
    desc: &Descriptor,
    iter: &mut dyn Iterator<Item = &'a Descriptor>,
) -> Result<Replication, Box<dyn Error>> {
    debug_assert_eq!(desc.f_value(), 1, "Not a replication descriptor, f={}", desc.f_value());

    let num_descriptors = desc.x_value();

    let mut num_repititions: usize = desc.y_value() as usize;

    if num_repititions == 0 {
        let reps = iter.next()
            .ok_or(std::io::Error::new(std::io::ErrorKind::InvalidData, "Incomplete Replcation Descriptor"))?;
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

        num_repititions = f.read_usize(bits)?
            .ok_or(std::io::Error::new(std::io::ErrorKind::InvalidData, "Incomplete Replcation Descriptor"))?;
    }

    let mut descriptors = Vec::with_capacity(num_descriptors as usize);

    for _ in 0..num_descriptors {
        match iter.next() {
            Some(desc) => descriptors.push(desc),
            None => panic!("Ran out of descriptors in replication!"),
        }
    }
    let descriptors = descriptors;

    let mut rep = Replication::new_with_capacity(num_repititions);

    for _ in 0..num_repititions {
        for desc in &descriptors {
            let structure = match desc.f_value() {
                0 => Structure::Element(read_element_descriptor(f, &desc)?),
                1 => Structure::Replication(read_replication_descriptor(f, &desc, iter)?),
                2 => panic!("Operator descriptors not supported at this time."),
                3 => Structure::Group(read_sequence_descriptor(f, &desc)?),
                _ => panic!("Unknown descriptor type."),
            };

            rep.push(structure);
        }
    }

    Ok(rep)
}

fn read_sequence_descriptor<'a>(
    f: &mut BitBuffer,
    desc: &Descriptor,
) -> Result<Group, Box<dyn Error>> {
    let entry = table_d::TABLE_D.get(&desc.string_form() as &str).ok_or(std::io::Error::other("Invalid Table D Entry"))?;
    let sequence: Vec<Descriptor> = entry
        .elements
        .iter()
        .map(|d| Descriptor::from_string_form(d))
        .collect::<Result<_,_>>()?;

    let mut group = Group::new_with_capacity(sequence.len(), entry.group_name, entry.fxy);
    let mut desc_iter = sequence.iter();

    loop {
        let desc = match desc_iter.next() {
            Some(desc) => desc,
            None => break,
        };

        let structure = match desc.f_value() {
            0 => Structure::Element(read_element_descriptor(f, &desc)?),
            1 => Structure::Replication(read_replication_descriptor(f, &desc, &mut desc_iter)?),
            2 => panic!("Operator descriptors not supported at this time."),
            3 => Structure::Group(read_sequence_descriptor(f, &desc)?),
            _ => panic!("Unknown descriptor type."),
        };

        group.push(structure);
    }
    Ok(group)
}

pub(super) fn read_section_4(
    mut f: impl Read,
    descriptors: Vec<Descriptor>,
    builder: &mut BufrMessageBuilder,
) -> Result<(), Box<dyn Error>> {
    let mut octets_read: usize = 0;

    let section_size = read_3_octet_usize(&mut f)?
        .ok_or(std::io::Error::new(std::io::ErrorKind::InvalidData, "Section Size Required"))?;
    octets_read += 3;

    let _reserved: () = read_1_octet_u8(&mut f)?
        .ok_or(std::io::Error::new(std::io::ErrorKind::InvalidData, "Reserved Octet Required"))
        .and_then(|val| {
            match val {
                0 => Ok(()),
                _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Reserved Octet Required To Be 0")),
            }
        })?;
    octets_read += 1;

    assert!(!descriptors.is_empty());

    let mut desc_iter = descriptors.iter();

    let bytes_left_in_section = section_size - octets_read;
    let mut bit_buffer = BitBuffer::new(&mut f, bytes_left_in_section)?;

    let mut vec = vec![];

    loop {
        let descriptor = match desc_iter.next() {
            Some(desc) => desc,
            None => break,
        };

        let structure = match descriptor.f_value() {
            0 => Structure::Element(read_element_descriptor(&mut bit_buffer, descriptor)?),
            1 => Structure::Replication(read_replication_descriptor(
                &mut bit_buffer,
                descriptor,
                &mut desc_iter,
            )?),
            2 => panic!("Operator descriptors not supported at this time."),
            3 => Structure::Group(read_sequence_descriptor(&mut bit_buffer, descriptor)?),
            _ => panic!("Unknown descriptor type."),
        };

        vec.push(structure);
    }

    builder.elements(vec);

    octets_read += bytes_left_in_section;

    // Push the stream ahead to the end of the section
    for _ in 0..(section_size - octets_read) {
        let _v = read_1_octet_u8(&mut f)?;
    }

    Ok(())
}
