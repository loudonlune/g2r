
use std::{ffi::{CString}, alloc::{Layout, alloc, dealloc}, collections::HashMap, fmt::{Display, Formatter}, mem::size_of, u32};

use chrono::{DateTime, Utc};
use chrono::prelude::*;
use libg2c_sys;

pub mod codetables;
pub mod grid;
pub mod product;

use grid::LatLonGridDefinition;
use product::{HorizontalLayerProductDefinition, FixedSurface};

#[derive(Debug, Clone)]
pub enum Grib2Error {
    InfoReadError(u8),
    FieldReadError(u8),
    FunctionNotImplemented,
    Unknown
}

impl ToString for Grib2Error {
    fn to_string(&self) -> String {
        String::from(
            match self {
                Grib2Error::InfoReadError(cdp) => {
                    match cdp {
                        0 => "No Error",
                        1 => "Beginning chars \"GRIB\" not found",
                        2 => "GRIB message is not valid for GRIB2",
                        3 => "Could not find Section 1 where expected",
                        4 => "End string \"7777\" found but not where expected",
                        5 => "End string \"7777\" not found at end of message",
                        _ => "unknown"
                    }
                },
                Grib2Error::FieldReadError(cdp) => {
                    match cdp {
                        0 => "No Error",
                        1 => "Beginning chars \"GRIB\" not found",
                        2 => "GRIB message is not valid for GRIB2",
                        3 => "Data field request number was not positive",
                        4 => "End string \"7777\" found but not where expected",
                        6 => "GRIB message did not contain requested count of data fields",
                        7 => "End string \"7777\" not found at end of message",
                        8 => "Unrecognized section encountered",
                        9 => "Data Representation Template 5.NN not yet implemented.",
                        15 => "Error unpacking Section 1",
                        16 => "Error unpacking Section 2",
                        10 => "Error unpacking Section 3",
                        11 => "Error unpacking Section 4",
                        12 => "Error unpacking Section 5",
                        13 => "Error unpacking Section 6",
                        14 => "Error unpacking Section 7",
                        _ => "unknown"
                    }
                }
                _ => "unknown"
            }
        )
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Grib2Info {
    pub center: u16,
    pub subcenter: u16,
    pub master_table_version: u8,
    pub local_table_version: u8,
    pub ref_time_significance: u8,
    pub ref_time: DateTime<Utc>,
    pub prod_status: u8,
    pub data_type: u8,
    pub discipline: u8,
    pub grib_edition: u8,
    pub length: i64
}

impl Display for Grib2Info {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "\
discipline:             {},
grib_version:           {},
length:                 {},
center:                 {},
subcenter:              {},
master_table_version:   {},
local_table_version:    {},
ref_time_significance:  {},
ref_time:               {},
prod_status:            {},
data_type:              {}
        ",
        self.discipline,
        self.grib_edition,
        self.length,
        self.center,
        self.subcenter,
        self.master_table_version,
        self.local_table_version,
        self.ref_time_significance,
        self.ref_time,
        self.prod_status,
        self.data_type)
    }
}

impl Grib2Info {
    unsafe fn new(sec0: *mut i64, sec1: *mut i64) -> Grib2Info {
        Grib2Info {
            center: *sec1 as u16,
            subcenter: *sec1.offset(1) as u16,
            master_table_version: *sec1.offset(2) as u8,
            local_table_version: *sec1.offset(3) as u8,
            ref_time_significance: *sec1.offset(4) as u8,
            ref_time: Utc.ymd(
                *sec1.offset(5) as i32, 
                *sec1.offset(6) as u32, 
                *sec1.offset(7) as u32)
            .and_hms(
                *sec1.offset(8) as u32, 
                *sec1.offset(9) as u32, 
                *sec1.offset(10) as u32),
            prod_status: *sec1.offset(11) as u8,
            data_type: *sec1.offset(12) as u8,
            discipline: *sec0 as u8,
            grib_edition: *sec0.offset(1) as u8,
            length: *sec0.offset(2)
        }
    }
}

#[derive(Clone)]
pub struct Grib2Field {
    pub field_number: u32, // field info

    local: Option<String>, // local info

    // gridpoint info
    grd_is_ct31: bool,
    grid_template: Vec<i64>,
    grid_arrays_lengths_list: Option<Vec<i64>>,
    pub interpretation_of_optional_gridpoints: u8,
    pub grid_template_len: u32,
    pub num_coords: u32,
    pub count_gridpoints: u32,
    pub count_optional_octets_grid: u8,
    pub grid_def: u32,

    // product info
    prod_template: Vec<i64>,
    pub prod_codepoint: u16,
    pub prod_template_len: u32,

    // finally, data
    expanded: bool,
    unpacked: bool,
    data_representation_template: Vec<i64>,
    pub data_representation_codepoint: u16,
    pub data_representation_template_len: u32,
    
    bitmap: Option<Vec<i64>>,
    pub bitmap_indicator: u8,
    
    gridpoint_data: Vec<f32>,
    pub num_datapoints: u32
}

impl Display for Grib2Field {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f,
        "\
field_number:                           {},
local:                                  {},
grid_is_ct31:                           {},
grid_template:                          {},
grid_arrays_lengths_list:               {},
interpretation_of_optional_gridpoints:  {},
grid_template_len:                      {},
num_coords:                             {},
count_gridpoints:                       {},
count_optional_octets_grid:             {},
grid_def:                               {},
prod_template:                          {},
prod_codepoint:                         {},
prod_template_len:                      {},
expanded:                               {},
unpacked:                               {},
data_representation_template:           {},
data_representation_codepoint:          {},
data_representation_template_len:       {},
bitmap:                                 {},
bitmap_indicator:                       {},
num_datapoints:                         {},
gridpoint_data:                         {}
        ",
        self.field_number,
        if let Some(local) = self.local() { (*local).clone() } else { String::from("None") },
        self.grid_data_is_ct31(),
        self.grid_template().len(),
        if let Some(gall) = self.get_grid_arrays_list() { gall.len().to_string() } else { String::from("None") },
        self.interpretation_of_optional_gridpoints,
        self.grid_template_len,
        self.num_coords,
        self.count_gridpoints,
        self.count_optional_octets_grid,
        self.grid_def,
        self.prod_template_values().len(),
        self.prod_codepoint,
        self.prod_template_len,
        self.expanded,
        self.unpacked,
        self.data_representation_template.len(),
        self.data_representation_codepoint,
        self.data_representation_template_len,
        if let Some(bitmap) = self.bitmap() { bitmap.len().to_string() } else { String::from("None") },
        self.bitmap_indicator,
        self.num_datapoints,
        self.gridpoint_data.len())
    }
}

impl Grib2Field {
    unsafe fn new(data: *mut libg2c_sys::gribfield) -> Grib2Field {
        Grib2Field {
            local: if (*data).locallen > 0 {
                if let Ok(cstr) = std::ffi::CStr::from_ptr((*data).local as *const i8).to_str() {
                    Some(cstr.to_string())
                } else {
                    None
                }
            } else {
                None
            },
            field_number: (*data).ifldnum as u32,

            grd_is_ct31: (*data).griddef == 0,
            num_coords: (*data).num_coord as u32,
            count_gridpoints: (*data).ngrdpts as u32,
            count_optional_octets_grid: (*data).numoct_opt as u8,
            interpretation_of_optional_gridpoints: (*data).interp_opt as u8,
            grid_def: (*data).igdtnum as u32,
            grid_template: (0..(*data).igdtlen).map(|x| *((*data).igdtmpl).offset(x as isize)).collect(),
            grid_template_len: (*data).igdtlen as u32,
            grid_arrays_lengths_list: if (*data).numoct_opt > 0 { 
                    Some( 
                        (0..(*data).num_opt)
                        .map(|x| {
                            *(*data).list_opt.offset(x as isize)
                        }).collect()
                    )
                } else {
                    None
                },

            prod_codepoint: (*data).ipdtnum as u16,
            prod_template_len: (*data).ipdtlen as u32,
            prod_template: (0..(*data).ipdtlen)
                .map(|x| *((*data).ipdtmpl.offset(x as isize)))
                .collect(),
            
            gridpoint_data: (0..(*data).ndpts)
                .map(|x| *((*data).fld.offset(x as isize)))
                .collect(),
            num_datapoints: (*data).ndpts as u32,
            unpacked: (*data).unpacked == 1,
            expanded: (*data).expanded == 1,
            data_representation_codepoint: (*data).idrtnum as u16,
            data_representation_template: (0..(*data).idrtlen)
                .map(|x| {
                    *((*data).idrtmpl.offset(x as isize))
                })
                .collect(),
            data_representation_template_len: (*data).idrtlen as u32,
            bitmap_indicator: (*data).ibmap as u8,
            bitmap: None
        }
    }

    pub fn is_expanded(&self) -> bool {
        self.expanded
    }

    pub fn is_unpacked(&self) -> bool {
        self.unpacked
    }

    pub fn bitmap(&self) -> Option<&Vec<i64>> {
        self.bitmap.as_ref()
    }

    pub fn data_representation_template_values(&self) -> &Vec<i64> {
        &self.data_representation_template
    }

    pub fn prod_template_values(&self) -> &Vec<i64> {
        &self.prod_template
    }

    pub fn prod_template_as_horizontal_layer_definition(&self) -> Option<HorizontalLayerProductDefinition> {
        HorizontalLayerProductDefinition::new(self.prod_template_values())
    }

    pub fn grid_template_as_geograph_coordinate_grid_definition(&self) -> Option<LatLonGridDefinition> {
        LatLonGridDefinition::new(self.grid_template())
    }

    pub fn get_grid_arrays_list(&self) -> Option<&Vec<i64>> {
        self.grid_arrays_lengths_list.as_ref()
    }

    pub fn data(&self) -> &Vec<f32> {
        &self.gridpoint_data
    }

    pub fn grid_template(&self) -> &Vec<i64> {
        &self.grid_template
    }

    pub fn grid_data_is_ct31(&self) -> bool {
        self.grd_is_ct31
    }

    pub fn local(&self) -> Option<&String> {
        self.local.as_ref()
    }

    pub fn empty(&self) -> bool {
       self.gridpoint_data.is_empty()
    }
}

#[derive(Clone)]
pub struct Grib2Message {
    info: Result<Grib2Info, Grib2Error>,
    fields: Vec<Grib2Field>,
    errors: Vec<(usize, Grib2Error)>
}

impl Grib2Message {
    unsafe fn new(grib_data: *mut u8) -> Result<Grib2Message, Grib2Error> {
        let mut new_inst = Grib2Message {
            info: Err(Grib2Error::Unknown),
            fields: Vec::new(),
            errors: Vec::new()
        };

        // declare memory layouts for Section 0 and Section 1 of the GRIB2 file
        let sec0buf_layout = Layout::from_size_align(3 * size_of::<i64>(), 1).unwrap();
        let sec1buf_layout = Layout::from_size_align(13 * size_of::<i64>(), 1).unwrap();

        // allocate memory for sec 0 and sec 1
        let sec0buf = alloc(sec0buf_layout);
        let sec1buf = alloc(sec1buf_layout);

        let mut field: *mut libg2c_sys::gribfield = 0 as *mut libg2c_sys::gribfield;

        let mut count_fields: i64 = 0;
        let mut count_locals: i64 = 0;

        let info_err = libg2c_sys::g2_info(
            grib_data, 
            sec0buf as *mut i64, 
            sec1buf as *mut i64, 
            &mut count_fields, 
            &mut count_locals);

        new_inst.info = if info_err != 0 {
            Err(Grib2Error::InfoReadError(info_err as u8))
        } else {
            Ok(Grib2Info::new(sec0buf as *mut i64, sec1buf as *mut i64))
        };

        dealloc(sec0buf, sec0buf_layout);
        dealloc(sec1buf, sec1buf_layout);  
        
        for i in 0..count_fields {
            let field_error = libg2c_sys::g2_getfld(grib_data, i + 1, 1, 1, &mut field);
    
            if field_error == 0 {
                new_inst.fields.push(Grib2Field::new(field))
            } else {
                new_inst.errors.push((i as usize, Grib2Error::FieldReadError(field_error as u8)))
            }
    
            libg2c_sys::g2_free(field);
        }    
            
        Ok(new_inst)
    }

    pub fn info(&self) -> Result<&Grib2Info, &Grib2Error> {
        self.info.as_ref()
    }

    pub fn fields(&self) -> &Vec<Grib2Field> {
        &self.fields
    }

    pub fn errors(&self) -> &Vec<(usize, Grib2Error)> {
        &self.errors
    }
}

pub struct Grib2 {
    path: String,
    messages: Vec<Grib2Message>,
    errors: Vec<(usize, Grib2Error)>
}

impl Grib2 {
    pub fn empty(&self) -> bool {
        self.messages.is_empty()
    }

    pub fn new(path: String) -> Grib2 {
        Grib2 {
            path: path,
            messages: Vec::new(),
            errors: Vec::new()
        }
    }

    pub fn messages(&self) -> &Vec<Grib2Message> {
        self.messages.as_ref()
    }

    pub fn messages_by_layer(&self) -> HashMap<(FixedSurface, FixedSurface), Vec<Grib2Message>> {
        let mut map: HashMap<(FixedSurface, FixedSurface), Vec<Grib2Message>> = HashMap::new();

        for msg in &self.messages {
            for field in msg.fields() {
                if let Some(prod_def) = field.prod_template_as_horizontal_layer_definition() {
                    let fixedsfcs = prod_def.get_fixed_surfaces();
                    if let Some(vect) = map.get_mut(&fixedsfcs) {
                        vect.push((*msg).clone());
                    } else {
                        let mut nvec = Vec::new();
                        nvec.push((*msg).clone());
                        map.insert(
                            fixedsfcs,
                            nvec
                        );
                    }
                }
            }
        }

        map
    }

    pub fn errors(&self) -> &Vec<(usize, Grib2Error)> {
        &self.errors
    }

    pub fn path(&self) -> &String {
        &self.path
    }

    pub fn write_all(&self) -> Result<(), Grib2Error> {
        Err(Grib2Error::FunctionNotImplemented)
    }

    // returns the count of messages, or None if an error occurred while trying to read the file
    pub fn read_all(&mut self) -> Option<usize> {
        let mut msgs = Vec::new();

        unsafe {
            let fname = CString::new(self.path.as_str()).unwrap();
            let fmode = CString::new("r").unwrap();
            // Open C file
            let file_handle = libg2c_sys::fopen(
                fname.as_ptr(), 
                fmode.as_ptr());

            if file_handle == (0 as *mut libg2c_sys::_IO_FILE) {
                println!("g2r: Failed to open {}", self.path);
                let mut errno: i8 = 0;
                libg2c_sys::perror(&mut errno as *mut i8);

                return None;
            }

            if libg2c_sys::ferror(file_handle) != 0 {
                println!("g2r: ferror");
                libg2c_sys::fclose(file_handle);
                return None;
            }
    
            let mut gmsg_end = 0;
            let mut gmsg_begin = 0;
            let mut gmsg_prim_length = 0;

            let mut count = 0;
            loop { 
                // look for GRIB messages
                libg2c_sys::seekgb(file_handle, gmsg_end, 32000, &mut gmsg_begin, &mut gmsg_prim_length);
                // if no GRIB messages could be found, must be EOF or file is corrupt
                if gmsg_prim_length == 0 { break };

                // allocate space for GRIB message
                let gribmsg_memlayout = Layout::from_size_align(gmsg_prim_length as usize, 1).unwrap();
                let gribmsg = alloc(gribmsg_memlayout);

                // jump to message begin position
                libg2c_sys::fseek(file_handle, gmsg_begin, libg2c_sys::SEEK_SET as i32);
                
                // read the GRIB message
                libg2c_sys::fread(
                    gribmsg as *mut std::ffi::c_void, 
                    size_of::<libg2c_sys::__u_char>() as u64, 
                    gmsg_prim_length as u64, 
                    file_handle);

                // move the message begin ptr to the next message
                gmsg_end = gmsg_begin + gmsg_prim_length;

                // attempt to create new message
                let maybe_msg = Grib2Message::new(gribmsg);
                if let Err(why) = maybe_msg { // if we failed
                    // stow the error
                    self.errors.push((count, why))
                } else { // otherwise
                    // push the new message
                    count += 1;
                    msgs.push(maybe_msg.unwrap());
                }

                // free message mem
                dealloc(gribmsg, gribmsg_memlayout);
            }

            // close the file
            libg2c_sys::fclose(file_handle);
        }

        self.messages = msgs;

        Some(self.messages.len())
    }
}

mod tests {

    #[test]
    fn grib2_new() {
        let g2 = crate::Grib2::new(String::from("test_path"));

        assert_eq!(g2.path().as_str(), "test_path");
    }

    #[test]
    fn grib2_open() {
        let mut g2 = crate::Grib2::new(String::from("test.grib2"));
        let mut ctbs = crate::codetables::CodetableManager::new();

        assert!(ctbs.scan_dir(std::path::Path::new("GRIB2/")).is_ok());

        let ct00_table = ctbs.table(ctbs.search_for_key("CodeFlag_0_0_").unwrap()).unwrap();
        let ct40_table = ctbs.table(ctbs.search_for_key("CodeFlag_4_0_").unwrap()).unwrap();
        let ct41_table = ctbs.table(ctbs.search_for_key("CodeFlag_4_1_").unwrap()).unwrap();
        let ct42_table = ctbs.table(ctbs.search_for_key("CodeFlag_4_2_").unwrap()).unwrap();
        let ct43_table = ctbs.table(ctbs.search_for_key("CodeFlag_4_3_").unwrap()).unwrap();

        println!("Calling read_all...");
        g2.read_all();

        println!("Printing all message infos...");
        for msg in g2.messages.iter().enumerate() {
            println!("Message {}: ", msg.0);
            
            let mut discipline = 0;
            match msg.1.info() {
                Ok(info) => {
                    discipline = info.discipline;
                    println!("{}", info);
                    println!("Decoded values: ");

                    println!("\tDiscipline: {}", ct00_table
                    .codepoint_lookup(
                        info.discipline.clone() as i64, 
                        ct00_table
                        .find_parameter("Meaning")
                        .unwrap()
                        .as_str())
                        .unwrap());
                },
                Err(why) => println!("\tERROR: {}", why.to_string())
            }

            for field in msg.1.fields().iter().enumerate() {
                println!("\t FIELD {}: ", field.0);
                println!("{}", field.1);

                println!("Decoded values: ");
                println!("\tProduct Template Type: {}", ct40_table
                .codepoint_lookup(
                    field.1.prod_codepoint.clone() as i64,
                    ct40_table
                    .find_parameter("Meaning")
                    .unwrap()
                    .as_str())
                    .unwrap()
                );

                println!("\tParameter Category: {}", ct41_table
                .codepoint_lookup(
                    *field
                    .1
                    .prod_template_values()
                    .get(0)
                    .unwrap(), 
                    ct41_table
                    .find_parameter("Meaning")
                    .unwrap()
                    .as_str())
                    .unwrap()
                );

                let pinfo = ct42_table.parameter_number_codepoint_lookup(
                    discipline as i64, 
                    *field.1.prod_template_values().get(0).unwrap(), 
                    *field.1.prod_template_values().get(1).unwrap())
                    .unwrap();

                println!("\tParameter info: ({}, {})", pinfo.0, pinfo.1);

                println!("\tGeneration Process: {}", ct43_table.codepoint_lookup(
                    *field.1.prod_template_values().get(2).unwrap(), 
                    ct43_table.find_parameter("Meaning").unwrap().as_str()).unwrap()
                    )
            }
        }
    }
}
