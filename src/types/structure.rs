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

pub(super) fn print_structure_data(
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
