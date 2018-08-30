extern crate rami;
extern crate serde_json;
extern crate linked_hash_map;

use linked_hash_map::LinkedHashMap;

use rami::driver::driver::DriverData;
use rami::util;
use rami::category::Signature;

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};

fn ctype(type_str: &str) -> String {
    match type_str {
        "number" => "double",
        "integer" => "long",
        "string" => "const char*",
        _ => unimplemented!()
    }.to_string()
}

fn merge_map(map1: &LinkedHashMap<String, serde_json::Value>, map2: &LinkedHashMap<String, serde_json::Value>) -> LinkedHashMap<String, serde_json::Value>{
    map1.keys()
        .map(|key| {
            (
                key.clone(),
                match (map1.get(key), map2.get(key)) {
                    (Some(schema), Some(v)) => {
                        let mut new_val = v.clone();
                        util::merge_value(&mut new_val, &schema);
                        new_val
                    }
                    (Some(schema), None) => schema.clone(),
                    _ => unreachable!(),
                },
            )
        }).collect()
}

fn main() {
    let drv = DriverData::new(".").unwrap();

    let mut writer = BufWriter::new(File::create("rami.gen.h").unwrap());

    let fields = drv.schemas().iter().fold(String::new(), |acc, (k, v)| {
        format!(
            "{}  {} {};\n",
            acc,
            ctype(v["type"].as_str().unwrap()),
            k.replace(".", "_")
        )
    });
    write!(
        &mut writer,
        "typedef struct Config_ {{\n{}}} __attribute__((__packed__)) Config;\n",
        fields
    );

    let mut merged_signs: HashMap<String, Signature> = HashMap::new();
    for ctg in drv.category() {
        for (name, sign) in ctg.signatures().into_iter() {
            let sign: Signature = sign.clone();
            let gs = if let Some(existing_sign) = merged_signs.get(name) {
                let args = if let Some(ref v_args) = existing_sign.args {
                    merge_map(&v_args, &sign.args.unwrap_or_default())
                } else {
                    sign.args.unwrap_or_default()
                };
                let returns = if let Some(ref v_rets) = existing_sign.returns {
                    merge_map(&v_rets, &sign.returns.unwrap_or_default())
                } else {
                    sign.returns.unwrap_or_default()
                };
                Signature {
                    args: Some(args),
                    returns: Some(returns),
                }
            } else {
                sign
            };
            merged_signs.insert(name.clone(), gs);
        }
    }

    for (name, sign) in merged_signs {
        let arg_fields =
            sign.args
                .clone()
                .unwrap_or_default()
                .iter()
                .fold(String::new(), |acc, (key, val)| {
                    format!(
                        "{}  {} {};\n",
                        acc,
                        ctype(val["type"].as_str().unwrap()),
                        key.replace(".", "_")
                    )
                });
        write!(
            &mut writer,
            "typedef struct {0}_args_ {{\n{1}}} __attribute__((__packed__)) {0}_args;\n",
            name, arg_fields
        );

        let return_fields = sign.returns.clone().unwrap_or_default().iter().fold(
            String::new(),
            |acc, (key, val)| {
                format!(
                    "{}  {} {};\n",
                    acc,
                    ctype(val["type"].as_str().unwrap()),
                    key.replace(".", "_")
                )
            },
        );
        write!(
            &mut writer,
            "typedef struct {0}_returns_ {{\n{1}}} __attribute__((__packed__)) {0}_returns;\n",
            name, return_fields
        );

        write!(
            &mut writer,
            "{0}_returns* {0}({0}_args* args, Config* conf);\n",
            name
        );
    }
}
