//! Parses a VIA-style definition: name, ids, matrix size and KLE-style key geometry.
use serde_json::Value;

use crate::error::{Error, Result};

#[derive(Debug, Clone, PartialEq)]
pub struct KeyDef {
    pub row: u8,
    pub col: u8,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModelDef {
    pub name: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub rows: u8,
    pub cols: u8,
    pub keys: Vec<KeyDef>,
}

fn bad(msg: impl Into<String>) -> Error {
    Error::Layout(msg.into())
}

fn hex16(v: &Value, field: &str) -> Result<u16> {
    let s = v
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| bad(format!("missing {field}")))?;
    u16::from_str_radix(s.trim_start_matches("0x").trim_start_matches("0X"), 16)
        .map_err(|_| bad(format!("bad {field}: {s}")))
}

fn matrix_dim(v: &Value, field: &str) -> Result<u8> {
    v.pointer(&format!("/matrix/{field}"))
        .and_then(Value::as_u64)
        .and_then(|n| u8::try_from(n).ok())
        .ok_or_else(|| bad(format!("missing matrix.{field}")))
}

pub fn parse_definition(json: &str) -> Result<ModelDef> {
    let v: Value = serde_json::from_str(json).map_err(|e| bad(e.to_string()))?;
    let name = v
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| bad("missing name"))?
        .to_string();
    let (rows, cols) = (matrix_dim(&v, "rows")?, matrix_dim(&v, "cols")?);
    let kle = v
        .pointer("/layouts/keymap")
        .and_then(Value::as_array)
        .ok_or_else(|| bad("missing layouts.keymap"))?;

    let mut keys = Vec::new();
    let mut y = 0f32;
    for row in kle {
        let items = row
            .as_array()
            .ok_or_else(|| bad("keymap row is not an array"))?;
        let (mut x, mut w, mut h) = (0f32, 1f32, 1f32);
        for item in items {
            match item {
                Value::Object(props) => {
                    let num = |k: &str| props.get(k).and_then(Value::as_f64).map(|n| n as f32);
                    x += num("x").unwrap_or(0.0);
                    y += num("y").unwrap_or(0.0);
                    w = num("w").unwrap_or(w);
                    h = num("h").unwrap_or(h);
                }
                Value::String(label) => {
                    // "row,col", optionally followed by "\n<decal>" which we ignore.
                    let pos = label.lines().next().unwrap_or("");
                    let (r, c) = pos
                        .split_once(',')
                        .ok_or_else(|| bad(format!("bad key label {label:?}")))?;
                    let parse = |s: &str| {
                        s.trim()
                            .parse::<u8>()
                            .map_err(|_| bad(format!("bad key label {label:?}")))
                    };
                    let (r, c) = (parse(r)?, parse(c)?);
                    if r >= rows || c >= cols {
                        return Err(bad(format!("key {r},{c} outside {rows}x{cols} matrix")));
                    }
                    keys.push(KeyDef {
                        row: r,
                        col: c,
                        x,
                        y,
                        w,
                        h,
                    });
                    x += w;
                    w = 1.0;
                    h = 1.0;
                }
                _ => return Err(bad("unexpected item in keymap row")),
            }
        }
        y += 1.0;
    }

    Ok(ModelDef {
        name,
        vendor_id: hex16(&v, "vendorId")?,
        product_id: hex16(&v, "productId")?,
        rows,
        cols,
        keys,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const TINY: &str = r#"{
        "name": "Tiny", "vendorId": "0x388d", "productId": "0x00AB",
        "matrix": {"rows": 2, "cols": 3},
        "layouts": {"keymap": [
            [{"w": 1.5}, "0,0", "0,1", {"x": 0.5}, "0,2"],
            [{"y": 0.25}, "1,0\nDecal", {"w": 2}, "1,1"]
        ]}
    }"#;

    #[test]
    fn parses_ids_and_matrix() {
        let m = parse_definition(TINY).unwrap();
        assert_eq!(
            (m.name.as_str(), m.vendor_id, m.product_id, m.rows, m.cols),
            ("Tiny", 0x388D, 0x00AB, 2, 3)
        );
    }

    #[test]
    fn computes_geometry() {
        let k = parse_definition(TINY).unwrap().keys;
        assert_eq!(
            k[0],
            KeyDef {
                row: 0,
                col: 0,
                x: 0.0,
                y: 0.0,
                w: 1.5,
                h: 1.0
            }
        );
        assert_eq!(k[1].x, 1.5);
        assert_eq!(k[2].x, 3.0); // 1.5 + 1.0 + 0.5 gap
        assert_eq!(
            k[3],
            KeyDef {
                row: 1,
                col: 0,
                x: 0.0,
                y: 1.25,
                w: 1.0,
                h: 1.0
            }
        );
        assert_eq!(k[4].w, 2.0);
    }

    #[test]
    fn rejects_key_outside_matrix() {
        let bad_json = TINY.replace("\"0,2\"", "\"0,9\"");
        assert!(matches!(parse_definition(&bad_json), Err(Error::Layout(_))));
    }

    #[test]
    fn rejects_missing_fields() {
        assert!(matches!(parse_definition("{}"), Err(Error::Layout(_))));
        assert!(matches!(
            parse_definition("not json"),
            Err(Error::Layout(_))
        ));
    }
}
