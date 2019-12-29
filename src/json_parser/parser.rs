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
use failure::Error;
use simd_json;
use simd_json::value::borrowed::Value;
/// get_value_from_json is used to get value of the given json from the flattened key.
pub fn get_value_from_json(key: String, json: &mut [u8]) -> Result<Option<Value>, Error> {
    //TODO: this function should split into two, where we give the simd_json object becacuse
    // no need to keep parsing for when we use operators like AND or NOT.
    // Now we have to parse the json in order to get the value from the flattend
    // json key.
    let mut json: simd_json::BorrowedValue = simd_json::to_borrowed_value(json)?;
    // Now recursively find value of the given flattened key.
    let path: Vec<&str> = key.split(".").collect();
    let path_len = path.len();
    let mut traversed_len = 0;

    for key in path {
        match json {
            Value::Object(mut obj) => {
                if let Some(inner) = obj.remove(key) {
                    traversed_len = traversed_len + 1;
                    json = inner;
                    if traversed_len == path_len {
                        // We're at the end of the traversal. So return here.
                        return Ok(Some(json));
                    }
                    continue;
                }
                break;
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
}
