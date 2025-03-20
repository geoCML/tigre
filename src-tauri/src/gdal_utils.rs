use gdal::{Dataset, DriverManager};
use gdal::spatial_ref::SpatialRef;
use gdal::vector::{LayerOptions, LayerAccess, Geometry};
use gdal_sys::GDALVectorTranslate;
use std::fs;
use std::fs::File;
use std::ffi::CString;
use std::ptr::{null, null_mut};
use std::io::BufRead;

pub async fn generic_to_postgis_layer(
    dataset: Dataset,
    mut pgsql_client: postgres::Client,
    name: &str
) {
    let mut fields: Vec<String> = vec![];
    let mut geometries: Vec<Geometry> = vec![];
    let mut geometry_type = String::new();

    unsafe {
        let mut raw_dataset: Vec<gdal_sys::GDALDatasetH> = vec![dataset.c_dataset()];

        GDALVectorTranslate(
            //null(),
            //CString::new(state.lock().unwrap().pgsql_connection.clone()).unwrap().as_ptr(),
            CString::new(format!("/tmp/{}.csv", name)).unwrap().as_ptr(),
            null_mut(),
            1,
            raw_dataset.as_mut_ptr(),
            null(),
            null_mut(),
        );
    };

    dataset.layers()
        .for_each(| mut lyr | {
            // COLLECT LAYER GEOMETRIES
            lyr.features()
                .for_each(| feature | {
                    match feature.geometry() {
                        Some(geometry) => geometries.push(geometry.clone()),
                        _ => ()
                    }
                });

            // COLLECT FIELD TYPES
            for geometry in geometries.clone() {
                if geometry_type == "" {
                    geometry_type = geometry.geometry_name();
                    continue;
                }

                if geometry_type != geometry.geometry_name() {
                    panic!("ERROR! Some features in layer have mismatched geometries. Expected {}, but got {}.", geometry_type, geometry.geometry_name());
                }
            }

            lyr.defn()
                .fields()
                .for_each(| f | {
                    let pg_field_type = match f.field_type() {
                        8 => "bytea",
                        9 => "date",
                        11 => "timestamp",
                        0 => "integer",
                        12 => "bigint",
                        13 => "bigint[]",
                        1 => "integer[]",
                        2 => "numeric",
                        3 => "numeric[]",
                        4 => "text",
                        5 => "text[]",
                        10 => "time",
                        6 => "text",
                        7 => "text[]",
                        _ => "text"
                    };
                    fields.push(format!("{} {}", f.name(), pg_field_type));
                });
        });

    // CREATE TABLE
    let create_layer_result = pgsql_client.execute(
        format!(
            "CREATE TABLE {} ({}, geom geometry)",
            name,
            fields.join(", ")
        )
        .as_str(),
        &[],
    );
    match create_layer_result {
        Ok(_) => (),
        Err(_) => {
            let _ = fs::remove_file(format!("/tmp/{}.csv", &name));
            panic!("ERROR! Failed to create layer in database.");
        }
    };

    // SET GEOMETRY TYPE
    let set_geometry_query = match geometries[0].spatial_ref() {
        Some(val) => {
            let srid = match val.auth_code() {
                Ok(val) => val,
                Err(_) => 4326
            };

            format!(
                "ALTER TABLE \"{}\" ALTER COLUMN geom TYPE Geometry({}, {})",
                name,
                geometry_type,
                srid
            )
        },
        None => format!(
            "ALTER TABLE \"{}\" ALTER COLUMN geom TYPE Geometry({})",
            name, geometry_type
        ),
    };

    let set_geometry_result = pgsql_client.execute(set_geometry_query.as_str(), &[]);
    match set_geometry_result {
        Ok(_) => (),
        Err(_) => {
            panic!("ERROR! Failed to set geometry information.");
        }
    };

    // COPY FROM CSV -> NEW PGSQL TABLE
    let csv_file = File::open_buffered(format!("/tmp/{}.csv", name).as_str()).unwrap();
    let mut cols = String::new();
    let mut queries: Vec<String> = vec![];

    for (line, i) in csv_file.lines().zip(1..) {
        if i == 1 {
            cols = line
                .unwrap()
                .clone()
                .as_str()
                .split(",")
                .collect::<Vec<_>>()
                .join(", ");

            continue;
        }
        let the_line = line
            .unwrap()
            .clone()
            .as_str()
            .split(",")
            .collect::<Vec<_>>()
            .into_iter()
            .map(|item| {
                return format!("\'{}\'", item);
            })
            .collect::<Vec<_>>()
            .join(", ");

        queries.push(format!(
            "INSERT INTO {} ({}, geom) VALUES ({}, '{}')",
            name,
            cols,
            the_line,
            geometries[i - 2].wkt().unwrap()
        ))
    }

    let insert_result = pgsql_client.batch_execute(queries.join(";").as_str());
    match insert_result {
        Ok(_) => (),
        Err(_) => {
            panic!("ERROR! Failed to load raw data into database table.");
        }
    }

    let _ = fs::remove_file(format!("/tmp/{}.csv", name));
}

pub async fn generic_to_gpkg(dataset_name: &str) -> Result<Vec<String>, ()> {
    let mut gpkgs = vec![];

    match Dataset::open(dataset_name) { 
        Ok(dataset) => {
            let driver = DriverManager::get_driver_by_name("GPKG").unwrap();
            for mut layer in dataset.layers() {
                let name = layer.name();
                let long_name = format!("public.{}", name);
                let mut gpkg_dataset = driver.create_vector_only(format!("/tmp/tigre/{}.gpkg", long_name)).unwrap();

                let layer_srs = SpatialRef::from_epsg(4326).unwrap();

                let layer_geom = match layer.features().collect::<Vec<_>>().first() {
                    Some(layer_geom) => layer_geom.geometry().unwrap().geometry_type(),
                    None => {
                        panic!("ERROR! Layer '{}' has no features.", long_name);
                    }
                };

                let layer_options = LayerOptions {
                    name: name.as_str(),
                    srs: Some(&layer_srs),
                    ty: layer_geom,
                    options: Some(&["GEOMETRY_NAME=geom", "FID=fid"]),
                };
                let mut gpkg_layer = gpkg_dataset.create_layer(layer_options).unwrap();

                for feature in layer.features() {
                    gpkg_layer.create_feature(feature.geometry().unwrap().clone()).unwrap();
                }

                gpkgs.push(format!("/tmp/tigre/{}.gpkg", long_name));
            }
        },
        Err(err) => panic!("Failed to open database: {}", err)
    }
    Ok(gpkgs)
}

pub async fn postgis_layer_to_gpkg(
    name: &str,
    schema: &str,
    gdal_pgsql_connection: String
) {
    let long_name = format!("{}.{}", schema, name);

    std::env::set_var("GDAL_SKIP", "GNMFile,GNMDatabase,PostGISRaster"); // This forces GDAL to use the PostgreSQL Driver
    let postgis_dataset = Dataset::open(gdal_pgsql_connection).unwrap();
    let mut postgis_layer = postgis_dataset.layer_by_name(name).unwrap();

    let driver = DriverManager::get_driver_by_name("GPKG").unwrap();
    let mut gpkg_dataset = driver.create_vector_only(format!("/tmp/tigre/{}.gpkg", long_name)).unwrap();

    let layer_srs = SpatialRef::from_epsg(4326).unwrap();

    let layer_geom = match postgis_layer.features().collect::<Vec<_>>().first() {
        Some(layer_geom) => layer_geom.geometry().unwrap().geometry_type(),
        None => {
            panic!("ERROR! Layer '{}' has no features.", long_name);
        }
    };

    let layer_options = LayerOptions {
        name,
        srs: Some(&layer_srs),
        ty: layer_geom,
        options: None,
    };
    let mut gpkg_layer = gpkg_dataset.create_layer(layer_options).unwrap();

    for feature in postgis_layer.features() {
        gpkg_layer.create_feature(feature.geometry().unwrap().clone()).unwrap();
    }
}
