use std::error::Error;
use std::fmt;

// TODO investigate error types later
#[derive(Debug)]
pub enum JsonError {
    StringToJsonError,
    StringToJsonListError,
    InvalidSyntaxError,
}
impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "something went wrong")
    }
}
impl Error for JsonError {}

pub enum JsonValue {
    String(String),
    Boolean(bool),
    Number(f64),
    List(Vec<JsonObj>),
    Object(JsonObj),
    Null,
    None,
}

pub enum JsonEntry {
    Object(JsonObj),
    Array(Vec<JsonEntry>),
    Pair(JsonKVPair),
}

pub struct JsonKVPair {
    key: String,
    value: JsonValue,
}

pub type JsonObj = Vec<JsonKVPair>;

// !!! TODO json lists might just be arrays of values
// TODO replace all input[] with get to stop panicking
pub fn stojson_list(input: &str) -> Result<JsonEntry, JsonError> {
    if input[0..1] != *"[" {
        return Err(JsonError::StringToJsonListError);
    }
    let mut stack: Vec<char> = vec![];
    let mut json_obj_strings: Vec<String> = vec![];
    let mut json_obj_string: String = String::new();
    for c in input.chars() {
        match c {
            '[' => stack.push(c),
            '{' => {
                stack.push(c);
                json_obj_string.push(c);
            }
            '}' => {
                stack.pop();
                json_obj_string.push(c);
                if stack.len() < 2 {
                    json_obj_strings.push(json_obj_string.clone());
                    json_obj_string.clear(); // capacity?
                }
            }
            ',' => {
                // don't push json object separating commas
                if stack.len() < 2 {
                    continue;
                } else {
                    json_obj_string.push(c);
                }
            }
            ']' => drop(stack.pop()),
            _ => json_obj_string.push(c),
        }
        if stack.is_empty() {
            // possibly redundant but if input string is weird..?
            break;
        }
    }
    let output: Result<Vec<JsonEntry>, JsonError> = json_obj_strings
        .iter()
        .map(|json_str| stojson(json_str))
        .collect();

    Ok(JsonEntry::Array(output?))
}

// Parses a json string in the format {..}
pub fn stojson(input: &str) -> Result<JsonEntry, JsonError> {
    match input.as_bytes().get(0) {
        Some(c) => match c {
            b'{' => {
                // remove first and last char
                input.chars().next();
                input.chars().next_back();
                Ok(JsonEntry::Object(handle_json_obj(input)?))
            }
            b'[' => Ok(stojson_list(input)?),
            b'"' => Ok(JsonEntry::Pair(handle_json_kvpair(input)?)),
            _ => Err(JsonError::StringToJsonError),
        },
        None => Err(JsonError::StringToJsonError),
    }
}

// creates a list of key:value pairs
fn handle_json_obj(input: &str) -> Result<JsonObj, JsonError> {
    let mut result: JsonObj = vec![];
    match input.as_bytes()[0] {
        b' ' | b'\t' | b'\n' | b'\r' | b',' => match input.get(1..) {
            Some(x) => {
                result.append(&mut handle_json_obj(x)?);
            }
            None => return Err(JsonError::InvalidSyntaxError),
        },
        b'"' => result.push(handle_json_kvpair(&input)?),
        b'}' => return Ok(result),
        _ => return Err(JsonError::InvalidSyntaxError),
    }
    return Ok(result);
}

// creates a key:value pair
fn handle_json_kvpair(input: &str) -> Result<JsonKVPair, JsonError> {
    let mut result: JsonKVPair = JsonKVPair {
        key: String::new(),
        value: JsonValue::None,
    };
    let key_end: usize = handle_json_string(&input[1..])?;
    result.key.push_str(&input[1..key_end]);
    let val_start: usize = find_value_start(&input[key_end + 1..])? + 1;
    result.value = handle_json_value(&input[val_start..])?;
    return Ok(result);
}

// Returns the index of the : separator
fn find_value_start(input: &str) -> Result<usize, JsonError> {
    let mut result: usize = 0;
    for byte in input.as_bytes() {
        result += 1;
        match byte {
            b':' => break,
            b' ' | b'\r' | b'\n' | b'\t' => {}
            _ => return Err(JsonError::InvalidSyntaxError),
        }
    }
    return Ok(result);
}

// returns the index where the last " is in a json string
fn handle_json_string(input: &str) -> Result<usize, JsonError> {
    return match input.as_bytes()[0] {
        b'\\' => {
            Ok(0)
            // !!!TODO determine if valid escape
        }
        b'"' => Ok(1),
        _ => Ok(1 + handle_json_string(&input[1..])?),
    };
}

// !!! TODO deal with objects.
// handles values
fn handle_json_value(input: &str) -> Result<JsonValue, JsonError> {
    return match input.as_bytes()[0] {
        b'"' => {
            // turn json string into a JsonValue
            let mut string_value: String = String::new();
            string_value.push_str(&input[1..handle_json_string(&input[1..])?]);

            Ok(JsonValue::String(string_value))
        }
        b'n' => {
            let value: Option<&str> = input.get(0..4);
            return match value {
                Some(x) => {
                    return match x {
                        //handle null
                        "null" => Ok(JsonValue::Null),
                        _ => Err(JsonError::InvalidSyntaxError),
                    };
                }
                None => Err(JsonError::InvalidSyntaxError),
            };
        }
        b't' => {
            let value: Option<&str> = input.get(0..4);
            return match value {
                Some(x) => {
                    return match x {
                        "true" => Ok(JsonValue::Boolean(true)),
                        _ => Err(JsonError::InvalidSyntaxError),
                    }
                }
                None => Err(JsonError::InvalidSyntaxError),
            };
        } //handle true
        b'f' => {
            //handle false
            let value: Option<&str> = input.get(0..4);
            return match value {
                Some(x) => {
                    return match x {
                        "true" => Ok(JsonValue::Boolean(false)),
                        _ => Err(JsonError::InvalidSyntaxError),
                    }
                }
                None => Err(JsonError::InvalidSyntaxError),
            };
        }
        b' ' | b'\t' | b'\n' | b'\r' => Ok(handle_json_value(&input[1..])?),
        b'{' => Ok(JsonValue::Object(handle_json_obj(&input[1..])?)),
        _ => Ok(JsonValue::Number(
            input[0..handle_json_num(&input[1..])?].parse().unwrap(),
        )),
    };
}

fn handle_json_num(input: &str) -> Result<usize, JsonError> {
    // loop forward until a whitespace or any tab, newline, return
    return match input.as_bytes()[0] {
        b'0'..b'9' | b'.' => Ok(1 + handle_json_num(&input[1..])?),
        b',' | b'\t' | b'\r' | b'\n' | b' ' => Ok(1),
        _ => Err(JsonError::InvalidSyntaxError),
    };
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parses_json_list_simple() {
        let input: String = String::from("[{\"key\":\"value\"}]");
        if let JsonEntry::Array(arr) = stojson_list(&input).unwrap() {
            assert_eq!(arr.len(), 1);
            if let JsonEntry::Object(obj) = &arr[0] {
                assert_eq!(&obj[0].key, "key");
                if let JsonValue::String(s) = &obj[0].value {
                    assert_eq!(s, "value");
                }
            }
        }
    }

    #[test]
    fn parses_json_list_real() {
        // A common output for Sway ipc get_workspaces
        let input: String = String::from("[
  {
    \"id\": 4,
    \"type\": \"workspace\",
    \"orientation\": \"horizontal\",
    \"percent\": null,
    \"urgent\": false,
    \"marks\": [],
    \"layout\": \"splith\",
    \"border\": \"none\",
    \"current_border_width\": 0,
    \"rect\": {
      \"x\": 0,
      \"y\": 0,
      \"width\": 1920,
      \"height\": 1080
    },
    \"deco_rect\": {
      \"x\": 0,
      \"y\": 0,
      \"width\": 0,
      \"height\": 0
    },
    \"window_rect\": {
      \"x\": 0,
      \"y\": 0,
      \"width\": 0,
      \"height\": 0
    },
    \"geometry\": {
      \"x\": 0,
      \"y\": 0,
      \"width\": 0,
      \"height\": 0
    },
    \"name\": \"1\",
    \"window\": null,
    \"nodes\": [],
    \"floating_nodes\": [],
    \"focus\": [
      119
    ],
    \"fullscreen_mode\": 1,
    \"sticky\": false,
    \"floating\": null,
    \"scratchpad_state\": null,
    \"num\": 1,
    \"output\": \"eDP-1\",
    \"representation\": \"H[firefox]\",
    \"focused\": false,
    \"visible\": false
  },
  {
    \"id\": 23,
    \"type\": \"workspace\",
    \"orientation\": \"horizontal\",
    \"percent\": null,
    \"urgent\": false,
    \"marks\": [],
    \"layout\": \"splith\",
    \"border\": \"none\",
    \"current_border_width\": 0,
    \"rect\": {
      \"x\": 0,
      \"y\": 0,
      \"width\": 1920,
      \"height\": 1080
    },
    \"deco_rect\": {
      \"x\": 0,
      \"y\": 0,
      \"width\": 0,
      \"height\": 0
    },
    \"window_rect\": {
      \"x\": 0,
      \"y\": 0,
      \"width\": 0,
      \"height\": 0
    },
    \"geometry\": {
      \"x\": 0,
      \"y\": 0,
      \"width\": 0,
      \"height\": 0
    },
    \"name\": \"2\",
    \"window\": null,
    \"nodes\": [],
    \"floating_nodes\": [],
    \"focus\": [
      45
    ],
    \"fullscreen_mode\": 1,
    \"sticky\": false,
    \"floating\": null,
    \"scratchpad_state\": null,
    \"num\": 2,
    \"output\": \"eDP-1\",
    \"representation\": \"H[T[H[foot org.pwmt.zathura] H[foot org.pwmt.zathura] foot obsidian jetbrains-idea-ce]]\",
    \"focused\": true,
    \"visible\": true
  },
  {
    \"id\": 15,
    \"type\": \"workspace\",
    \"orientation\": \"horizontal\",
    \"percent\": null,
    \"urgent\": false,
    \"marks\": [],
    \"layout\": \"splith\",
    \"border\": \"none\",
    \"current_border_width\": 0,
    \"rect\": {
      \"x\": 0,
      \"y\": 0,
      \"width\": 1920,
      \"height\": 1080
    },
    \"deco_rect\": {
      \"x\": 0,
      \"y\": 0,
      \"width\": 0,
      \"height\": 0
    },
    \"window_rect\": {
      \"x\": 0,
      \"y\": 0,
      \"width\": 0,
      \"height\": 0
    },
    \"geometry\": {
      \"x\": 0,
      \"y\": 0,
      \"width\": 0,
      \"height\": 0
    },
    \"name\": \"3\",
    \"window\": null,
    \"nodes\": [],
    \"floating_nodes\": [],
    \"focus\": [
      34
    ],
    \"fullscreen_mode\": 1,
    \"sticky\": false,
    \"floating\": null,
    \"scratchpad_state\": null,
    \"num\": 3,
    \"output\": \"eDP-1\",
    \"representation\": \"H[T[H[discord Spotify] thunderbird]]\",
    \"focused\": false,
    \"visible\": false
  }
]
");
        if let JsonEntry::Array(arr) = stojson_list(&input).unwrap() {
            assert_eq!(3, arr.len());
        } else {
            panic!("ruh roh this was supposed to be an array!");
        }
    }

    #[test]
    fn parses_json_obj_simple() {
        let input: String = String::from("{\"key\":\"value\"}");
        if let JsonEntry::Object(output) = stojson(&input).unwrap() {
            assert_eq!(output[0].key, "key");
            if let JsonValue::String(s) = &output[0].value {
                assert_eq!(s, "value");
            }
        }
    }

    #[test]
    fn parses_json_obj_real() {
        let input: &str = "{
    \"id\": 4,
    \"type\": \"workspace\",
    \"orientation\": \"horizontal\",
    \"percent\": null,
    \"urgent\": false,
    \"marks\": [],
    \"layout\": \"splith\",
    \"border\": \"none\",
    \"current_border_width\": 0,
    \"rect\": {
      \"x\": 0,
      \"y\": 0,
      \"width\": 1920,
      \"height\": 1080
    },
    \"deco_rect\": {
      \"x\": 0,
      \"y\": 0,
      \"width\": 0,
      \"height\": 0
    },
    \"window_rect\": {
      \"x\": 0,
      \"y\": 0,
      \"width\": 0,
      \"height\": 0
    },
    \"geometry\": {
      \"x\": 0,
      \"y\": 0,
      \"width\": 0,
      \"height\": 0
    },
    \"name\": \"1\",
    \"window\": null,
    \"nodes\": [],
    \"floating_nodes\": [],
    \"focus\": [
      119
    ],
    \"fullscreen_mode\": 1,
    \"sticky\": false,
    \"floating\": null,
    \"scratchpad_state\": null,
    \"num\": 1,
    \"output\": \"eDP-1\",
    \"representation\": \"H[firefox]\",
    \"focused\": false,
    \"visible\": false
  }";
        if let JsonEntry::Object(obj) = stojson(input).unwrap() {
            assert_eq!("id", obj[0].key);
            if let JsonValue::Number(n) = obj[0].value {
                assert_eq!(4f64, n);
            } else {
                panic!("test failed");
            };
        } else {
            panic!("ruh roh was not an object!");
        }
    }
}
