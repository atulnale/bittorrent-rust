use std::{env, io};

use serde::Deserialize;

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
#[derive(Debug, Clone, Deserialize)]
struct Torrent {
    announce: reqwest::Url,
    info: Info,
}
#[derive(Debug, Clone, Deserialize)]
struct Info {
    name: String,
    #[serde(rename = "piece length")]
    plength: usize,
    length: usize,
    #[serde(with = "serde_bytes")]
    pieces: Vec<u8>,
    #[serde(default)]
    files: Option<Vec<File>>,
}

#[derive(Debug, Clone, Deserialize)]
struct File {
    length: usize,
    path: Vec<String>,
}

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
    match command.as_str() {
        "decode" => {
            let encoded_value = &args[2];

            let (val, _) = decode_bencoded_value(encoded_value);
            // let val = decode_using_lib(encoded_value).unwrap();
            println!("{}", val.to_string());
        }
        "info" => {
            let content = get_file_content(&args[2]).unwrap();
            let torrent: Torrent = serde_bencode::from_bytes(content.as_slice()).unwrap();
            get_torrent_info(&torrent);
        }
        _ => {
            println!("unknown command: {}", args[1])
        }
    }
}

fn decode_using_lib(s: &str) -> io::Result<serde_json::Value> {
    let res: serde_json::Value = serde_bencode::from_bytes(s.as_bytes())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    dbg!(&res);
    Ok(res)
}

fn get_torrent_info(torrent: &Torrent) {
    println!("Tracker URL: {}", torrent.announce);
    println!("Length: {}", torrent.info.length);
}
fn get_file_content(path: &String) -> std::io::Result<Vec<u8>> {
    dbg!(path);
    let content = std::fs::read(path)?;
    Ok(content)
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
