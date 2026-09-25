use quick_xml::{Reader, events::Event};

use crate::error::{AppError, Result};

use super::{Bounds, UiNode, UiTree};

pub struct UiParser;

impl UiParser {
    pub fn parse(xml: &str) -> Result<UiTree> {
        let mut reader = Reader::from_str(xml);

        let mut nodes = Vec::new();

        loop {
            match reader.read_event() {
                Ok(Event::Start(event)) | Ok(Event::Empty(event)) => {
                    if event.name().as_ref() == "node" {
                        let mut index = nodes.len();

                        let mut text = None;
                        let mut resource_id = None;
                        let mut class_name = None;
                        let mut package = None;
                        let mut content_desc = None;
                        let mut clickable = false;
                        let mut enabled = false;
                        let mut bounds = None;

                        for attr in event.attributes() {
                            let attr = attr.map_err(|e| AppError::UiParse(e.to_string()))?;

                            let key =
                                String::from_utf8_lossy(attr.key.as_ref().as_bytes()).to_string();

                            let value =
                                String::from_utf8_lossy(attr.value.as_ref().as_bytes()).to_string();

                            match key.as_str() {
                                "index" => {
                                    index = value.parse().unwrap_or(nodes.len());
                                }

                                "text" => {
                                    if !value.is_empty() {
                                        text = Some(value);
                                    }
                                }

                                "resource-id" => {
                                    if !value.is_empty() {
                                        resource_id = Some(value);
                                    }
                                }

                                "class" => {
                                    if !value.is_empty() {
                                        class_name = Some(value);
                                    }
                                }

                                "package" => {
                                    if !value.is_empty() {
                                        package = Some(value);
                                    }
                                }

                                "content-desc" => {
                                    if !value.is_empty() {
                                        content_desc = Some(value);
                                    }
                                }

                                "clickable" => {
                                    clickable = value == "true";
                                }

                                "enabled" => {
                                    enabled = value == "true";
                                }

                                "bounds" => {
                                    bounds = Some(parse_bounds(&value)?);
                                }

                                _ => {}
                            }
                        }

                        let bounds = bounds.unwrap_or(Bounds {
                            left: 0,
                            top: 0,
                            right: 0,
                            bottom: 0,
                        });

                        nodes.push(UiNode {
                            index,
                            text,
                            resource_id,
                            class_name,
                            package,
                            content_desc,
                            clickable,
                            enabled,
                            bounds,
                        });
                    }
                }

                Ok(Event::Eof) => break,

                Err(error) => {
                    return Err(AppError::UiParse(error.to_string()));
                }

                _ => {}
            }
        }

        Ok(UiTree::new(nodes))
    }
}

fn parse_bounds(value: &str) -> Result<Bounds> {
    // [100,500][400,600]

    let value = value.trim();

    let parts: Vec<&str> = value.split("][").collect();

    if parts.len() != 2 {
        return Err(AppError::UiParse(format!("invalid bounds: {}", value)));
    }

    let left_top = parts[0].trim_start_matches('[');

    let right_bottom = parts[1].trim_end_matches(']');

    let p1: Vec<&str> = left_top.split(',').collect();

    let p2: Vec<&str> = right_bottom.split(',').collect();

    if p1.len() != 2 || p2.len() != 2 {
        return Err(AppError::UiParse(format!("invalid bounds: {}", value)));
    }

    let left = p1[0]
        .parse::<i32>()
        .map_err(|_| AppError::UiParse(format!("invalid left: {}", p1[0])))?;

    let top = p1[1]
        .parse::<i32>()
        .map_err(|_| AppError::UiParse(format!("invalid top: {}", p1[1])))?;

    let right = p2[0]
        .parse::<i32>()
        .map_err(|_| AppError::UiParse(format!("invalid right: {}", p2[0])))?;

    let bottom = p2[1]
        .parse::<i32>()
        .map_err(|_| AppError::UiParse(format!("invalid bottom: {}", p2[1])))?;

    Ok(Bounds {
        left,
        top,
        right,
        bottom,
    })
}
