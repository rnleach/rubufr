pub mod table_b;
pub mod table_d;

pub struct TableBEntry {
    pub(crate) fxy: &'static str,
    pub(crate) width_bits: usize,
    pub(crate) element_name: &'static str,
    pub(crate) units: &'static str,
    pub(crate) reference_val: i64,
    pub(crate) scale_val: i32,
}

pub struct TableDEntry {
    pub(crate) fxy: &'static str,
    pub(crate) group_name: &'static str,
    pub(crate) elements: Vec<&'static str>,
}
