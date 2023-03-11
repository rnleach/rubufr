use super::{read_1_octet_u8, read_3_octet_usize};
use crate::{
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

const BUF_SIZE: usize = (table_b::MAX_BIT_WIDTH / 8) + 1;

struct BitBuffer<'b> {
    // The source
    reader: &'b mut dyn Read,

    // Track where we are in the source.
    max_bytes_to_read: usize,
    bytes_read: usize,

    // The buffer
    buffer: [u8; BUF_SIZE],

    // Track where we are in the buffer
    byte_position: usize,
    bit_position: usize,
    buffer_len: usize,
}

impl<'b> BitBuffer<'b> {
    fn new(reader: &'b mut dyn Read, max_bytes_to_read: usize) -> Self {
        BitBuffer {
            reader,
            max_bytes_to_read,
            bytes_read: 0,
            buffer: [0; BUF_SIZE],
            byte_position: 0,
            bit_position: 0,
            buffer_len: 0,
        }
    }

    fn bytes_read(&self) -> usize {
        self.bytes_read
    }

    fn bits_remaining_in_buffer(&self) -> usize {
        (self.buffer_len - self.byte_position) * 8 - self.bit_position
    }

    fn refill_buffer(&mut self) -> Result<(), Box<dyn Error>> {
        if self.max_bytes_to_read - self.bytes_read >= BUF_SIZE {
            self.reader.read_exact(&mut self.buffer)?;
            self.buffer_len = BUF_SIZE;
            self.bytes_read += BUF_SIZE;
        } else {
            let num_bytes_remaining = self.max_bytes_to_read - self.bytes_read;
            let mut buf = &mut self.buffer[0..num_bytes_remaining];
            self.reader.read_exact(&mut buf)?;
            self.buffer_len = num_bytes_remaining;
            self.bytes_read += num_bytes_remaining;
        }

        self.byte_position = 0;
        self.bit_position = 0;

        Ok(())
    }

    fn next_byte(&mut self) -> Result<u8, Box<dyn Error>> {
        if self.bits_remaining_in_buffer() == 0 {
            self.refill_buffer()?;
        }

        Ok(self.buffer[self.byte_position])
    }

    fn read_u8(&mut self, bits: usize) -> Result<u8, Box<dyn Error>> {
        debug_assert!(bits <= 8, "bits too large {} > 8", bits);
        debug_assert!(bits > 0, "requested zero bits");

        //dbg!(bits);

        let mut val: u8 = 0;

        let bits_left_in_byte = 8 - self.bit_position;
        if bits_left_in_byte < bits {
            //dbg!("Not all my bits are in this byte.");
            // Not all my bits are in this byte

            let byte = self.next_byte()?;

            // Need to get the rightmost bits
            let mask = 0b1111_1111 >> (8 - bits_left_in_byte);
            val |= byte & mask;
            // Need to left shift by how much?
            let num_bits_in_next_byte = bits - bits_left_in_byte;
            val <<= num_bits_in_next_byte;

            // Move to the next byte in the buffer
            self.bit_position = 0;
            self.byte_position += 1;

            // Get the next byte
            let mut byte = self.next_byte()?;

            // Get the leftmost how many bits?
            byte >>= 8 - num_bits_in_next_byte;
            val |= byte;

            // Advance the bit buffer
            self.bit_position += num_bits_in_next_byte;
        } else {
            //dbg!("All my bits are in this byte.", self.bit_position);
            // All my bits are here
            let mut byte = self.next_byte()?;

            // Example - self.bit_position = 1
            //           bits = 5
            //           mask =    0b0111_1100
            //
            //           num ->    0b0001_1111
            //           offset -> 0b0111_1100
            //           byte ->   0b0001_1111

            // Build the mask
            // Get a mask the right size first
            let mut mask = 0b1111_1111 >> (8 - bits);
            // Now give it the correct offset
            let offset = 8 - self.bit_position - bits;
            mask <<= offset;
            byte &= mask;
            // undo offset
            byte >>= offset;
            val = byte;

            // Addvance the bit buffer
            self.bit_position += bits;
            if self.bit_position == 8 {
                self.bit_position = 0;
                self.byte_position += 1;
            }
            debug_assert!(self.bit_position < 8, "self.bit_postion = {}", self.bit_position);
        }

        debug_assert!((val as u16) < (1u16 << bits), "value too big: {} >= {}", val, 1u16 << bits);
        Ok(val)
    }

    fn read_text(&mut self, bits: usize) -> Result<String, Box<dyn Error>> {
        debug_assert_eq!(bits % 8, 0, "funky string size");

        let num_chars = bits / 8;
        //dbg!(num_chars, bits);
        let mut buf: Vec<u8> = Vec::with_capacity(num_chars);
        for _ in 0..num_chars {
            let c = self.read_u8(8)?;
            //dbg!(c);
            buf.push(c);
        }

        Ok(String::from_utf8(buf)?)
    }

    fn read_u64(&mut self, bits: usize) -> Result<u64, Box<dyn Error>> {
        debug_assert!(bits <= (8 * 8), "too many bits for u64: {}", bits);
        debug_assert!(bits > 0, "requested zero bits");

        let mut vals_buf: [u8; 8] = [0; 8];

        let mut num_bytes = bits / 8;
        if bits % 8 > 0 {
            num_bytes += 1;
        }

        //dbg!(num_bytes);
        if bits % 8 > 0 {
            vals_buf[8 - num_bytes] = self.read_u8(bits % 8)?;
            //dbg!(vals_buf[8 - num_bytes]);
        } else {
            vals_buf[8 - num_bytes] = self.read_u8(8)?;
            //dbg!(vals_buf[8 - num_bytes]);
        }

        //dbg!(num_bytes);
        for i in (1..num_bytes).rev() {
            vals_buf[8 - i] = self.read_u8(8)?;
            //dbg!(i, vals_buf[8-i]);
        }

        //dbg!(vals_buf);

        let val = u64::from_be_bytes(vals_buf);
        debug_assert!(val <  (1u64 << bits), "val too large: {} >= {}", val, 1u64 << bits);

        Ok(val)
    }

    fn read_usize(&mut self, bits: usize) -> Result<usize, Box<dyn Error>> {
        let val = self.read_u64(bits)?;

        Ok(usize::try_from(val)?)
    }

    fn read_i64(&mut self, bits: usize, reference_val: i64) -> Result<i64, Box<dyn Error>> {
        let val = self.read_u64(bits)?;

        let val = i64::try_from(val)? + reference_val;
        Ok(val)
    }

    fn read_f64(
        &mut self,
        bits: usize,
        reference_val: i64,
        scale: i32,
    ) -> Result<f64, Box<dyn Error>> {
        let mut val = self.read_i64(bits, reference_val)? as f64;

        if scale != 0 {
            val /= f64::powi(10.0, scale);
        }

        Ok(val)
    }
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
                vec.push(val);
            } else {
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

    debug_assert_eq!(desc.f_value(), 1, "Not a replication descriptor, f={}", desc.f_value());

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
            _ => panic!("unimplemented replication descriptor: {}", reps.string_form()),
        };

        num_repititions = f.read_usize(bits)?;
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
            0 => read_element_descriptor(&mut bit_buffer, descriptor, &mut float_vals, &mut float_arrays, false)?,
            1 => read_replication_descriptor(&mut bit_buffer, descriptor, &mut desc_iter, &mut float_vals, &mut float_arrays)?,
            2 => panic!("Operator descriptors not supported at this time."),
            3 => read_sequence_descriptor(&mut bit_buffer, descriptor, &mut float_vals, &mut float_arrays, false)?,
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
