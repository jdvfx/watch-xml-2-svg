#![allow(dead_code, unused_variables, unused_assignments, unused_imports)]

// read Apple watch XML and output
// the heart rate data to SVG
// split data per day and multiple days per SVG file

extern crate xml;
use xml::reader::{EventReader, XmlEvent};

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::process::Output;

use svg::node::element::path::Data;
use svg::node::element::{Path, Text};
use svg::Document;

fn create_text(position: (f32, f32), text: &str) -> Text {
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
fn create_line(from: (f32, f32), to: (f32, f32), color: &str, width: f32) -> Path {
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
    date: String,
    value: f32,
}

struct HeartRecord {
    time_norm: f32,
    bpm_value: f32,
    date_string: String,
}

#[derive(Debug)]
struct NiceDate {
    full_time_int: u32,
    date_int: u32,
    time: u32,
    time_string: String,
    date_str: String,
    time_norm: f32,
    value: f32,
}
fn date_reformat(strdate: &str, value: &f32) -> Option<NiceDate> {
    let x: Vec<_> = strdate.split(' ').collect();
    let date = x.first().unwrap().replace('-', "").parse::<u32>().unwrap();
    let t_ = x.get(1).unwrap().to_string();
    let mut time: Vec<u32> = t_.split(':').map(|x| x.parse().unwrap()).collect();
    time[0] *= 3600;
    time[1] *= 60;
    let t: u32 = time.iter().sum();

    let full_time_int = date * 100_000 + t;

    let maxsecs = 24 * 3600;
    let time_norm: f32 = t as f32 / maxsecs as f32;

    Some(NiceDate {
        full_time_int,
        date_int: date,
        time: t,
        time_string: t_,
        date_str: x.first().unwrap().to_string(),
        time_norm,
        value: *value,
    })
}

// Simple string formatting
fn format_watch_record(wr: WatchRecord) -> HeartRecord {
    let ss = wr.date.split(' ').collect::<Vec<&str>>();
    let date = ss.first().unwrap().to_string();
    let time = ss.get(1).unwrap();
    let t = time
        .split(':')
        .filter_map(|x| x.parse().ok())
        .collect::<Vec<u32>>();
    let secs = t[0] * 3600 + t[1] * 60 + t[2];
    let maxsecs = 24 * 3600;
    let time_norm: f32 = secs as f32 / maxsecs as f32;
    HeartRecord {
        time_norm,
        bpm_value: wr.value,
        date_string: date,
    }
}

// pixel sizes for A4 page
const PAGE_WIDTH: f32 = 793.70079;
const PAGE_HEIGHT: f32 = 1122.51969;
const DAYS_PER_PAGE: i32 = 20;

const DAY_SCALE: f32 = 0.13;
const VALUE_SCALE: f32 = 0.001;
const X_SCALE: f32 = PAGE_WIDTH;
const Y_SCALE: f32 = -400.0;
const MAX_RECORD_READ: i32 = 308_500_000;

fn main() {
    // let path = "output.log";
    // let mut output = File::create(path).unwrap();
    //
    // let logfilename = format!("output_{}.log", filenum);
    // let mut output = File::create(logfilename).unwrap();

    // your XML there...
    let xml_path = "/home/bunker/projects/applewatch/apple_health_export/export.xml";
    let file = File::open(xml_path).unwrap();
    let file = BufReader::new(file);

    let parser = EventReader::new(file);
    let mut record_num: i32 = 0;
    let mut old_start_date = "---".to_string();

    let mut hrs: Vec<NiceDate> = Vec::new();
    for e in parser {
        //
        record_num += 1;

        match &e {
            Ok(XmlEvent::StartElement { attributes, .. }) => {
                let mut watch_record = WatchRecord::default();
                // let mut is_watch_record = false;
                let mut is_heart_rate_record = false;
                for attrib in attributes {
                    let value = attrib.value.to_string();
                    let name = attrib.name.to_string();
                    let name_str = name.as_str();
                    match name_str {
                        "startDate" => watch_record.date = value.clone(),
                        "value" => {
                            let value_parsed: Result<f32, std::num::ParseFloatError> =
                                value.parse();
                            match value_parsed {
                                Ok(v) if { v > 10.0 && v < 300.0 } => watch_record.value = v,
                                _ => (),
                            }
                        }
                        // "sourceName" if { value.ends_with("Watch") } => is_watch_record = true,
                        "type" if { value.ends_with("HeartRate") } => is_heart_rate_record = true,
                        _ => (),
                    }
                }

                // let sd = watch_record.start_date.clone();
                // let date_: Vec<_> = sd.split(' ').collect();
                // let date__ = date_.first().unwrap().to_string();

                // if is_watch_record && is_heart_rate_record {
                if is_heart_rate_record {
                    // let sd = watch_record.start_date.clone();
                    // let date_: Vec<_> = sd.split(' ').collect();
                    let nice_date = date_reformat(&watch_record.date, &watch_record.value);
                    match nice_date {
                        Some(n) => hrs.push(n),
                        None => (),
                    }

                    //     let writeline = format!("{} {} {} {}", &_gy, &v, &date, &x);
                    //     writeln!(output, "{}", writeline).ok();
                    //
                    //     let mut y = day as f32 * DAY_SCALE;
                    //     if date__ != old_start_date {
                    //         println!("date__:{} old_startdate:{}", &date__, &old_start_date);
                    //         old_start_date = date__.clone();
                    //         day += 1;
                    //
                    //         // trace lines for 50,100,150 bpm
                    //
                    //         let y_50 = (y + 50.0 * VALUE_SCALE) * Y_SCALE + PAGE_HEIGHT;
                    //         let y_100 = (y + 100.0 * VALUE_SCALE) * Y_SCALE + PAGE_HEIGHT;
                    //         let y_150 = (y + 150.0 * VALUE_SCALE) * Y_SCALE + PAGE_HEIGHT;
                    //
                    //         let line = create_line((0.0, y_50), (PAGE_WIDTH, y_50), "green", 0.4);
                    //         paths.push(Box::new(line));
                    //         let line = create_line((0.0, y_100), (PAGE_WIDTH, y_100), "blue", 0.4);
                    //         paths.push(Box::new(line));
                    //         let line = create_line((0.0, y_150), (PAGE_WIDTH, y_150), "red", 0.4);
                    //         paths.push(Box::new(line));
                    //
                    //         let text = create_text((0.0, y_50 + 8.0), &date);
                    //         paths.push(Box::new(text));
                    //     }
                    //
                    //     // y = day as f32 * DAY_SCALE;
                    //     y += v * VALUE_SCALE;
                    //     x *= X_SCALE;
                    //     y *= Y_SCALE;
                    //     y += PAGE_HEIGHT;
                    //
                    //     // create a line of zero length
                    //     let point = create_line((x, y), (x, y), "black", 1.0);
                    //     paths.push(Box::new(point));
                    //
                    //     let writeline = format!("point: {} {}\n", &x, &y);
                    //     writeln!(output, "{}", writeline).ok();
                    //
                    //     // if day > DAYS_PER_PAGE || (record_num > MAX_RECORD_READ) {
                }
            }
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
            _ => (),
        }
    }

    let mut old_date = 0;
    let mut day = 0;

    hrs.sort_by(|a, b| a.full_time_int.cmp(&b.full_time_int));
    hrs.dedup_by(|a, b| a.time.eq(&b.time));

    // for i in hrs {
    //     println!("> {:?}", i);
    // }

    let mut filenum: i32 = 0;
    let mut paths: Vec<Box<dyn svg::node::Node>> = Vec::new();
    // let mut output = File::create(logfilename).unwrap();
    for hr in hrs {
        let mut y = day as f32 * DAY_SCALE;

        // let writeline = format!("{} : {} : {}", hr.date_int, hr.time, hr.time_string);
        // writeln!(output, "{}", writeline).ok();

        if hr.date_int != old_date {
            day += 1;
            old_date = hr.date_int;

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

            let text = create_text((0.0, y_50 + 8.0), &hr.date_str);
            paths.push(Box::new(text));
        }
        let x = hr.time_norm * X_SCALE;
        y += hr.value * VALUE_SCALE;
        y *= Y_SCALE;
        y += PAGE_HEIGHT;

        // create a line of zero length
        let point = create_line((x, y), (x, y), "black", 1.0);
        paths.push(Box::new(point));

        if day > DAYS_PER_PAGE {
            day = 0;
            filenum += 1;

            let filename = format!("image_{}.svg", filenum);
            // let d = doc(&paths); //.clone());
            svg::save(&filename, &doc(paths.clone())).unwrap();
            paths.clear();

            // let logfilename = format!("output_{}.log", filenum);
            // output = File::create(logfilename).unwrap();
        }

        // if record_num > MAX_RECORD_READ {
        //     break 'records;
        // }
    }

    // write the next file is paths not empty
    // if paths.len() > 0 {
    //     filenum += 1;
    //     let filename = format!("image_{}.svg", filenum);
    //     let d = doc(paths);
    //     svg::save(&filename, &d).unwrap();
    //     // paths.clear();
    // }
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
