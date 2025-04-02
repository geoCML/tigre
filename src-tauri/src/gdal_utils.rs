use gdal::spatial_ref::SpatialRef;
use gdal::vector::{Geometry, LayerAccess, LayerOptions};
use gdal::{Dataset, DriverManager};
use geozero::geojson::GeoJson;
use geozero::ToSvg;
use std::fs::File;
use std::io::Write;

pub async fn generic_to_svg(dataset: Dataset) -> Result<(), String> {
    // Create a spatial reference for Web Mercator
    let target_srs = SpatialRef::from_epsg(3857).unwrap();
    
    for mut layer in dataset.layers() {
        let mut svg_file = File::create(format!("/tmp/tigre/{}.svg", layer.name())).map_err(|e| e.to_string())?;
        writeln!(svg_file, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>")
            .map_err(|e| e.to_string())?;

        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;
        let mut svg_paths = vec![];

        for feature in layer.features() {
            if let Some(geometry) = feature.geometry() {
                let mut reprojected_geom = geometry.clone();

                match reprojected_geom.spatial_ref() {
                    Some(_srs) => {
                        reprojected_geom.transform_to(&target_srs).map_err(|e| e.to_string())?;
                    }
                    None => {
                        reprojected_geom.set_spatial_ref(target_srs.clone());
                    }
                }
                
                let geojson_str = reprojected_geom.json().map_err(|e| e.to_string())?;
                let geojson = GeoJson(&geojson_str);
                let envelope = reprojected_geom.envelope();

                min_x = min_x.min(envelope.MinX);
                min_y = min_y.min(envelope.MinY);
                max_x = max_x.max(envelope.MaxX);
                max_y = max_y.max(envelope.MaxY);

                let mut svg_path = geojson.to_svg().map_err(|e| e.to_string())?;
                svg_path = svg_path.replace("<path d=", "<path style=\"fill:#cccccc;stroke:#000000;stroke-width:0.01\" d=");

                svg_paths.push(svg_path);
            }
        }
        
        // Calculate width and height for viewBox (ensure they're positive)
        let width = max_x - min_x;
        let height = max_y - min_y;
        
        writeln!(
            svg_file,
            "<svg id=\"{}\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"{} {} {} {}\" data-bounds=\"{} {} {} {}\">",
            layer.name(), 
            min_x,
            min_y, width, height,
            min_x,  // west
            max_y,  // north
            max_x,  // east
            min_y   // south
        ).map_err(|e| e.to_string())?;

        svg_paths.iter().for_each(|path| {
            let path = format!("<g transform=\"matrix(1, 0, 0, -1, 0, {})\">{}</g>", height, path);
            let _ = writeln!(svg_file, "{}", path).map_err(|e| e.to_string());
        });

        writeln!(
            svg_file,
            "</svg>"
        ).map_err(|e| e.to_string())?;
    }

    Ok(())
}

pub async fn generic_to_postgis_layer(
    dataset: Dataset,
    mut pgsql_client: postgres::Client,
    name: &str,
) {
    let mut fields: Vec<String> = vec![];
    let mut geometries: Vec<Geometry> = vec![];
    let mut geometry_type = String::new();

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
    match pgsql_client.execute(
        format!(
            "CREATE TABLE {} ({}, geom geometry)",
            name,
            fields.join(", ").to_lowercase()
        )
        .as_str(),
        &[],
    ) {
        Ok(_) => (),
        Err(_) => {
            panic!("ERROR! Failed to create layer in database.");
        }
    };

    // SET GEOMETRY TYPE
    match pgsql_client.execute(
        format!(
            "ALTER TABLE \"{}\" ALTER COLUMN geom TYPE Geometry({}, 0)",
            name, geometry_type
        )
        .as_str(),
        &[],
    ) {
        Ok(_) => (),
        Err(_) => {
            panic!("ERROR! Failed to set geometry information.");
        }
    };

    // COPY FROM GENERIC DATASET -> NEW PGSQL TABLE
    dataset.layers().for_each(|mut lyr| {
        let mut queries: Vec<String> = vec![];

        let cols = lyr
            .defn()
            .fields()
            .map(|field| {
                return format!("\"{}\"", field.name());
            })
            .collect::<Vec<String>>();

        let mut i = 0;
        lyr.features().for_each(|feature| {
            let values = feature
                .fields()
                .filter(|field| field.0 != "geom")
                .map(|field| {
                    return match field.1 {
                        Some(gdal::vector::FieldValue::StringValue(val)) => format!("\'{}\'", val),
                        Some(gdal::vector::FieldValue::IntegerValue(val)) => format!("{}", val),
                        Some(gdal::vector::FieldValue::DateValue(val)) => format!("\'{}\'", val),
                        Some(gdal::vector::FieldValue::RealValue(val)) => format!("{}", val),
                        Some(gdal::vector::FieldValue::Integer64Value(val)) => format!("{}", val),
                        Some(gdal::vector::FieldValue::Integer64ListValue(val)) => format!(
                            "\'{}\'",
                            val.iter()
                                .map(|v| v.to_string())
                                .collect::<Vec<String>>()
                                .join(", ")
                        ),
                        Some(gdal::vector::FieldValue::IntegerListValue(val)) => format!(
                            "\'{}\'",
                            val.iter()
                                .map(|v| v.to_string())
                                .collect::<Vec<String>>()
                                .join(", ")
                        ),
                        Some(gdal::vector::FieldValue::RealListValue(val)) => format!(
                            "\'{}\'",
                            val.iter()
                                .map(|v| v.to_string())
                                .collect::<Vec<String>>()
                                .join(", ")
                        ),
                        Some(gdal::vector::FieldValue::DateTimeValue(val)) => {
                            format!("\'{}\'", val)
                        }
                        Some(gdal::vector::FieldValue::StringListValue(val)) => {
                            format!("\'{}\'", val.join(", "))
                        }
                        None => "NULL".to_string(),
                    };
                })
                .collect::<Vec<String>>()
                .join(", ");

            queries.push(format!(
                "INSERT INTO {} ({}, \"geom\") VALUES ({}, '{:?}')",
                name,
                cols.join(", ").to_lowercase(),
                values,
                geometries[i]
            ));
            i += 1;
        });

        queries.iter().for_each(|query| {
            let insert_result = pgsql_client.batch_execute(query.as_str());
            match insert_result {
                Ok(_) => (),
                Err(err) => {
                    println!(
                        "ERROR! Failed to load raw data into database table: {}",
                        err
                    );
                }
            }
        });
    });

    let _ = pgsql_client.execute(
        format!("COMMENT ON TABLE public.{} IS '{{\"fillColor\": \"#d18a69\", \"fillOpacity\": 0.5, \"color\": \"#d18a69\", \"weight\": 1}}'", name).as_str(),
        &[]
    );
}

pub async fn generic_to_gpkg(dataset: Dataset) -> Result<Vec<String>, ()> {
    let mut gpkgs = vec![];

    let driver = DriverManager::get_driver_by_name("GPKG").unwrap();
    for mut layer in dataset.layers() {
        let name = layer.name();
        let long_name = format!("public.{}", name);
        let mut gpkg_dataset = driver
            .create_vector_only(format!("/tmp/tigre/{}.gpkg", long_name))
            .unwrap();

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
            gpkg_layer
                .create_feature(feature.geometry().unwrap().clone())
                .unwrap();
        }

        gpkgs.push(format!("/tmp/tigre/{}.gpkg", long_name));
    }

    Ok(gpkgs)
}

pub async fn postgis_layer_to_gpkg(name: &str, schema: &str, gdal_pgsql_connection: String) {
    let long_name = format!("{}.{}", schema, name);

    std::env::set_var("GDAL_SKIP", "GNMFile,GNMDatabase,PostGISRaster"); // This forces GDAL to use the PostgreSQL Driver
    let postgis_dataset = Dataset::open(gdal_pgsql_connection).unwrap();
    let mut postgis_layer = postgis_dataset.layer_by_name(name).unwrap();

    let driver = DriverManager::get_driver_by_name("GPKG").unwrap();
    let mut gpkg_dataset = driver
        .create_vector_only(format!("/tmp/tigre/{}.gpkg", long_name))
        .unwrap();

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
        gpkg_layer
            .create_feature(feature.geometry().unwrap().clone())
            .unwrap();
    }
}
