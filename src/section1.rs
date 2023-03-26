use crate::{read_1_octet_u8, read_2_octet_u16, read_3_octet_usize, types::BufrMessageBuilder};
use std::{error::Error, io::Read};

#[rustfmt::skip]
pub(super) fn read_section_1(mut f: impl Read, builder: &mut BufrMessageBuilder) -> Result<bool, Box<dyn Error>> {
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

    builder.master_table(master_table)
        .originating_center(originating_center)
        .originating_subcenter(originating_subcenter)
        .update_num(update_num)
        .data_category(data_category)
        .data_subcategory(data_subcategory)
        .local_data_subcategory(local_data_subcategory)
        // TODO Check the master tables version is 0 or 10.
        .bufr_master_table_version(bufr_master_table_version)
        .local_tables_version(local_tables_version)
        .year(year)
        .month(month)
        .day(day)
        .hour(hour)
        .minute(minute)
        .second(second)
        .extra_seciont_1_data(extra_data);

    Ok(section_2_present)
}
