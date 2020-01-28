/*
 * Copyright 2019 Balaji Jinnah and Contributors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
use crate::util::util::convert_static_node_to_string;
use failure::Error;
use simd_json;
use simd_json::value::borrowed::{Object, Value};
use std::collections::{HashMap, VecDeque};
/// get_value_from_json is used to get value of the given json from the flattened key.
pub fn get_value_from_json(key: String, json: &mut [u8]) -> Result<Option<Value>, Error> {
    //TODO: this function should split into two, where we give the simd_json object becacuse
    // no need to keep parsing for when we use operators like AND or NOT.
    // Now we have to parse the json in order to get the value from the flattend
    // json key.
    let mut json: simd_json::BorrowedValue = simd_json::to_borrowed_value(json)?;
    // Now recursively find value of the given flattened key.
    let mut path: VecDeque<&str> = key.split(".").collect();
    let mut key = path.pop_front().unwrap();
    'outer: loop {
        match json {
            Value::Object(mut obj) => {
                // I'm using this variable to capture the reference of the combined key. If you have
                // better idea. Please do it.
                let mut holder = String::from("");
                'inner: loop {
                    if let Some(inner) = obj.remove(key) {
                        json = inner;
                        if path.len() == 0 {
                            // We're at the end of the traversal. So return here.
                            return Ok(Some(json));
                        }
                        key = path.pop_front().unwrap();
                        continue 'outer;
                    }
                    if path.len() == 0 {
                        break 'outer;
                    }
                    // If there is no key, append the next part and check for it. Beacuse,
                    // json keys may contain dot. For example k8s.io/stream. Yeah, bit wierd,
                    // This is the way it works.
                    holder = format!("{}.{}", key, path.pop_front().unwrap());
                    key = &holder;
                }
            }
            _ => {
                // We can only traverse over objects so breaking here.
                // There is no objects for the path.
                break;
            }
        }
    }
    Ok(None)
}

/// deep_flaten_json will deep traverse and flatten the json.
fn deep_flaten_json(past_key: String, val: Value, past_memory: &mut HashMap<String, Vec<String>>) {
    let mut collect_memory = |key: String, value: String| {
        if let Some(vals) = past_memory.get_mut(&key) {
            vals.push(value);
            return;
        }
        past_memory.insert(key, vec![value]);
        return;
    };

    match val {
        Value::Object(mut obj) => {
            for (key, value) in obj.drain() {
                deep_flaten_json(format!("{}.{}", past_key, key), value, past_memory);
            }
        }
        Value::Static(val) => {
            collect_memory(past_key, convert_static_node_to_string(val).unwrap());
        }
        Value::String(val) => {
            collect_memory(past_key, val.into_owned());
        }
        Value::Array(vals) => {
            for val in vals {
                deep_flaten_json(past_key.clone(), val, past_memory);
            }
        }
    }
}

/// flatten_json flatten the given json into key value pair.
pub fn flatten_json(buf: &mut Vec<u8>) -> Result<HashMap<String, Vec<String>>, failure::Error> {
    let mut result: HashMap<String, Vec<String>> = HashMap::new();
    let json: simd_json::BorrowedValue = simd_json::to_borrowed_value(buf)?;
    match json {
        Value::Object(mut obj) => {
            for (key, value) in obj.drain() {
                deep_flaten_json(key.into_owned(), value, &mut result);
            }
        }
        _ => panic!("Invalid json object traversal."),
    }
    Ok(result)
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use simd_json::value::borrowed::Value;
    use simd_json::value::tape::StaticNode;
    #[test]
    fn test_basic_json_parsing() {
        let mut json = br#"{
            "name": "Licenser",
            "skills": {
                "language": "Rust",
                "yo": {
                    "age": 1
                },
                "la": ["yo","yo1"]
            }
        }"#
        .to_vec();
        // 0 Level nesting.
        let val = get_value_from_json("name".to_string(), &mut json).unwrap();
        assert_eq!(
            Value::String(std::borrow::Cow::Borrowed("Licenser")),
            val.unwrap()
        );
        // 1 level nesting.
        let val = get_value_from_json("skills.language".to_string(), &mut json).unwrap();
        assert_eq!(
            Value::String(std::borrow::Cow::Borrowed("Rust")),
            val.unwrap()
        );
        // 2 level nesting.
        let val = get_value_from_json("skills.yo.age".to_string(), &mut json).unwrap();
        assert_eq!(Value::Static(StaticNode::U64(1)), val.unwrap());

        // Array assertion.
        let val = get_value_from_json("skills.la".to_string(), &mut json).unwrap();
        assert_eq!(
            Value::Array(vec![
                Value::String(std::borrow::Cow::Borrowed("yo")),
                Value::String(std::borrow::Cow::Borrowed("yo1"))
            ]),
            val.unwrap()
        );

        // None assertion
        // Array assertion.
        let val = get_value_from_json("skills.ram".to_string(), &mut json).unwrap();
        assert_eq!(None, val);
    }

    #[test]
    fn test_dotted_json() {
        let mut json = br#"{
            "name": "Licenser",
            "skills": {
                "language.name": "Rust"
            },
            "k8s.io/source":{
                "name": "cluster"
            }
        }"#
        .to_vec();

        let val = get_value_from_json("skills.language.name".to_string(), &mut json).unwrap();
        assert_eq!(
            Value::String(std::borrow::Cow::Borrowed("Rust")),
            val.unwrap()
        );
        // 2 level nesting.
        let val = get_value_from_json("k8s.io/source.name".to_string(), &mut json).unwrap();
        assert_eq!(
            Value::String(std::borrow::Cow::Borrowed("cluster")),
            val.unwrap()
        );
    }
}
