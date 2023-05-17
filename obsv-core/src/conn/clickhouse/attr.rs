//! Attributes

use clickhouse_client::orm::{DbType, DbValue};

use crate::attr::{Attr, AttrValue, Attrs};

impl DbType for Attrs {
    // NB: an attribute value is stored as a string
    const TYPE: &'static str = "Map(String, String)";
}

impl DbValue for Attrs {
    // NB: we store the attribute value with the RON serializer
    fn to_sql_str(&self) -> String {
        format!(
            "{{{}}}",
            self.0
                .iter()
                .map(|attr| {
                    let key = &attr.key;
                    let value_str = match ron::to_string(&attr.value) {
                        Ok(ok) => ok,
                        Err(err) => {
                            panic!("Failed to serialize attr value: {}", err);
                        }
                    };
                    format!("'{}': '{}'", key, value_str)
                })
                .collect::<Vec<_>>()
                .join(",")
        )
    }

    fn from_sql_str(s: &str) -> Result<Self, String> {
        let s = match s.strip_prefix('{') {
            Some(s) => s,
            None => return Err("Invalid map".to_string()),
        };
        let s = match s.strip_suffix('}') {
            Some(s) => s,
            None => return Err("Invalid map".to_string()),
        };
        let attrs = s
            .split(',')
            .filter_map(|part| {
                if part.is_empty() {
                    return None;
                }
                match part.split_once(':') {
                    Some((key, val)) => {
                        let key = strip_string_quotes(key.trim());
                        let val = strip_string_quotes(val.trim());
                        let attr_val =
                            ron::from_str::<AttrValue>(val).expect("Invalid attribute db value");
                        Some(Ok(Attr::new(key, attr_val)))
                    }
                    None => Some(Err("Invalid map".to_string())),
                }
            })
            .collect::<Result<Vec<Attr>, String>>()?;
        Ok(Attrs(attrs))
    }
}

/// Removes the leading and traing single quotes of DB strings
fn strip_string_quotes(s: &str) -> &str {
    s.strip_prefix('\'')
        .unwrap_or(s)
        .strip_suffix('\'')
        .unwrap_or(s)
}
