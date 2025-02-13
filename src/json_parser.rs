use std::cell::RefCell;
use std::error::Error;
use std::fmt;
use std::rc::Rc;

// TODO investigate error types later
#[derive(Debug)]
pub enum JsonError {
    StringToJsonError,
    StringToJsonListError,
    InvalidSyntaxError,
    RanOutOfCharsError,
    InvalidTypeError,
    InvalidNumberError,
}
impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "something went wrong")
    }
}
impl Error for JsonError {}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct JsonKVPair {
    key: String,
    value: JsonValue,
}

pub type JsonObj = Vec<JsonKVPair>;

// !!! TODO json lists might just be arrays of values
// TODO replace all input[] with get to stop panicking
pub fn stojson_list(input: Rc<RefCell<String>>) -> Result<JsonEntry, JsonError> {
    let input_borrow = input.borrow();
    if input_borrow[0..1] != *"[" {
        return Err(JsonError::StringToJsonListError);
    }
    let mut stack: Vec<char> = vec![];
    let mut json_obj_strings: Vec<String> = vec![];
    let mut json_obj_string: String = String::new();
    for c in input_borrow.chars() {
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
        .map(|json_str| stojson(Rc::new(RefCell::new(json_str.to_string()))))
        .collect();

    Ok(JsonEntry::Array(output?))
}

// TODO handle a json obj with multiple entries
// Parses a json string in the format {..}
pub fn stojson(input: Rc<RefCell<String>>) -> Result<JsonEntry, JsonError> {
    let mut input_borrow = input.borrow_mut();
    match input_borrow.as_bytes().get(0) {
        Some(c) => match c {
            b'{' => {
                // remove first and last char
                input_borrow.chars().next();
                input_borrow.chars().next_back();
                *input_borrow = input_borrow.as_str()[1..].to_string();
                Ok(JsonEntry::Object(handle_json_obj(Rc::clone(&input))?))
            }
            b'[' => Ok(stojson_list(Rc::clone(&input))?),
            b'"' => Ok(JsonEntry::Pair(handle_json_kvpair(Rc::clone(&input))?)),
            _ => Err(JsonError::StringToJsonError),
        },
        None => Err(JsonError::StringToJsonError),
    }
}

// takes in a potential json object with { peeled off (the object should look like ..}) creates a list of key:value pairs
fn handle_json_obj(input: Rc<RefCell<String>>) -> Result<JsonObj, JsonError> {
    let mut input_borrow = input.borrow_mut();
    let mut result: JsonObj = vec![];
    match input_borrow.as_bytes().get(0) {
        Some(b) => match b {
            b' ' | b'\t' | b'\n' | b'\r' | b',' => match input_borrow.as_bytes().get(0) {
                Some(_next_input) => {
                    *input_borrow = input_borrow.as_str()[1..].to_string();
                    let output = &mut handle_json_obj(Rc::clone(&input))?;
                    println!("appending: {:?}", output);
                    result.append(output);
                }
                None => return Err(JsonError::RanOutOfCharsError),
            },
            b'"' => result.push(handle_json_kvpair(Rc::clone(&input))?),
            b'}' => return Ok(result),
            _ => return Err(JsonError::InvalidTypeError),
        },
        None => return Err(JsonError::RanOutOfCharsError),
    }
    return Ok(result);
}

// creates a key:value pair from "k" .. : .. v
fn handle_json_kvpair(input: Rc<RefCell<String>>) -> Result<JsonKVPair, JsonError> {
    println!("handling KV pair");
    let mut input_borrow = input.borrow_mut();
    let mut result: JsonKVPair = JsonKVPair {
        key: String::new(),
        value: JsonValue::None,
    };
    let key_end: usize = handle_json_string(&input_borrow[1..])?;
    println!("pushing {}", &input_borrow[1..key_end]);
    result.key.push_str(&input_borrow[1..key_end]);
    // key_end is the ending index, but we want to remove that and the :
    *input_borrow = input_borrow[key_end + 1..].to_string();
    let val_start: usize = key_end + find_value_start(&input_borrow)? + 2;
    println!("checking for value in: {}", &input_borrow[val_start..]);
    *input_borrow = input_borrow[val_start..].to_string();
    result.value = handle_json_value(Rc::clone(&input))?;
    return Ok(result);
}

// Returns the index of the : separator
fn find_value_start(input: &str) -> Result<usize, JsonError> {
    println!("finding starting value");
    let mut result: usize = 0;
    for byte in input.as_bytes() {
        match byte {
            b':' => break,
            b' ' | b'\r' | b'\n' | b'\t' => {}
            _ => return Err(JsonError::InvalidSyntaxError),
        }
        result += 1;
    }
    return Ok(result);
}

// returns the index where the last " is in the original json string ".."
fn handle_json_string(input: &str) -> Result<usize, JsonError> {
    return match input.as_bytes().get(0) {
        Some(b) => match b {
            b'\\' => match input.as_bytes().get(2) {
                Some(_c) => Ok(2 + handle_json_string(&input[2..])?),
                None => Err(JsonError::RanOutOfCharsError),
            },
            b'"' => Ok(1),
            _ => Ok(1 + handle_json_string(&input[1..])?),
        },
        None => Err(JsonError::RanOutOfCharsError),
    };
}

// handles values in the format wv.. where w is any whitespace, v is the value and any remaining
// json strings that occur after
fn handle_json_value(input: Rc<RefCell<String>>) -> Result<JsonValue, JsonError> {
    let input_borrow = input.borrow_mut();
    return match input.as_bytes().get(0) {
        Some(b) => match b {
            b'"' => {
                // turn json string into a JsonValue
                let mut string_value: String = String::new();
                *input_borrow = input_borrow[1..].to_string();
                string_value.push_str(&input_borrow[..handle_json_string(&input_borrow)?]);

                Ok(JsonValue::String(string_value))
            }
            b'n' => {
                let value: Option<&str> = input_borrow.get(0..4);
                return match value {
                    Some(x) => {
                        return match x {
                            //handle null
                            "null" => {
                                *input_borrow = input_borrow[4..].to_string();
                                Ok(JsonValue::Null)
                            }
                            _ => Err(JsonError::InvalidTypeError),
                        };
                    }
                    None => Err(JsonError::RanOutOfCharsError),
                };
            }
            b't' => {
                let value: Option<&str> = input_borrow.get(0..4);
                return match value {
                    Some(x) => {
                        return match x {
                            "true" => {
                                *input_borrow = input_borrow[4..].to_string();
                                Ok(JsonValue::Boolean(true))
                            }
                            _ => Err(JsonError::InvalidTypeError),
                        }
                    }
                    None => Err(JsonError::RanOutOfCharsError),
                };
            } //handle true
            b'f' => {
                //handle false
                let value: Option<&str> = input_borrow.get(0..5);
                return match value {
                    Some(x) => {
                        return match x {
                            "false" => {
                                *input_borrow = input_borrow[5..].to_string();
                                Ok(JsonValue::Boolean(false))
                            }
                            _ => Err(JsonError::InvalidTypeError),
                        }
                    }
                    None => Err(JsonError::RanOutOfCharsError),
                };
            }
            b' ' | b'\t' | b'\n' | b'\r' => {
                *input_borrow = input_borrow[1..].to_string();
                Ok(handle_json_value(Rc::clone(&input))?)
            }
            b'{' => {
                *input_borrow = input_borrow[1..].to_string();
                Ok(JsonValue::Object(handle_json_obj(Rc::clone(&input))?))
            }
            _ => {
                // TODO exponents might be allowed ie. 1e10.
                if let Ok(n) = input_borrow[0..handle_json_num(&input_borrow[1..])?].parse::<f64>()
                {
                    *input_borrow = input_borrow[1..].to_string();
                    Ok(JsonValue::Number(n))
                } else {
                    Err(JsonError::InvalidNumberError)
                }
            }
        },
        None => Err(JsonError::RanOutOfCharsError),
    };
}

// Returns index of possible end of number value (not inclusive)
fn handle_json_num(input: &str) -> Result<usize, JsonError> {
    // loop forward until a whitespace, tab, newline, return, or any environment closing
    return match input.as_bytes().get(0) {
        Some(b) => match b {
            b'0'..b'9' | b'.' => Ok(1 + handle_json_num(&input[1..])?),
            b',' | b'\t' | b'\r' | b'\n' | b' ' | b'}' | b']' => Ok(1),
            _ => Err(JsonError::InvalidTypeError),
        },
        None => Err(JsonError::RanOutOfCharsError),
    };
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn handle_json_num_works_properly() {
        // good cases
        let simple_good: &str = "4,";
        let floating_good: &str = "4.07 ";
        let boundary_low: &str = "0.23\t";
        let boundary_hi: &str = "9.102}";
        let tricky_good: &str = "10.02\t \r\n 2039.0";
        assert_eq!(handle_json_num(simple_good).unwrap(), 1);
        assert_eq!(handle_json_num(floating_good).unwrap(), 4);
        assert_eq!(handle_json_num(boundary_low).unwrap(), 4);
        assert_eq!(handle_json_num(boundary_hi).unwrap(), 5);
        assert_eq!(handle_json_num(tricky_good).unwrap(), 5);
        let simple_bad: &str = "a,";
        let poor_endchar: &str = "1";
        let mixed_input: &str = "407eepy102";
        assert!(matches!(
            handle_json_num(simple_bad).unwrap_err(),
            JsonError::InvalidTypeError
        ));
        assert!(matches!(
            handle_json_num(poor_endchar).unwrap_err(),
            JsonError::RanOutOfCharsError
        ));
        assert!(matches!(
            handle_json_num(mixed_input).unwrap_err(),
            JsonError::InvalidTypeError
        ));
    }

    #[test]
    fn handle_json_string_works_properly() {
        let simple_good = "hello\"";
        let escaped_r_good = "he\\llo\"";
        let escape_proper_good = "this\\\" should pass!\"";
        let longer_good = "this\" is really long";
        let empty_good = "\"";
        let simple_fail = "hello";
        let escape_fail = "this fails\\";
        assert_eq!(handle_json_string(simple_good).unwrap(), 6);
        assert_eq!(handle_json_string(escaped_r_good).unwrap(), 7);
        assert_eq!(handle_json_string(escape_proper_good).unwrap(), 20);
        assert_eq!(handle_json_string(longer_good).unwrap(), 5);
        assert_eq!(handle_json_string(empty_good).unwrap(), 1);
        assert!(matches!(
            handle_json_string(simple_fail).unwrap_err(),
            JsonError::RanOutOfCharsError
        ));
        assert!(matches!(
            handle_json_string(escape_fail).unwrap_err(),
            JsonError::RanOutOfCharsError
        ));
    }

    #[test]
    fn handle_json_value_good_cases() {
        let true_value = "true";
        let false_value = "false";
        let null_value = "null";
        let string_value = "\"the world is your oyster\"";
        let num_value = "2347 ";
        if let JsonValue::String(s) = handle_json_value(string_value).unwrap() {
            assert_eq!(s, "the world is your oyster");
        } else {
            panic!("basic string failed")
        };
        if let JsonValue::Boolean(b) = handle_json_value(true_value).unwrap() {
            assert!(b);
        } else {
            panic!("true failed")
        };
        if let JsonValue::Boolean(b) = handle_json_value(false_value).unwrap() {
            assert!(!b);
        } else {
            panic!("false failed")
        };
        assert!(matches!(
            handle_json_value(null_value).unwrap(),
            JsonValue::Null
        ));
        if let JsonValue::Number(n) = handle_json_value(num_value).unwrap() {
            assert_eq!(n, 2347f64);
        } else {
            panic!("basic number failed")
        };
    }

    #[test]
    fn handle_json_value_cplx_good_cases() {
        let rand_esc_str = "\"the world \\is your oyster\"";
        let real_esc_str = "\"\\n yep heres some nums too 102\"";
        let float_val = "723.47 ";
        if let JsonValue::String(s) = handle_json_value(rand_esc_str).unwrap() {
            assert_eq!(s, "the world \\is your oyster");
        } else {
            panic!("basic string failed")
        };
        if let JsonValue::String(s) = handle_json_value(real_esc_str).unwrap() {
            assert_eq!(s, "\\n yep heres some nums too 102");
        } else {
            panic!("basic string failed")
        };
        if let JsonValue::Number(n) = handle_json_value(float_val).unwrap() {
            assert_eq!(n, 723.47f64);
        } else {
            panic!("basic number failed")
        };
    }

    #[test]
    fn handle_json_value_err_cases() {
        let true_fail = "TRUE";
        let false_fail = "fALSE";
        let float_fail = "3.14.15 ";
        let null_fail = "NuLL";
        let nan_type = "NaN";
        let wrong_true = "t2gp";
        assert!(matches!(
            handle_json_value(true_fail).unwrap_err(),
            JsonError::InvalidTypeError
        ));
        assert!(matches!(
            handle_json_value(false_fail).unwrap_err(),
            JsonError::InvalidTypeError
        ));
        assert!(matches!(
            handle_json_value(float_fail).unwrap_err(),
            JsonError::InvalidNumberError
        ));
        assert!(matches!(
            handle_json_value(null_fail).unwrap_err(),
            JsonError::InvalidTypeError
        ));
        assert!(matches!(
            handle_json_value(nan_type).unwrap_err(),
            JsonError::InvalidTypeError
        ));
        assert!(matches!(
            handle_json_value(wrong_true).unwrap_err(),
            JsonError::InvalidTypeError
        ));
    }

    #[test]
    fn find_value_start_functions() {
        let simple = ": 7";
        let crazy_whitespace = "   :           7";
        let crazy_tabs = "\t:           7";
        let crazy_newlines = "\n\n\n\n\n\n:  \n     7";
        let crazy_returns = "\r\r\r\r:           7";
        let crazy = "\r\t\n\r : \r\t\n 23 bruhhhh";
        let fail = "\r\t\n\ra : \r\t\n 23 bruhhhh";
        assert_eq!(find_value_start(simple).unwrap(), 0);
        assert_eq!(find_value_start(crazy_whitespace).unwrap(), 3);
        assert_eq!(find_value_start(crazy_tabs).unwrap(), 1);
        assert_eq!(find_value_start(crazy_newlines).unwrap(), 6);
        assert_eq!(find_value_start(crazy_returns).unwrap(), 4);
        assert_eq!(find_value_start(crazy).unwrap(), 5);
        assert!(matches!(
            find_value_start(fail).unwrap_err(),
            JsonError::InvalidSyntaxError
        ));
    }

    #[test]
    fn handle_json_kv_pair_functions() {
        let basic_string = Rc::new(RefCell::new(String::from("\"key\":\"value\"")));
        let basic_int = Rc::new(RefCell::new(String::from("\"int\":7}")));
        let basic_float = Rc::new(RefCell::new(String::from("\"float\":4.7}")));
        let basic_null = Rc::new(RefCell::new(String::from("\"null\":null")));
        let basic_true = Rc::new(RefCell::new(String::from("\"true\":true")));
        let basic_false = Rc::new(RefCell::new(String::from("\"false\":false")));
        assert_eq!(
            handle_json_kvpair(Rc::clone(&basic_string)).unwrap().key,
            "key"
        );
        if let JsonValue::String(s) = handle_json_kvpair(Rc::clone(&basic_string)).unwrap().value {
            assert_eq!(s, "value");
        } else {
            panic!("value should have been string");
        }
        assert_eq!(
            handle_json_kvpair(Rc::clone(&basic_int)).unwrap().key,
            "int"
        );
        if let JsonValue::Number(n) = handle_json_kvpair(Rc::clone(&basic_int)).unwrap().value {
            assert_eq!(n, 7f64);
        } else {
            panic!("value should have been int");
        }
        assert_eq!(
            handle_json_kvpair(Rc::clone(&basic_float)).unwrap().key,
            "float"
        );
        if let JsonValue::Number(n) = handle_json_kvpair(Rc::clone(&basic_float)).unwrap().value {
            assert_eq!(n, 4.7f64);
        } else {
            panic!("value should have been float");
        }
        assert_eq!(
            handle_json_kvpair(Rc::clone(&basic_true)).unwrap().key,
            "true"
        );
        if let JsonValue::Boolean(b) = handle_json_kvpair(Rc::clone(&basic_true)).unwrap().value {
            assert_eq!(b, true);
        } else {
            panic!("value should have been true");
        }
        assert_eq!(
            handle_json_kvpair(Rc::clone(&basic_false)).unwrap().key,
            "false"
        );
        if let JsonValue::Boolean(b) = handle_json_kvpair(Rc::clone(&basic_false)).unwrap().value {
            assert_eq!(b, false);
        } else {
            panic!("value should have been false");
        }
        assert_eq!(
            handle_json_kvpair(Rc::clone(&basic_null)).unwrap().key,
            "null"
        );
        assert!(matches!(
            handle_json_kvpair(Rc::clone(&basic_null)).unwrap().value,
            JsonValue::Null
        ));
    }

    #[test]
    fn parses_json_list_simple() {
        let input: Rc<RefCell<String>> =
            Rc::new(RefCell::new(String::from("[{\"key\":\"value\"}]")));
        if let JsonEntry::Array(arr) = stojson_list(Rc::clone(&input)).unwrap() {
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
        let input = Rc::new(RefCell::new(String::from("[
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
")));
        if let JsonEntry::Array(arr) = stojson_list(Rc::clone(&input)).unwrap() {
            assert_eq!(3, arr.len());
        } else {
            panic!("ruh roh this was supposed to be an array!");
        }
    }

    #[test]
    fn parses_json_obj_simple() {
        let input = Rc::new(RefCell::new(String::from("{\"key\":\"value\"}")));
        if let JsonEntry::Object(output) = stojson(Rc::clone(&input)).unwrap() {
            assert_eq!(output[0].key, "key");
            if let JsonValue::String(s) = &output[0].value {
                assert_eq!(s, "value");
            }
        }

        let whitespaces = Rc::new(RefCell::new(String::from(
            "{        \"key\"    : \"value\"                              }",
        )));
        if let JsonEntry::Object(output) = stojson(Rc::clone(&whitespaces)).unwrap() {
            assert_eq!(output[0].key, "key");
            if let JsonValue::String(s) = &output[0].value {
                assert_eq!(s, "value");
            }
        }
    }

    #[test]
    fn parses_json_obj_less_simple() {
        let input = Rc::new(RefCell::new(String::from(
            "{\"key1\":\"value\",\"key2\":2,\"key3\":null,\"monke\":true}",
        )));
        if let JsonEntry::Object(output) = stojson(input).unwrap() {
            assert_eq!(output[0].key, "key1");
            assert_eq!(output[1].key, "key2");
            assert_eq!(output[2].key, "key3");
            assert_eq!(output[3].key, "monke");
            if let JsonValue::String(s) = &output[0].value {
                assert_eq!(s, "value");
            }
            if let JsonValue::Number(n) = output[1].value {
                assert_eq!(n, 2f64);
            }
            assert!(matches!(output[2].value, JsonValue::Null));
            if let JsonValue::Boolean(b) = &output[3].value {
                assert!(b);
            }
        }
    }

    #[test]
    fn parses_json_obj_real() {
        let input = Rc::new(RefCell::new(String::from(
            "{
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
  }",
        )));
        if let JsonEntry::Object(obj) = stojson(input).unwrap() {
            assert_eq!("id", obj[0].key);
            if let JsonValue::Number(n) = obj[0].value {
                assert_eq!(4f64, n);
            } else {
                panic!("test failed");
            };
            assert_eq!("deco_rect", obj[10].key);
            if let JsonValue::Object(o) = &obj[10].value {
                assert_eq!(o[0].key, "x");
                assert_eq!(o[3].key, "height");
            } else {
                panic!("test failed");
            };
        } else {
            panic!("ruh roh was not an object!");
        }
    }
}
