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
use simd_json::value::tape::StaticNode;

/// convert_string_to_f32 is used to convert string into f32.
pub fn convert_string_to_f32(val: String) -> Option<f32> {
    if let Ok(val) = val.parse::<f32>() {
        return Some(val);
    }
    None
}

/// convert_static_node_to_f32 is used to convert static node to f32
pub fn convert_static_node_to_f32(val: StaticNode) -> Option<f32> {
    match val {
        StaticNode::I64(num) => Some(num as f32),
        StaticNode::U64(num) => Some(num as f32),
        StaticNode::F64(num) => Some(num as f32),
        _ => None,
    }
}

/// convert_static_node_to_string is used to convert static node to string
pub fn convert_static_node_to_string(val: StaticNode) -> Option<String> {
    match val {
        StaticNode::I64(num) => Some(format!("{}", num)),
        StaticNode::U64(num) => Some(format!("{}", num)),
        StaticNode::F64(num) => Some(format!("{}", num)),
        _ => None,
    }
}
