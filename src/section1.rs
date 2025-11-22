use crate::{read_1_octet_u8, read_2_octet_u16, read_3_octet_usize, types::BufrMessageBuilder};
use std::{error::Error, io::Read};

#[rustfmt::skip]
pub(super) fn read_section_1(mut f: impl Read, builder: &mut BufrMessageBuilder) -> Result<bool, Box<dyn Error>> {
    let section_size = read_3_octet_usize(&mut f)?                                                           // octets 1-3
        .ok_or(std::io::Error::new(std::io::ErrorKind::InvalidData, "Section Size Required"))?;
    let master_table = read_1_octet_u8(&mut f)?                                                              // octet 4
        .ok_or(std::io::Error::new(std::io::ErrorKind::InvalidData, "Master Table Required"))?;
    let originating_center = read_2_octet_u16(&mut f)?                                                       // octets 5-6
        .ok_or(std::io::Error::new(std::io::ErrorKind::InvalidData, "Origination Center Required"))?;
    let originating_subcenter = read_2_octet_u16(&mut f)?                                                    // octets 7-8
        .ok_or(std::io::Error::new(std::io::ErrorKind::InvalidData, "Origination Subcenter Required"))?;
    let update_num = read_1_octet_u8(&mut f)?                                                                // octet 9
        .ok_or(std::io::Error::new(std::io::ErrorKind::InvalidData, "Update Number Required"))?;
    let section_2_present = read_1_octet_u8(&mut f)?                                                         // octet 10
        .ok_or(std::io::Error::new(std::io::ErrorKind::InvalidData, "Section 2 Present Flag Required"))? > 0;
    let data_category = read_1_octet_u8(&mut f)?                                                             // octet 11
        .ok_or(std::io::Error::new(std::io::ErrorKind::InvalidData, "Data Category Required"))?;
    let data_subcategory = read_1_octet_u8(&mut f)?                                                          // octet 12
        .ok_or(std::io::Error::new(std::io::ErrorKind::InvalidData, "Data Subcategory Required"))?;
    let local_data_subcategory = read_1_octet_u8(&mut f)?;                                                   // octet 13
    let bufr_master_table_version = read_1_octet_u8(&mut f)?                                                 // octet 14
        .ok_or(std::io::Error::new(std::io::ErrorKind::InvalidData, "Master Table Version Required"))?;
    let local_tables_version = read_1_octet_u8(&mut f)?                                                      // octet 15
        .ok_or(std::io::Error::new(std::io::ErrorKind::InvalidData, "Local Tables Version Required"))?;
    let year = read_2_octet_u16(&mut f)?                                                                     // octets 16-17
        .ok_or(std::io::Error::new(std::io::ErrorKind::InvalidData, "Year Not Allowed To Be Missing"))?;
    let month = read_1_octet_u8(&mut f)?                                                                     // octet 18
        .ok_or(std::io::Error::new(std::io::ErrorKind::InvalidData, "Month Not Allowed To Be Missing"))?;
    let day = read_1_octet_u8(&mut f)?                                                                       // octet 19
        .ok_or(std::io::Error::new(std::io::ErrorKind::InvalidData, "Day Not Allowed To Be Missing"))?;
    let hour = read_1_octet_u8(&mut f)?                                                                      // octet 20
        .ok_or(std::io::Error::new(std::io::ErrorKind::InvalidData, "Hour Not Allowed To Be Missing"))?;
    let minute = read_1_octet_u8(&mut f)?                                                                    // octet 21
        .ok_or(std::io::Error::new(std::io::ErrorKind::InvalidData, "Minute Not Allowed To Be Missing"))?;
    let second = read_1_octet_u8(&mut f)?                                                                    // octet 22
        .ok_or(std::io::Error::new(std::io::ErrorKind::InvalidData, "Second Not Allowed To Be Missing"))?;

    let mut extra_data = vec![];
    f.take(section_size as u64 - 22).read_to_end(&mut extra_data)?;

    if master_table != 0 && master_table != 10 {
        return Err(Box::new(std::io::Error::other("Non-meteorological / non-oceanographic data!")));
    }

    builder.master_table(master_table)
        .originating_center(originating_center)
        .originating_subcenter(originating_subcenter)
        .update_num(update_num)
        .data_category(data_category)
        .data_subcategory(data_subcategory)
        .local_data_subcategory(local_data_subcategory)
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
