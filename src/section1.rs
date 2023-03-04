use super::{read_1_octet_u8, read_2_octet_u16, read_3_octet_usize};
use std::{error::Error, fmt::Display, io::Read};

pub struct Section1 {
    section_size: usize,
    master_table: u8,
    originating_center: u16,
    originating_subcenter: u16,
    update_num: u8,
    section_2_present: bool,
    data_category: u8,
    data_subcategory: u8,
    local_data_subcategory: u8,
    bufr_master_table_version: u8,
    local_tables_version: u8,
    year: u16,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    extra_data: Vec<u8>,
}

impl Section1 {
    pub fn section_2_exists(&self) -> bool {
        self.section_2_present
    }
}

impl Display for Section1 {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        writeln!(f, "           Section Length: {}", self.section_size)?;
        writeln!(f, "        Master Table Code: {}", self.master_table)?;
        writeln!(f, "       Originating Center: {}", self.originating_center)?;
        writeln!(f, "    Originating Subcenter: {}", self.originating_subcenter)?;
        writeln!(f, "            Update Number: {}", self.update_num)?;
        writeln!(f, "        Section 2 Present: {}", self.section_2_present)?;
        writeln!(f, "            Data Category: {}", self.data_category)?;
        writeln!(f, "         Data Subcategory: {}", self.data_subcategory)?;
        writeln!(f, "   Local Data Subcategory: {}", self.local_data_subcategory)?;
        writeln!(f, "BUFR Master Table Version: {}", self.bufr_master_table_version)?;
        writeln!(f, "     Local Tables Version: {}", self.local_tables_version)?;
        writeln!(f, "                     Year: {}", self.year)?;
        writeln!(f, "                    Month: {}", self.month)?;
        writeln!(f, "                      Day: {}", self.day)?;
        writeln!(f, "                     Hour: {}", self.hour)?;
        writeln!(f, "                   Minute: {}", self.minute)?;
        writeln!(f, "                   Second: {}", self.second)?;

        let extra_data = !self.extra_data.is_empty();
        writeln!(f, "               Extra Data: {}", extra_data)?;

        if extra_data {
            writeln!(f, "     Amount Of Extra Data: {}", self.extra_data.len())?;
        }

        Ok(())
    }
}

#[rustfmt::skip]
pub(super) fn read_section_1(mut f: impl Read) -> Result<Section1, Box<dyn Error>> {
    let section_size = read_3_octet_usize(&mut f)?;                                 // octets 1-3
    let master_table = read_1_octet_u8(&mut f)?;                                    // octet 4
    let originating_center = read_2_octet_u16(&mut f)?;                             // octets 5-6
    let originating_subcenter = read_2_octet_u16(&mut f)?;                          // octets 7-8
    let update_num = read_1_octet_u8(&mut f)?;                                      // octet 9
    let section_2_present = read_1_octet_u8(&mut f)? > 0;                           // octet 10
    let data_category = read_1_octet_u8(&mut f)?;                                   // octet 11
    let data_subcategory = read_1_octet_u8(&mut f)?;                                // octet 12
    let local_data_subcategory = read_1_octet_u8(&mut f)?;                          // octet 13
    let bufr_master_table_version = read_1_octet_u8(&mut f)?;                       // octet 14
    let local_tables_version = read_1_octet_u8(&mut f)?;                            // octet 15
    let year = read_2_octet_u16(&mut f)?;                                           // octets 16-17
    let month = read_1_octet_u8(&mut f)?;                                           // octet 18
    let day = read_1_octet_u8(&mut f)?;                                             // octet 19
    let hour = read_1_octet_u8(&mut f)?;                                            // octet 20
    let minute = read_1_octet_u8(&mut f)?;                                          // octet 21
    let second = read_1_octet_u8(&mut f)?;                                          // octet 22

    let mut extra_data = vec![];
    f.take(section_size as u64 - 22).read_to_end(&mut extra_data)?;

    Ok(Section1 {
        section_size,
        master_table,
        originating_center,
        originating_subcenter,
        update_num,
        section_2_present,
        data_category,
        data_subcategory,
        local_data_subcategory,
        bufr_master_table_version,
        local_tables_version,
        year,
        month,
        day,
        hour,
        minute,
        second,
        extra_data,
    })
}
