
use crate::codetables::Codetable;

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct HorizontalLayerProductDefinition {
    pub parameter_category: u8,
    pub parameter_number: u8,
    pub generating_process: u8,
    pub background_generating_process_identifier: u8,
    pub generating_process_identified: u8,
    pub hours_after_ref_time: u16,
    pub minutes_after_ref_time: u8,
    pub indicator_of_time_range_unit: u8,
    pub forecast_time: u32,
    pub first_fixed_surface_type: u8,
    pub first_fixed_surface_scale_factor: u8,
    pub first_fixed_surface_scale_value: i64,
    pub second_fixed_surface_type: u8,
    pub second_fixed_surface_scale_factor: u8,
    pub second_fixed_surface_scale_value: i64
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct FixedSurface {
    pub sfc_type: u8,
    pub sfc_scale_factor: u8,
    pub sfc_scale_value: i64
}

impl FixedSurface {
    pub fn new(sft: u8) -> FixedSurface {
        FixedSurface {
            sfc_type: sft,
            sfc_scale_factor: 0,
            sfc_scale_value: 0
        }
    }

    pub fn with_factor(mut self, factor: u8) -> FixedSurface {
        self.sfc_scale_factor = factor;
        self
    }

    pub fn with_value(mut self, value: i64) -> FixedSurface {
        self.sfc_scale_value = value;
        self
    }

    pub fn sfc_type(&self, codetable_4_5: &Codetable) -> Option<String> {
        codetable_4_5.codepoint_lookup(
            self.sfc_type as i64, 
            codetable_4_5.find_parameter("Meaning")?)
    }
}

impl HorizontalLayerProductDefinition {
    pub fn new(values: &Vec<i64>) -> Option<HorizontalLayerProductDefinition> {
        Some(HorizontalLayerProductDefinition {
            parameter_category: *values.get(0)? as u8,
            parameter_number: *values.get(1)? as u8,
            generating_process: *values.get(2)? as u8,
            background_generating_process_identifier: *values.get(3)? as u8,
            generating_process_identified: *values.get(4)? as u8,
            hours_after_ref_time: *values.get(5)? as u16,
            minutes_after_ref_time: *values.get(6)? as u8,
            indicator_of_time_range_unit: *values.get(7)? as u8,
            forecast_time: *values.get(8)? as u32,
            first_fixed_surface_type: *values.get(9)? as u8,
            first_fixed_surface_scale_factor: *values.get(10)? as u8,
            first_fixed_surface_scale_value: *values.get(11)?,
            second_fixed_surface_type: *values.get(12)? as u8,
            second_fixed_surface_scale_factor: *values.get(13)? as u8,
            second_fixed_surface_scale_value: *values.get(14)?
        })
    }

    pub fn parameter_info(&self, code_table_4_2: &Codetable, prod_dsc: i64) -> Option<(String, String)> {
        code_table_4_2.parameter_number_codepoint_lookup(
            prod_dsc, 
            self.parameter_category as i64, 
            self.parameter_number as i64)
    }

    pub fn gen_process_type(&self, code_table_4_3: &Codetable) -> Option<String> {
        code_table_4_3.codepoint_lookup(
            self.generating_process as i64, 
            "Meaning")
    }

    pub fn get_fixed_surfaces(&self) -> (FixedSurface, FixedSurface) {
        (
            FixedSurface {
                sfc_type: self.first_fixed_surface_type,
                sfc_scale_factor: self.first_fixed_surface_scale_factor,
                sfc_scale_value: self.first_fixed_surface_scale_value
            },
            FixedSurface {
                sfc_type: self.second_fixed_surface_type,
                sfc_scale_factor: self.second_fixed_surface_scale_factor,
                sfc_scale_value: self.second_fixed_surface_scale_value
            }
        )
    }    
}
