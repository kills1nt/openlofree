//! Registry of supported models. Adding a model means one entry here and one file in `layouts/`.
use crate::error::Result;
use crate::layout::{parse_definition, ModelDef};

pub struct Model {
    pub id: &'static str,
    pub label: &'static str,
    /// True only for models tested on real hardware.
    pub verified: bool,
    json: &'static str,
}

impl Model {
    pub fn definition(&self) -> Result<ModelDef> {
        parse_definition(self.json)
    }
}

pub const MODELS: &[Model] = &[Model {
    id: "flow2-mac-84",
    label: "Flow 2 Mac 84",
    verified: true,
    json: include_str!("../../../layouts/flow2-mac-84.json"),
}];

pub fn by_product_id(product_id: u16) -> Option<(&'static Model, ModelDef)> {
    MODELS.iter().find_map(|m| {
        m.definition()
            .ok()
            .filter(|d| d.product_id == product_id)
            .map(|d| (m, d))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_registered_layout_parses() {
        for m in MODELS {
            m.definition().unwrap_or_else(|e| panic!("{}: {e}", m.id));
        }
    }

    #[test]
    fn flow2_mac_84_has_84_keys_in_16_unit_rows() {
        let def = MODELS[0].definition().unwrap();
        assert_eq!((def.rows, def.cols, def.product_id), (6, 15, 0x0028));
        assert_eq!(def.keys.len(), 84);
        for row in 0..def.rows {
            let width: f32 = def.keys.iter().filter(|k| k.row == row).map(|k| k.w).sum();
            assert_eq!(width, 16.0, "row {row}");
        }
    }

    #[test]
    fn looks_up_by_product_id() {
        assert_eq!(by_product_id(0x0028).unwrap().0.id, "flow2-mac-84");
        assert!(by_product_id(0xFFFF).is_none());
    }
}
