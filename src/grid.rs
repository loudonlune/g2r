
use crate::codetables::Codetable;

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct LatLonGridDefinition {
    pub earth_shape: u8,
    pub earth_radius_scale_factor: u8,
    pub earth_radius_scale_value: i64,
    pub earth_oblate_spheroid_major_axis_scale_factor: u8,
    pub earth_oblate_spheroid_major_axis_scale_value: i64,
    pub earth_oblate_spheroid_minor_axis_scale_factor: u8,
    pub earth_oblate_spheroid_minor_axis_scale_value: i64,
    pub parallel_point_count: i64,
    pub meridional_point_count: i64,
    pub init_production_domain_basic_angle: i64,
    pub basic_angle_subdivisions: i64,
    pub first_gridpoint_latitude: i64,
    pub first_gridpoint_longitude: i64,
    pub resolution_component_flags: u8,
    pub last_gridpoint_latitude: i64,
    pub last_gridpoint_longitude: i64,
    pub direction_increment_i: i64,
    pub direction_increment_j: i64,
    pub scanning_mode: u8,
    pub list_point_counts: Vec<i64>
}

impl LatLonGridDefinition {
    pub fn new(grid_template: &Vec<i64>) -> Option<LatLonGridDefinition> {
        Some(LatLonGridDefinition {
            earth_shape: *grid_template.get(0)? as u8,
            earth_radius_scale_factor: *grid_template.get(1)? as u8,
            earth_radius_scale_value: *grid_template.get(2)?,
            earth_oblate_spheroid_major_axis_scale_factor: *grid_template.get(3)? as u8,
            earth_oblate_spheroid_major_axis_scale_value: *grid_template.get(4)?,
            earth_oblate_spheroid_minor_axis_scale_factor: *grid_template.get(5)? as u8,
            earth_oblate_spheroid_minor_axis_scale_value: *grid_template.get(6)?,
            parallel_point_count: *grid_template.get(7)?,
            meridional_point_count: *grid_template.get(8)?,
            init_production_domain_basic_angle: *grid_template.get(9)?,
            basic_angle_subdivisions: *grid_template.get(10)?,
            first_gridpoint_latitude: *grid_template.get(11)?,
            first_gridpoint_longitude: *grid_template.get(12)?,
            resolution_component_flags: *grid_template.get(13)? as u8,
            last_gridpoint_latitude: *grid_template.get(14)?,
            last_gridpoint_longitude: *grid_template.get(15)?,
            direction_increment_i: *grid_template.get(16)?,
            direction_increment_j: *grid_template.get(17)?,
            scanning_mode: *grid_template.get(18)? as u8,
            list_point_counts: (19..grid_template.len()).map(|exnn| *grid_template.get(exnn).unwrap()).collect()
        })
    }

    pub fn earth_shape(&self, code_table_3_2: &Codetable) -> Option<String> {
        code_table_3_2.codepoint_lookup(
            self.earth_shape as i64,
            code_table_3_2.find_parameter("Meaning")?
        )
    }

    pub fn lat_span(&self) -> (i64, i64) {
        (self.first_gridpoint_latitude, self.last_gridpoint_latitude)
    }

    pub fn lon_span(&self) -> (i64, i64) {
        (self.first_gridpoint_longitude, self.last_gridpoint_longitude)
    }
}
