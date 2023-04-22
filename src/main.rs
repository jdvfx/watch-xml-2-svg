// read Apple watch XML and output
// the heart rate data to SVG
// split data per day and multiple days per SVG file

extern crate xml;
use xml::reader::{EventReader, XmlEvent};

use std::fs::File;
use std::io::BufReader;

use svg::node::element::path::Data;
use svg::node::element::{Path, Text};
use svg::Document;

#[derive(Debug, Default)]
struct WatchRecord {
    date: String,
    value: f64,
}

#[derive(Debug)]
struct HeartRecord {
    time_int: u32,
    date_str: String,
    time_norm: f64,
    bpm: f64,
}
fn date_reformat(strdate: &str, value: &f64) -> Option<HeartRecord> {
    // from: "2020-09-30 20:59:01 -0700" extract:
    // time_int  = 20200930205901 (to sort list)
    // date_str  = "2020-09-30"
    // time_norm = time/(3600*24)
    //
    let x: Vec<_> = strdate.split(' ').collect();
    let date = x.first().unwrap().replace('-', "").parse::<u32>().unwrap();
    //
    let t_ = x.get(1).unwrap().to_string();
    let mut time: Vec<u32> = t_.split(':').map(|x| x.parse().unwrap()).collect();
    time[0] *= 3600;
    time[1] *= 60;
    let t: u32 = time.iter().sum();
    let full_time_int = date * 100_000 + t;
    let maxsecs = 24 * 3600;
    let time_norm: f64 = t as f64 / maxsecs as f64;

    Some(HeartRecord {
        time_int: full_time_int,
        date_str: x.first().unwrap().to_string(),
        time_norm,
        bpm: *value,
    })
}

// pixel sizes for A4 page
const PAGE_WIDTH: f64 = 793.700_8;
const PAGE_HEIGHT: f64 = 1_122.519_7;
const DAYS_PER_PAGE: i32 = 20;

const DAY_SCALE: f64 = 0.13;
const VALUE_SCALE: f64 = 0.001;
const X_SCALE: f64 = PAGE_WIDTH;
const Y_SCALE: f64 = -400.0;

fn main() {
    // your XML there...
    let xml_path = "/home/bunker/projects/applewatch/apple_health_export/export.xml";
    let file = File::open(xml_path).unwrap();
    let file = BufReader::new(file);

    let parser = EventReader::new(file);

    let mut hrs: Vec<HeartRecord> = Vec::new();
    println!("reading XML data ...");
    for e in parser {
        match &e {
            Ok(XmlEvent::StartElement { attributes, .. }) => {
                let mut watch_record = WatchRecord::default();
                let mut is_heart_rate_record = false;
                for attrib in attributes {
                    let value = attrib.value.to_string();
                    let name = attrib.name.to_string();
                    let name_str = name.as_str();
                    match name_str {
                        "startDate" => watch_record.date = value.clone(),
                        "value" => {
                            let value_parsed: Result<f64, std::num::ParseFloatError> =
                                value.parse();
                            match value_parsed {
                                Ok(v) if { v > 10.0 && v < 300.0 } => watch_record.value = v,
                                _ => (),
                            }
                        }
                        "type" if { value.ends_with("HeartRate") } => is_heart_rate_record = true,
                        _ => (),
                    }
                }

                if is_heart_rate_record {
                    let nice_date = date_reformat(&watch_record.date, &watch_record.value);
                    if let Some(n) = nice_date {
                        hrs.push(n)
                    };
                }
            }
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
            _ => (),
        }
    }

    // sort heart records and remove duplicates
    hrs.sort_by(|a, b| a.time_int.cmp(&b.time_int));
    hrs.dedup_by(|a, b| a.time_int.eq(&b.time_int));

    let mut day = 0;
    let mut old_date = "".to_string();
    let mut filenum: i32 = 0;
    let mut paths: Vec<Box<dyn svg::node::Node>> = Vec::new();

    for hr in hrs {
        // x,y are coordinates on the SVG document
        let mut y = day as f64 * DAY_SCALE;

        // new day
        if hr.date_str != old_date {
            day += 1;
            old_date = hr.date_str.clone();

            // trace lines for 50,100,150 bpm and text for date
            let y_50 = (y + 50.0 * VALUE_SCALE) * Y_SCALE + PAGE_HEIGHT;
            let y_100 = (y + 100.0 * VALUE_SCALE) * Y_SCALE + PAGE_HEIGHT;
            let y_150 = (y + 150.0 * VALUE_SCALE) * Y_SCALE + PAGE_HEIGHT;

            let line = create_line((0.0, y_50), (PAGE_WIDTH, y_50), "green", 0.4);
            paths.push(Box::new(line));
            let line = create_line((0.0, y_100), (PAGE_WIDTH, y_100), "blue", 0.4);
            paths.push(Box::new(line));
            let line = create_line((0.0, y_150), (PAGE_WIDTH, y_150), "red", 0.4);
            paths.push(Box::new(line));
            let text = create_text((0.0, y_50 + 8.0), &hr.date_str);
            paths.push(Box::new(text));
        }

        let x = hr.time_norm * X_SCALE;
        y += hr.bpm * VALUE_SCALE;
        y *= Y_SCALE;
        y += PAGE_HEIGHT;

        // create points (lines of zero length)
        let point = create_line((x, y), (x, y), "black", 1.0);
        paths.push(Box::new(point));

        if day > DAYS_PER_PAGE {
            filenum += 1;
            let filename = format!("image_{}.svg", filenum);
            svg::save(&filename, &doc(paths.clone())).unwrap();
            paths.clear();
            println!("writing SVG: {}", &filename);
            day = 0;
        }
    }
}

fn create_line(from: (f64, f64), to: (f64, f64), color: &str, width: f64) -> Path {
    let data = Data::new().move_to(from).line_to(to);
    Path::new()
        .set("fill", "none")
        .set("stroke", color)
        .set("stroke-width", width)
        .set("stroke-linejoin", "square")
        .set("stroke-linecap", "round")
        .set("d", data)
}
fn create_text(position: (f64, f64), text: &str) -> Text {
    Text::new()
        .add(svg::node::Text::new(text))
        .set("x", position.0)
        .set("y", position.1)
        .set("text-anchor", "start")
        .set("font-family", "monospace")
        .set("alignment-baseline", "middle")
        .set("font-size", 8)
        .set("fill", "green")
}
fn doc(paths: Vec<Box<dyn svg::node::Node>>) -> Document {
    let document = Document::new()
        .set("width", PAGE_WIDTH)
        .set("height", PAGE_HEIGHT)
        .set("viewBox", (PAGE_WIDTH, PAGE_HEIGHT))
        .set("style", "background-color: white;");
    paths
        .into_iter()
        .fold(document, |document, path| document.add(path))
}
