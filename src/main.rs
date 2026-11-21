use std::{
    collections::{BTreeMap, HashMap},
    env,
};

// Available if you need it!
// use serde_bencode

// #[allow(dead_code)]
// fn decode_bencoded_value(encoded_value: &str) -> serde_json::Value {
//     if let Some(x) = encoded_value
//         .strip_prefix("i")
//         .and_then(|x| x.split_once("e"))
//         .and_then(|(digits, _)| digits.parse::<i64>().ok())
//     {
//         return x.into();
//     } else if let Some((len, rest)) = encoded_value.split_once(':') {
//         if let Ok(len) = len.parse::<usize>() {
//             return serde_json::Value::String(rest[..len].to_string());
//         }
//     }
//     panic!("Unhandled encoded value: {}", encoded_value)
// }

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &str) -> (serde_json::Value, &str) {
    match encoded_value.chars().next() {
        Some('i') => {
            if let Some((n, rest)) = encoded_value
                .strip_prefix("i")
                .and_then(|rest| rest.split_once("e"))
            {
                let num = n.parse::<i64>().ok().unwrap();
                return (num.into(), rest);
            }
        }
        Some('l') => {
            let mut v: Vec<serde_json::Value> = Vec::new();
            let mut rest = &encoded_value[1..];
            while !rest.is_empty() && !rest.starts_with('e') {
                let (val, remainder) = decode_bencoded_value(rest);
                v.push(val);
                rest = remainder;
            }
            return (v.into(), &rest[1..]);
        }
        Some('d') => {
            let mut map = serde_json::Map::new();
            let mut rest = encoded_value.split_at(1).1;
            while !rest.is_empty() && !rest.starts_with('e') {
                let (key, rem) = decode_bencoded_value(rest);
                let key = match key {
                    serde_json::Value::String(k) => k,
                    k => {
                        panic!("key should be string not {k:?}")
                    }
                };
                let (val, rem) = decode_bencoded_value(rem);
                map.insert(key, val);
                rest = rem;
            }
            return (map.into(), &rest[1..]);
        }
        Some('1'..='9') => {
            if let Some((len, rest)) = encoded_value.split_once(":") {
                if let Ok(len) = len.parse::<usize>() {
                    return (rest[..len].to_string().into(), &rest[len..]);
                }
            }
        }
        _ => {}
    }
    panic!("Unhandled encoded value: {}", encoded_value)
}

// Usage: your_program.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        // You can use print statements as follows for debugging, they'll be visible when running tests.

        let encoded_value = &args[2];
        let (val, _) = decode_bencoded_value(encoded_value);
        println!("{}", val.to_string());
    } else {
        println!("unknown command: {}", args[1])
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    #[test]
    fn test1() {
        let (val, _) = decode_bencoded_value("i32e");
        assert_eq!(serde_json::Value::Number(32.into()), val);
    }
    #[test]
    fn test2() {
        let (val, _) = decode_bencoded_value("5:hello");
        assert_eq!(json!("hello"), val);
    }
    #[test]
    fn test3() {
        let (val, _) = decode_bencoded_value("li32ee");
        assert_eq!(json!([32]), val);
    }
    #[test]
    fn test4() {
        let (val, _) = decode_bencoded_value("li32eli44eee");
        assert_eq!(json!([32, [44]]), val);
    }
    #[test]
    fn test5() {
        let (val, _) = decode_bencoded_value("lli4eei5ee");
        assert_eq!(json!([[4], 5]), val);
    }
}
