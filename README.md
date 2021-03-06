
g2r
==============
g2r is currently a simple (and unfinished) "safe" and rusty binding of NCEP's g2c library for Rust. Its aim is to make the library easier to use within Rust programs. It provides wrapper structs for templates used within the GRIB2 data format standard, and hopes to one day fully encapsulate most or all of the standard. 

All credit for the test GRIB2 file provided and the g2c library itself is attributed to the NOAA's National Center for Environmental Prediction.
The g2c repository can be found here: https://github.com/NOAA-EMC/NCEPLIBS-g2c

What can g2r do?
--------------
Currently, g2r has enough functionality to read the global grid used by NCEP's GFS deterministic model. It can only reliably unpack a single grid template at this time.
Future versions will allow unpacking of more complex grids such as those used for the high resolution model family.



Building g2r
--------------
Building g2r is very simple. The build script handles pulling down g2c and generating bindings for it.
Simply add it as a dependency for your project:
```
[dependencies]
g2r = "0.1.0"
```
Now you're all set to begin using the crate.

Testing
--------------
To test, download a GRIB2 file from NOMADS here: https://nomads.ncep.noaa.gov/
Name this file test.grib2 and move it into where you cloned this repository, and run cargo test.

Example
-------------
Using g2r in your application is easy. Simply call the read_all() fn on your Grib2 instance, and messages will become available.
```
fn main() {
    let g2r = Grib2::new(String::from("some_data.grib2"));
    g2r.read_all();

    for message in g2r.messages() {
        for field in message.fields() {
            println!("Found Field: {}", field);
        }
    }
}
```

Codetables
-------------
These are meant for use with the CSV files in this repository: https://github.com/wmo-im/GRIB2 This module serves to deserialize coded values and make output a little more human readable.
