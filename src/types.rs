use std::fmt::Display;

pub(crate) struct BufrMessageBuilder {
    bm: BufrMessage,
}

impl BufrMessageBuilder {
    pub fn new() -> Self {
        BufrMessageBuilder {
            bm: BufrMessage {
                bufr_version: !0,
                master_table: !0,
                originating_center: !0,
                originating_subcenter: !0,
                update_num: !0,
                data_category: !0,
                data_subcategory: !0,
                local_data_subcategory: !0,
                bufr_master_table_version: !0,
                local_tables_version: !0,
                year: !0,
                month: !0,
                day: !0,
                hour: !0,
                minute: !0,
                second: !0,
                extra_section_1_data: vec![],

                num_datasets: !0,
                observed_data: false,
                compressed_data: false,

                section_2_data: vec![],

                elements: vec![],
            },
        }
    }

    pub fn bufr_version(&mut self, version: u8) -> &mut Self {
        self.bm.bufr_version = version;
        self
    }

    pub fn master_table(&mut self, master_table: u8) -> &mut Self {
        self.bm.master_table = master_table;
        self
    }

    pub fn originating_center(&mut self, originating_center: u16) -> &mut Self {
        self.bm.originating_center = originating_center;
        self
    }

    pub fn originating_subcenter(&mut self, originating_subcenter: u16) -> &mut Self {
        self.bm.originating_subcenter = originating_subcenter;
        self
    }

    pub fn update_num(&mut self, update_num: u8) -> &mut Self {
        self.bm.update_num = update_num;
        self
    }

    pub fn data_category(&mut self, data_category: u8) -> &mut Self {
        self.bm.data_category = data_category;
        self
    }

    pub fn data_subcategory(&mut self, data_subcategory: u8) -> &mut Self {
        self.bm.data_subcategory = data_subcategory;
        self
    }

    pub fn local_data_subcategory(&mut self, local_data_subcategory: u8) -> &mut Self {
        self.bm.local_data_subcategory = local_data_subcategory;
        self
    }

    pub fn bufr_master_table_version(&mut self, bufr_master_table_version: u8) -> &mut Self {
        self.bm.bufr_master_table_version = bufr_master_table_version;
        self
    }

    pub fn local_tables_version(&mut self, local_tables_version: u8) -> &mut Self {
        self.bm.local_tables_version = local_tables_version;
        self
    }

    pub fn year(&mut self, year: u16) -> &mut Self {
        self.bm.year = year;
        self
    }

    pub fn month(&mut self, month: u8) -> &mut Self {
        self.bm.month = month;
        self
    }

    pub fn day(&mut self, day: u8) -> &mut Self {
        self.bm.day = day;
        self
    }

    pub fn hour(&mut self, hour: u8) -> &mut Self {
        self.bm.hour = hour;
        self
    }

    pub fn minute(&mut self, minute: u8) -> &mut Self {
        self.bm.minute = minute;
        self
    }

    pub fn second(&mut self, second: u8) -> &mut Self {
        self.bm.second = second;
        self
    }

    pub fn extra_seciont_1_data(&mut self, extra_data: Vec<u8>) -> &mut Self {
        self.bm.extra_section_1_data = extra_data;
        self
    }

    pub fn num_datasets(&mut self, num_datasets: u16) -> &mut Self {
        self.bm.num_datasets = num_datasets;
        self
    }

    pub fn observed_data(&mut self, observed_data: bool) -> &mut Self {
        self.bm.observed_data = observed_data;
        self
    }

    pub fn compressed_data(&mut self, compressed_data: bool) -> &mut Self {
        self.bm.compressed_data = compressed_data;
        self
    }

    pub fn section_2_data(&mut self, section_2_data: Vec<u8>) -> &mut Self {
        self.bm.section_2_data = section_2_data;
        self
    }

    pub fn elements(&mut self, elements: Vec<Structure>) -> &mut Self {
        self.bm.elements = elements;
        self
    }

    pub fn build(self) -> BufrMessage {
        if self.bm.num_datasets > 1 {
            panic!("multiple datasets not implemented");
        }

        if self.bm.bufr_master_table_version > crate::MAX_BUFR_TABLE_VERSION_SUPPORTED {
            panic!("data encoded with tables newer than supported in this version");
        }

        if self.bm.bufr_version > crate::MAX_BUFR_EDITION_SUPPORTED
            || self.bm.bufr_version < crate::MIN_BUFR_EDITION_SUPPORTED
        {
            panic!(
                "data encoded with BUFR version {}, which is unsupported.",
                self.bm.bufr_version
            );
        }

        self.bm
    }
}

#[derive(Debug)]
pub struct BufrMessage {
    bufr_version: u8,
    master_table: u8,
    originating_center: u16,
    originating_subcenter: u16,
    update_num: u8,
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
        writeln!(f, "   Local Data Subcategory: {}", self.local_data_subcategory)?;
        writeln!(f)?;
        writeln!(f, "       Originating Center: {}", self.originating_center)?;
        writeln!(f, "    Originating Subcenter: {}", self.originating_subcenter)?;
        writeln!(f, "            Update Number: {}", self.update_num)?;
        writeln!(f)?;
        writeln!(f, "       Extra Sect. 1 Data: {}", self.section_1_extra_data_present())?;
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
            print_structure_data(f, structure, 0)?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub(crate) enum Value {
    Missing,
    Float(f64),
    Code(u64),
    Numeric(i64),
    Str(String),
}

#[derive(Debug)]
pub(crate) struct Element {
    val: Value,
    fxy: &'static str,
    units: &'static str,
    name: &'static str,
}

impl Element {
    pub fn new(val: Value, units: &'static str, name: &'static str, fxy: &'static str) -> Self {
        Self {
            val,
            units,
            name,
            fxy,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Group {
    items: Vec<Structure>,
    fxy: &'static str,
    name: &'static str,
}

impl Group {
    pub fn new_with_capacity(cap: usize, name: &'static str, fxy: &'static str) -> Self {
        Group {
            name,
            fxy,
            items: Vec::with_capacity(cap),
        }
    }

    pub fn push(&mut self, structure: Structure) {
        self.items.push(structure);
    }
}

#[derive(Debug)]
pub(crate) struct Replication {
    items: Vec<Structure>,
}

impl Replication {
    pub fn new_with_capacity(cap: usize) -> Self {
        Replication {
            items: Vec::with_capacity(cap),
        }
    }

    pub fn push(&mut self, structure: Structure) {
        self.items.push(structure);
    }
}

#[derive(Debug)]
pub(crate) enum Structure {
    Element(Element),
    Group(Group),
    Replication(Replication),
}

fn print_structure_data(
    f: &mut std::fmt::Formatter,
    structure: &Structure,
    level: u32,
) -> Result<(), std::fmt::Error> {
    for _ in 0..(4 * level) {
        write!(f, " ")?;
    }

    match structure {
        Structure::Element(e) => {
            write!(f, r#"Element: "{:6}" | Value: "#, e.fxy)?;
            match &e.val {
                Value::Missing => write!(f, "{:12}", "Missing")?,
                Value::Float(v) => write!(f, "{:12}", v)?,
                Value::Code(c) => write!(f, "{:12}", c)?,
                Value::Numeric(n) => write!(f, "{:12}", n)?,
                Value::Str(s) => write!(f, r#"{:12}"#, s)?,
            }
            writeln!(f, r#" | Units: {:12} | Name: "{}""#, e.units, e.name)?;
        }
        Structure::Replication(r) => {
            writeln!(f, r#"Replication ({})"#, r.items.len())?;
            let mut iter = r.items.iter();
            for item in iter.by_ref().take(2) {
                print_structure_data(f, item, level + 1)?;
            }
            if let Some(item) = iter.last() {
                for _ in 0..6 {
                    for _ in 0..(4 * level) {
                        write!(f, " ")?;
                    }
                    writeln!(f, ".")?;
                }
                print_structure_data(f, item, level + 1)?;
            }
        }
        Structure::Group(g) => {
            writeln!(f, r#"Group: "{:6}" | "{}""#, g.fxy, g.name)?;
            for item in &g.items {
                print_structure_data(f, item, level + 1)?;
            }
        }
    }

    Ok(())
}
