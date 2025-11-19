use quick_xml::{events::Event, reader::Reader};
use std::{
    collections::HashMap,
    error::Error,
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter, Write},
};

fn main() -> Result<(), Box<dyn Error>> {
    make_table_b()?;
    make_table_d()?;

    Ok(())
}

const TABLE_B_INPUT: &str = "Tables/BUFRCREX_TableB_en.xml";
const TABLE_B_OUTPUT: &str = "src/tables/table_b.rs";

fn make_table_b() -> Result<(), Box<dyn Error>> {
    let mut table_b = HashMap::new();

    let mut reader = Reader::from_reader(BufReader::new(File::open(TABLE_B_INPUT)?));
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut txt = String::new();

    let mut element_name = String::new();
    let mut fxy = String::new();
    let mut units = String::new();
    let mut scale: i32 = 0;
    let mut reference_value: i32 = 0;
    let mut width_bits = 8;

    let mut max_width_bits = width_bits;

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Eof => break,

            Event::Start(e) => match e.name().as_ref() {
                b"BUFRCREX_TableB_en" => {
                    element_name.clear();
                    fxy.clear();
                    units.clear();
                    scale = 0;
                    reference_value = 0;
                    width_bits = 0;
                }
                _ => {
                    txt.clear();
                }
            },
            Event::End(e) => match e.name().as_ref() {
                b"BUFRCREX_TableB_en" => {
                    let key = fxy.clone();
                    let element_name = element_name.clone();
                    let units = units.clone();
                    table_b.insert(
                        key,
                        (element_name, units, scale, reference_value, width_bits),
                    );
                }
                b"FXY" => {
                    fxy.push_str(&txt);
                }
                b"ElementName_en" => {
                    element_name.push_str(&txt);
                }
                b"BUFR_Unit" => {
                    units.push_str(&txt);
                }
                b"BUFR_Scale" => {
                    scale = txt.parse::<i32>()?;
                }
                b"BUFR_ReferenceValue" => {
                    reference_value = txt.parse::<i32>()?;
                }
                b"BUFR_DataWidth_Bits" => {
                    width_bits = txt.parse::<usize>()?;
                    max_width_bits = max_width_bits.max(width_bits);
                }
                _ => {}
            },
            Event::Text(e) => txt.push_str(&e.decode().unwrap()),

            // There are several other `Event`s we do not consider here
            _ => {}
        }
        // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
        buf.clear();
    }

    let mut w = BufWriter::new(
        OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(TABLE_B_OUTPUT)?,
    );

    // Output the rust hashmap
    writeln!(w, "use lazy_static::lazy_static;")?;
    writeln!(w, "use std::collections::HashMap;")?;
    writeln!(w, "use super::TableBEntry;")?;
    writeln!(w)?;
    writeln!(w, "pub const MAX_BIT_WIDTH: usize = {};", max_width_bits)?;
    writeln!(w)?;
    writeln!(w, "lazy_static! {{")?;
    writeln!(w, "    pub static ref TABLE_B: HashMap<&'static str, TableBEntry> = [")?;

    for (key, value) in table_b.into_iter() {
        writeln!(
            w,
            r##"("{}", TableBEntry{{fxy:"{}", width_bits:{}, element_name:r#"{}"#, units:r#"{}"#, reference_val: {}, scale_val: {}}}),"##,
            key, key, value.4, value.0, value.1, value.3, value.2
        )?;
    }

    // Close out the hash table
    writeln!(w, "        ].into_iter().collect();")?;
    writeln!(w, "}}")?;

    Ok(())
}

const TABLE_D_INPUT: &str = "Tables/BUFR_TableD_en.xml";
const TABLE_D_OUTPUT: &str = "src/tables/table_d.rs";

fn make_table_d() -> Result<(), Box<dyn Error>> {
    let mut table_d = HashMap::new();

    let mut reader = Reader::from_reader(BufReader::new(File::open(TABLE_D_INPUT)?));
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut txt = String::new();

    let mut group_name = String::new();
    let mut fxy1 = String::new();
    let mut fxy2 = String::new();

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Eof => break,

            Event::Start(e) => match e.name().as_ref() {
                b"BUFR_TableD_en" => {
                    fxy1.clear();
                    fxy2.clear();
                    group_name.clear();
                }
                _ => {
                    txt.clear();
                }
            },
            Event::End(e) => match e.name().as_ref() {
                b"BUFR_TableD_en" => {
                    let key = fxy1.clone();
                    let fxy2 = fxy2.clone();
                    let group_name = group_name.clone();
                    let (_name, vals) = table_d.entry(key).or_insert((group_name, vec![]));
                    vals.push(fxy2);
                }
                b"FXY1" => {
                    fxy1.push_str(&txt);
                }
                b"FXY2" => {
                    fxy2.push_str(&txt);
                }
                b"Title_en" => {
                    group_name.push_str(&txt);
                }
                _ => {}
            },
            Event::Text(e) => txt.push_str(&e.decode().unwrap()),

            // There are several other `Event`s we do not consider here
            _ => {}
        }
        // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
        buf.clear();
    }

    let mut w = BufWriter::new(
        OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(TABLE_D_OUTPUT)?,
    );

    // Output the rust hashmap
    writeln!(w, "use lazy_static::lazy_static;")?;
    writeln!(w, "use std::collections::HashMap;")?;
    writeln!(w, "use super::TableDEntry;")?;
    writeln!(w)?;
    writeln!(w, "lazy_static! {{")?;
    writeln!(
        w,
        "    pub static ref TABLE_D: HashMap<&'static str, TableDEntry> = ["
    )?;

    for (key, (name, fxy2s)) in table_d {
        write!(
            w,
            r##"("{}", TableDEntry{{fxy:"{}", group_name:r#"{}"#, elements:vec![r#"{}"#"##,
            key, key, name, fxy2s[0]
        )?;
        for fxy2 in fxy2s.into_iter().skip(1) {
            write!(w, r##", r#"{}"#"##, fxy2)?;
        }
        writeln!(w, "]}}),")?;
    }

    // Close out the enum
    writeln!(w, "        ].into_iter().collect();")?;
    writeln!(w, "}}")?;

    Ok(())
}
