use svg::node::element::path::Data;
use svg::node::element::{Path, Text};
use svg::Document;

use crate::conf::PAGE_HEIGHT;
use crate::conf::PAGE_WIDTH;

pub fn create_line(from: (f64, f64), to: (f64, f64), color: &str, width: f64) -> Path {
    let data = Data::new().move_to(from).line_to(to);
    Path::new()
        .set("fill", "none")
        .set("stroke", color)
        .set("stroke-width", width)
        .set("stroke-linejoin", "square")
        .set("stroke-linecap", "round")
        .set("d", data)
}
pub fn create_text(position: (f64, f64), text: &str) -> Text {
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
pub fn doc(paths: Vec<Box<dyn svg::node::Node>>) -> Document {
    let document = Document::new()
        .set("width", PAGE_WIDTH)
        .set("height", PAGE_HEIGHT)
        .set("viewBox", (PAGE_WIDTH, PAGE_HEIGHT))
        .set("style", "background-color: white;");
    paths
        .into_iter()
        .fold(document, |document, path| document.add(path))
}
