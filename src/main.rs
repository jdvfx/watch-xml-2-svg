// #![allow(dead_code, unused_variables, unused_assignments, unused_imports)]

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

// Apple Watch record, value=heart_rate
#[derive(Debug, Default)]
struct WatchRecord {
    start_date: String,
    value: f64,
}

// Simple string formatting
fn format_watch_record(wr: WatchRecord) -> (f64, f64, f64, String) {
    //
    let ss = wr.start_date.split(' ').collect::<Vec<&str>>();

    let date = ss.first().unwrap();
    let time = ss.get(1).unwrap();
    let t = time
        .split(':')
        .filter_map(|x| x.parse().ok())
        .collect::<Vec<u32>>();

    let d = date
        .split('-')
        .filter_map(|x| x.parse().ok())
        .collect::<Vec<u32>>();

    let secs = t[0] * 3600 + t[1] * 60 + t[2];
    let maxsecs = 24 * 3600;
    let time_norm: f64 = secs as f64 / maxsecs as f64;
    let y: String = format!("{}{}{}", d[0], d[1], d[2]);
    let y_: u32 = y.parse().unwrap_or(0);

    (time_norm, y_ as f64, wr.value, format!("{}", date))
}

// pixel sizes for A4 page
const PAGE_WIDTH: f64 = 793.70079;
const PAGE_HEIGHT: f64 = 1122.51969;
const DAYS_PER_PAGE: i32 = 20;

const DAY_SCALE: f64 = 0.13;
const VALUE_SCALE: f64 = 0.001;
const X_SCALE: f64 = PAGE_WIDTH;
const Y_SCALE: f64 = -400.0;
const MAX_RECORD_READ: i32 = 208500;

fn main() {
    //
    let mut filenum: i32 = 0;
    let mut paths: Vec<Box<dyn svg::node::Node>> = Vec::new();

    // your XML there...
    let xml_path = "/home/bunker/projects/applewatch/apple_health_export/export.xml";
    let file = File::open(xml_path).unwrap();
    let file = BufReader::new(file);

    let parser = EventReader::new(file);
    let mut record_num: i32 = 0;
    let mut day: i32 = 0;
    let mut old_start_date = "---".to_string();

    'records: for e in parser {
        //
        record_num += 1;

        match &e {
            Ok(XmlEvent::StartElement { attributes, .. }) => {
                let mut watch_record = WatchRecord::default();
                let mut is_watch_record = false;
                for i in attributes {
                    let v = i.value.to_string();
                    let n = i.name.to_string();
                    let nn = n.as_str();
                    match nn {
                        "startDate" => watch_record.start_date = v.clone(),
                        "value" => watch_record.value = v.clone().parse().unwrap_or_default(),
                        "sourceName" if { v.ends_with("Watch") } => is_watch_record = true,
                        _ => (),
                    }
                }

                if is_watch_record {
                    let (mut x, mut y, v, date) = format_watch_record(watch_record);

                    let yy = y.clone().to_string();
                    if yy != old_start_date {
                        println!("> filenum: {}, date: {}", filenum, &date);
                        old_start_date = yy;

                        y = day as f64 * DAY_SCALE;

                        // trace lines for 50,100,150 bpm
                        let y_50 = (y + 50.0 * VALUE_SCALE) * Y_SCALE + PAGE_HEIGHT;
                        let y_100 = (y + 100.0 * VALUE_SCALE) * Y_SCALE + PAGE_HEIGHT;
                        let y_150 = (y + 150.0 * VALUE_SCALE) * Y_SCALE + PAGE_HEIGHT;

                        let line = create_line((0.0, y_50), (PAGE_WIDTH, y_50), "green", 0.4);
                        paths.push(Box::new(line));
                        let line = create_line((0.0, y_100), (PAGE_WIDTH, y_100), "blue", 0.4);
                        paths.push(Box::new(line));
                        let line = create_line((0.0, y_150), (PAGE_WIDTH, y_150), "red", 0.4);
                        paths.push(Box::new(line));

                        let text = create_text((0.0, y_50), &date);
                        paths.push(Box::new(text));
                        day += 1;
                    }

                    y = day as f64 * DAY_SCALE;
                    y += v * VALUE_SCALE;
                    x *= X_SCALE;
                    y *= Y_SCALE;
                    y += PAGE_HEIGHT;

                    // create a line of zero length
                    let point = create_line((x, y), (x, y), "black", 1.0);
                    paths.push(Box::new(point));

                    if day > DAYS_PER_PAGE || (record_num > MAX_RECORD_READ) {
                        day = 0;
                        let filename = format!("image_{}.svg", filenum);
                        filenum += 1;
                        let d = doc(paths);
                        svg::save(&filename, &d).unwrap();
                        paths = Vec::new();
                    }
                    if record_num > MAX_RECORD_READ {
                        break 'records;
                    }
                }
            }
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
            _ => {}
        }
    }
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
