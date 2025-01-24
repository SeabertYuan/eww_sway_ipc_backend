use std::error::Error;
use std::fmt;

// TODO investigate error types later
#[derive(Debug)]
pub enum JsonError {
    StringToJsonError,
    StringToJsonListError,
}
impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "something went wrong")
    }
}
impl Error for JsonError {}

pub enum JsonValue {
    String(String),
    Number(f64),
    List(Vec<JsonObj>),
    Obj(JsonObj),
}
pub struct JsonKVPair {
    key: String,
    value: JsonValue,
}

pub type JsonObj = Vec<JsonKVPair>;

pub fn stojson_list(input: &str) -> Result<Vec<JsonObj>, JsonError> {
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
                json_obj_strings.push(json_obj_string.clone());
                json_obj_string.clear(); // capacity?
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
    let output: Vec<Result<JsonObj, JsonError>> = json_obj_strings
        .iter()
        .map(|json_str| stojson(json_str))
        .collect();

    output.into_iter().collect()
}

pub fn stojson(input: &str) -> Result<JsonObj, JsonError> {
    Ok(vec![JsonKVPair {
        key: String::new(),
        value: JsonValue::Number(1f64),
    }])
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parses_json_list_simple() {
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
        let output: Vec<JsonObj> = stojson_list(&input).unwrap();
        assert_eq!(output.len(), 3);
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
        let output: JsonObj = stojson(input).unwrap();
        assert_eq!("id", output[0].key);
        if let JsonValue::Number(n) = output[0].value {
            assert_eq!(4f64, n);
        } else {
            panic!("test failed");
        };
    }
}
