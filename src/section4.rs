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
    let desc = table_b::TABLE_B.get(&desc.string_form() as &str).unwrap();
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

        num_repititions = f.read_usize(bits)?.expect("Missing number of reps!?");
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
    let entry = table_d::TABLE_D.get(&desc.string_form() as &str).unwrap();
    let sequence: Vec<_> = entry
        .elements
        .iter()
        .map(|d| Descriptor::from_string_form(d))
        .collect();

    let mut desc_iter = sequence.iter();

    let mut group = Group::new_with_capacity(sequence.len(), entry.group_name, entry.fxy);

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

    // TODO: If this shows up as missing, return an error.
    let section_size = read_3_octet_usize(&mut f)?.unwrap();
    octets_read += 3;

    // TODO: If this shows up as missing, return an error.
    // TODO: Check if this is zero and return an error if it isn't
    let reserved = read_1_octet_u8(&mut f)?.unwrap();
    octets_read += 1;
    debug_assert_eq!(reserved, 0, "Section 4: Octet 4 must be zero.");

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
