use std::fmt::Display;

pub(crate) mod message_builder;
pub(crate) use message_builder::BufrMessageBuilder;

pub(crate) mod structure;
pub use structure::{Element, Group, Replication, Structure, Value};

#[derive(Debug)]
pub struct BufrMessage {
    bufr_version: u8,
    master_table: u8,
    originating_center: u16,
    originating_subcenter: u16,
    update_num: u8,
    data_category: u8,
    data_subcategory: u8,
    local_data_subcategory: Option<u8>,
    bufr_master_table_version: u8,
    local_tables_version: u8,
    year: u16,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    extra_section_1_data: Vec<u8>,

    num_datasets: u16,
    observed_data: bool,
    compressed_data: bool,

    section_2_data: Vec<u8>,

    elements: Vec<Structure>,
}

impl BufrMessage {
    /** Query if there is any data in BUFR section 2. */
    pub fn section_2_present(&self) -> bool {
        !self.section_2_data.is_empty()
    }

    /** Get the BUFR Section 2 data. */
    pub fn section_2_data(&self) -> &[u8] {
        &self.section_2_data
    }

    /** Query if there is any extra data in BUFR section 1.
     *
     * There may be data here if there are any local tables defined by the originator etc. used.
     */
    pub fn section_1_extra_data_present(&self) -> bool {
        !self.extra_section_1_data.is_empty()
    }

    /** Get the elements vector, or a vector of structures. */
    pub fn get_elements(&self) -> &[Structure] {
        &self.elements
    }

    fn master_table_str(&self) -> &'static str {
        match self.master_table {
            0 => "Meteorology (maintained by WMO)",
            10 => "Oceanography (maintained by IOC of UNESCO)",
            x => panic!("Nonexistent Master Table Version: {}", x),
        }
    }
}

#[rustfmt::skip]
impl Display for BufrMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        writeln!(f)?;
        writeln!(f, "             BUFR version: {}", self.bufr_version)?;
        writeln!(f)?;
        writeln!(f, "             Master Table: {} - {}", self.master_table, self.master_table_str())?;
        writeln!(f, "BUFR Master Table Version: {}", self.bufr_master_table_version)?;
        writeln!(f, "     Local Tables Version: {}", self.local_tables_version)?;
        writeln!(f)?;
        writeln!(f, "            Data Category: {}", self.data_category)?;
        writeln!(f, "         Data Subcategory: {}", self.data_subcategory)?;
        writeln!(f, "   Local Data Subcategory: {:?}", self.local_data_subcategory)?;
        writeln!(f)?;
        writeln!(f, "       Originating Center: {}", self.originating_center)?;
        writeln!(f, "    Originating Subcenter: {}", self.originating_subcenter)?;
        writeln!(f, "            Update Number: {}", self.update_num)?;
        writeln!(f)?;
        writeln!(f, "     Extra Section 1 Data: {}", self.section_1_extra_data_present())?;
        writeln!(f, "        Section 2 Present: {}", self.section_2_present())?;
        writeln!(f)?;
        writeln!(f, "                     Year: {}", self.year)?;
        writeln!(f, "                    Month: {}", self.month)?;
        writeln!(f, "                      Day: {}", self.day)?;
        writeln!(f, "                     Hour: {}", self.hour)?;
        writeln!(f, "                   Minute: {}", self.minute)?;
        writeln!(f, "                   Second: {}", self.second)?;
        writeln!(f)?;
        writeln!(f, "       Number of Datasets: {}", self.num_datasets)?;
        writeln!(f, "            Observed Data: {}", self.observed_data)?;
        writeln!(f)?;

        if !self.section_2_data.is_empty() {
            writeln!(f, "   Size of Section 2 Data: {}", self.section_2_data.len())?;
            writeln!(f)?;
        }

        writeln!(f, "-------------------- Data --------------------")?;
        writeln!(f)?;

        for structure in self.elements.iter() {
            structure::print_structure_data(f, structure, &mut vec![])?;
        }

        Ok(())
    }
}
