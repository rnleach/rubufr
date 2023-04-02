use super::{BufrMessage, Structure};

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
                local_data_subcategory: None,
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

    pub fn local_data_subcategory(&mut self, local_data_subcategory: Option<u8>) -> &mut Self {
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
