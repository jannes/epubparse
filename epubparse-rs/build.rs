use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

use serde_json::Value;

// https://html.spec.whatwg.org/entities.json
const NAMED_ENTITIES_JSON: &str = include_str!("resources/named_html_entities.json");

fn main() {
    let named_entitites_json: Value = serde_json::from_str(NAMED_ENTITIES_JSON).unwrap();
    let named_entities: HashMap<&str, &str> = named_entitites_json
        .as_object()
        .unwrap()
        .iter()
        .map(|(name, v)| {
            let chars = v
                .as_object()
                .unwrap()
                .get("characters")
                .unwrap()
                .as_str()
                .unwrap();
            (name.as_str(), chars)
        })
        .collect();

    // construct named_entities.rs
    let mut named_entities_rs = String::new();
    named_entities_rs.push_str(
        "use std::collections::HashMap;
    pub fn get_named_entities() -> HashMap<String, String> {
        HashMap::from([\n",
    );
    for (name, chars) in named_entities {
        let name = &name[1..name.len() - 1];
        named_entities_rs.push_str(&format!(
            "    (\"{}\".to_string(), \"{}\".to_string()),\n",
            name,
            chars.escape_unicode()
        ))
    }
    named_entities_rs.push_str(
        "])
    }",
    );

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("entities.rs");
    fs::write(&dest_path, named_entities_rs).unwrap();
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=resources/named_html_entities.json");
}
