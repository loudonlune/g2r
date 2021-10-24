
use std::{collections::{HashMap}, fmt::{Debug, Display, Formatter}, fs::File, io::{BufReader}, path::Path};
use std::fs;

use text_io::scan;

#[derive(Clone)]
pub struct Codetable {
    is_template: bool,
    path: String,
    titles: Vec<String>,
    data: Vec<Vec<String>>
}

impl Debug for Codetable {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f,
        "Codetable")
    }
}

pub enum CodetableLoadError {
    IOError(String),
    InvalidCSV
}

impl Debug for CodetableLoadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f,
        "CodetableError: {}",
        match self {
            CodetableLoadError::IOError(val) => (*val).clone(),
            CodetableLoadError::InvalidCSV => String::from("InvalidCSV")
        })
    }
}

impl From<std::io::Error> for CodetableLoadError {
    fn from(ioerr: std::io::Error) -> Self {
        Self::IOError(ioerr.to_string())
    }
}

impl Codetable {
    pub fn new(path: &Path) -> Result<Codetable, CodetableLoadError> {
        let mut csvreader = csv::Reader::from_reader(BufReader::new(File::open(path)?));

        let packed_table: Vec<Vec<String>> = csvreader.records().map(|x| x.unwrap().iter().map(|x| x.to_string()).collect()).collect();
        let titles: Vec<String> = csvreader.headers().unwrap().iter().map(|x| x.to_string()).collect();

        Ok(Codetable {
            path: path.to_str().unwrap().to_string(),
            is_template: titles.contains(&String::from("OctetNo")),
            titles: titles,
            data: packed_table
        })
    }

    pub fn header(&self) -> &Vec<String> {
        &self.titles
    }

    pub fn columns(&self) -> usize {
        self.titles.len()
    }

    pub fn rows(&self) -> usize {
        self.data.len()
    }

    pub fn column_index(&self, name: &str) -> Option<usize> {
        for t in self.titles.iter().enumerate() {
            if t.1.eq(name) {
                return Some(t.0)
            }
        }

        return None
    }

    pub fn row(&self, row: usize) -> Option<Vec<String>> {
        Some(self.data.get(row)?.clone())
    }

    pub fn column(&self, col: usize) -> Option<Vec<String>> {
        Some({
            let mut collection = Vec::new();
            
            for row in &self.data {
                collection.push((*row.get(col)?).clone())
            }

            collection
        })
    }

    pub fn lookup(&self, lookup_value: String, lookup_column: usize, output_column: usize) -> Option<String> {
        for lv in self.data.get(lookup_column)?.iter().enumerate() {
            if lv.1.eq(&lookup_value) {
                return Some(self.data.get(output_column)?.get(lv.0)?.clone());
            }
        }

        None
    }

    pub fn find_parameter(&self, keyword: &str) -> Option<&String> {
        for x in self.header() {
            if x.contains(keyword) {
                return Some(x);
            }
        }

        None
    }

    pub fn codepoint_lookup(&self, codepoint: i64, parameter: &str) -> Option<String> {
        let in_param = if self.is_template { "OctetNo" } else { "CodeFlag" };

        for col_val in self.column(self.column_index(self.find_parameter(in_param)?)?)?.iter().enumerate() {
            if col_val.1.contains('-') {
                let range: Vec<i64> = col_val.1.split('-').map(|x| { x.to_string().parse::<i64>().unwrap() }).collect();

                if codepoint >= *range.get(0).unwrap() && codepoint <= *range.get(1).unwrap() {
                    return Some((*self.column(self.column_index(parameter)?)?.get(col_val.0)?).clone())
                }
            } else {
                let str_val = col_val.1.parse::<i64>().unwrap();

                if str_val == codepoint {
                    return Some((*self.column(self.column_index(parameter)?)?.get(col_val.0)?).clone())
                }
            }
        }

        None
    }

    /*
    Returns a tuple containing (Field type, Units)
    */
    pub fn parameter_number_codepoint_lookup(&self, discipline: i64, parameter_category: i64, codepoint: i64) -> Option<(String, String)> {
        let code_flags = self.column(self.column_index(self.find_parameter("CodeFlag")?)?)?;
        let column_subtitles = self.column(self.column_index(self.find_parameter("SubTitle")?)?)?;

        for i in 0..self.rows() {
            let cflo;
            let cfhi;

            let cfstr = code_flags.get(i)?;

            if cfstr.contains('-') {
                let range: Vec<i64> = cfstr.split('-').map(|x| { x.to_string().parse::<i64>().unwrap() }).collect();

                cfhi = *range.get(1).unwrap();
                cflo = *range.get(0).unwrap();
            } else {
                cfhi = cfstr.parse::<i64>().unwrap_or(0);
                cflo = cfhi;
            }
            
            let disc: i64;
            let disc_text: String;
            let param_cat: i64;
            let param_cat_text: String;

            scan!(
                column_subtitles.get(i)?.bytes() => "Product discipline {} - {}, parameter category {}: {}", 
                disc, 
                disc_text, 
                param_cat, 
                param_cat_text);

            if disc == discipline && parameter_category == param_cat && (cflo <= codepoint && cfhi >= codepoint) {
                return Some((
                    (*self.column(self.column_index(self.find_parameter("Meaning")?)?)?.get(i)?).clone(), 
                    (*self.column(self.column_index(self.find_parameter("Unit")?)?)?.get(i)?).clone())
                )
            }            
        }

        None
    }

    pub fn path(&self) -> &str {
        self.path.as_str()
    }
}

impl Display for Codetable {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "Codetable: \n")?;
        write!(f, "CT Path: {}\n", self.path)?;

        for title in self.titles.iter().enumerate() {
            write!(f, "\t{}:\n", title.1)?;
            
            for cv in self.column(title.0).unwrap() {
                write!(f, "\t\t{}\n", cv)?;
            }
        }

        write!(f, "END Codetable")
    }
}

pub struct CodetableManager {
    tables: HashMap<String, Codetable>
}

impl CodetableManager {
    pub fn new() -> CodetableManager {
        CodetableManager {
            tables: HashMap::new()
        }
    }

    pub fn scan_dir(&mut self, dir: &Path) -> Result<usize, CodetableLoadError> {
        let mut read = 0;

        for entry in fs::read_dir(dir).unwrap() {
            if let Ok(file) = entry {
                let filename = file.file_name();
                let filename_str = filename.to_str().unwrap();

                if filename_str.ends_with(".csv") {
                    let tokens: Vec<&str> = filename_str.split(".").collect();
                    self.tables.insert(tokens.get(0).unwrap().to_string(), Codetable::new(file.path().as_path()).unwrap());
                    read += 1;
                }
            }
        }

        Ok(read)
    }

    pub fn search_for_key(&self, keyword: &str) -> Option<&str> {
        for x in self.tables.iter() {
            if x.0.contains(keyword) {
                return Some(x.0.as_str());
            }
        }

        None
    }

    pub fn table_names(&self) -> Vec<String> {
        self.tables.keys().map(|x| x.clone()).collect()
    }

    pub fn search_for_table(&self, keyword: &str) -> Option<&Codetable> {
        self.table(self.search_for_key(keyword)?)
    }

    pub fn table(&self, name: &str) -> Option<&Codetable> {
        self.tables.get(name)
    }

    pub fn query_parameters(&self, name: &str) -> Option<&Vec<String>> {
        Some(self.table(name)?.header())
    }
}
