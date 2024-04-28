use serde::{de::DeserializeOwned, Serialize};
use unicode_segmentation::UnicodeSegmentation;

pub const FUTHARK: &'static str = include_str!("../data/alphabet.txt");
pub const ALPHA_NUM: &'static str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ123456";

// DECODING!
/**
This function takes a string of runes and converts it to a vector of numbers between 0 and 31.

The function takes two arguments:

   * runes: A string of runes to convert.

   * futhark: A boolean value that determines whether to use the Futhark alphabet or plain ASCII.
*/
pub fn parse_runes_to_points(runes: &str, alphabet: &str) -> Vec<u8> {
    let mut results = Vec::new();
    for rune in runes.graphemes(true) {
        let mut alphabet = alphabet.graphemes(true);
        if let Some(idx) = alphabet.position(|alpha| alpha == rune) {
            results.push(idx as u8);
            if idx == 32 {
                break;
            }
        }
    }
    results
}

/**
This function takes a vector of numbers between 0 and 31 and converts it to a vector of bytes.

This treats the numbers as a series of 5-bit values, and packs them into bytes.
*/
pub fn points_to_bytes(points: Vec<u8>) -> Vec<u8> {
    let mut results = Vec::new();
    let mut bits: u32 = 0;
    let mut offset = 0;
    for point in points {
        if offset == 0 {
            bits = point as u32;
            offset = 5;
        } else {
            bits |= (point as u32) << offset;
            offset += 5;
        }
        if offset >= 8 {
            results.push((bits & 0xff) as u8);
            bits >>= 8;
            offset -= 8;
        }
    }
    results
}

/**
This function takes a string of runes and converts it to a vector of bytes, for further parsing.
*/
pub fn parse_runes(runes: &str, alphabet: &str) -> Vec<u8> {
    let points = parse_runes_to_points(runes, alphabet);
    points_to_bytes(points)
}

/**
Returns a deserialized value from a string of runes.

This function takes two arguments:

   * runes: A string of runes to convert.

   * futhark: A boolean value that determines whether to use the Futhark alphabet or plain ASCII.
*/
pub fn read_from_runes<T: DeserializeOwned>(runes: &str, alphabet: &str) -> Option<T> {
    let bytes = parse_runes(runes, alphabet);
    postcard::from_bytes(&bytes).ok()
}

// ENCODING!
/**
This function takes a vector of bytes and converts it to a vector of numbers between 0 and 31.

This treats the bytes as a series of 8-bit values, and repacks them into 5-bit values.
 */
pub fn bytes_to_points(bytes: &[u8]) -> Vec<u8> {
    let mut results = Vec::new();
    let mut bits: u32 = 0;
    let mut offset = 0;
    for byte in bytes {
        bits |= (*byte as u32) << offset;
        offset += 8;
        while offset >= 5 {
            results.push((bits & 0x1f) as u8);
            bits >>= 5;
            offset -= 5;
        }
    }
    if offset != 0 {
        results.push(bits as u8);
    }
    results
}

/**
This function takes a vector of bytes and converts it to a Unicode String of runes.

This function takes two arguments:

   * bytes: A vector of bytes to convert.

   * futhark: A boolean value that determines whether to use the Futhark alphabet or plain ASCII.
*/
pub fn generate_runes(bytes: &[u8], alphabet: &str) -> String {
    let points = bytes_to_points(bytes);
    points
        .iter()
        .map(|point| alphabet.graphemes(true).nth(*point as usize).unwrap())
        .collect()
}

fn simple_generate_runes_ascii(bytes: &[u8]) -> String {
    let alphabet = "abcdefghijklmnopqrstuvwxyz123456";
    let points = bytes_to_points(bytes);
    points
        .iter()
        .map(|point| alphabet.bytes().nth(*point as usize).unwrap() as char)
        .collect::<String>()
}

pub fn create_runes<T: Serialize>(t: &T, alphabet: &str) -> String {
    let data = postcard::to_allocvec(t).unwrap();
    generate_runes(data.as_slice(), alphabet)
}

#[cfg(test)]
mod runes_tests {
    use super::*;

    #[test]
    fn test_parse_runes_to_points() {
        assert_eq!(parse_runes_to_points("ᚠᚢ", FUTHARK), vec![0, 1]);
        assert_eq!(parse_runes_to_points("ᚦᚨ", FUTHARK), vec![2, 3]);
        assert_eq!(parse_runes_to_points("AB", ALPHA_NUM), vec![0, 1]);
        assert_eq!(parse_runes_to_points("CD", ALPHA_NUM), vec![2, 3]);
    }

    #[test]
    fn test_parse_runes() {
        assert_eq!(parse_runes("ᚠᚠ", FUTHARK), vec![0b00000]);
        assert_eq!(parse_runes("ᚢᚠᛌ", FUTHARK), vec![0b00001]);
        assert_eq!(parse_runes("ᚠᛁᚢᚠ", FUTHARK), vec![64, 5]);
        assert_eq!(parse_runes("AKBA", ALPHA_NUM), vec![64, 5]);
        assert_eq!(
            parse_runes("AKBAAKBAAKBA", ALPHA_NUM),
            vec![64, 5, 0, 84, 0, 64, 5]
        );
    }

    #[test]
    fn test_generate_runes() {
        assert_eq!(generate_runes(&[0b00000000], FUTHARK), "ᚠᚠ");
        assert_eq!(generate_runes(&[0b00000001], FUTHARK), "ᚢᚠ");
        assert_eq!(generate_runes(&[0b100000], FUTHARK), "ᚠᚢ");
        assert_eq!(generate_runes(&[64, 5], ALPHA_NUM), "AKBA");
        assert_eq!(
            generate_runes(&[64, 5, 0, 84, 0, 64, 5], FUTHARK),
            "ᚠᛁᚢᚠᚠᛁᚢᚠᚠᛁᚢᚠ"
        );
    }
}

#[cfg(test)]
mod cac_tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct TestStruct {
        comments: String,
        code: u32,
    }

    #[test]
    fn test_bytes_and_such() {
        let test = TestStruct {
            comments: "Hello".to_string(),
            code: 42,
        };
        assert_eq!(test.comments.as_bytes(), b"Hello");
        assert_eq!(test.code.to_be_bytes(), [0, 0, 0, 42]);
        assert_eq!(postcard::to_allocvec(&test).unwrap(), b"\x05Hello*");
        assert_eq!(
            create_runes(&serde_json::to_string(&test).unwrap().as_str(), FUTHARK),
            "ᚪᚡᚪᚱᛖᛒᚩᛈᛈᛃᚥᛁᚷᛟᛒᛉᛗᛗᚺᛚᚨᛒᚠᚾᚲᚨᚥᚡᛞᛟᚾᚱᛇᛒᚺᚷᛞᛟᛒᛇᚲᛗᚺᛚᚨᚤᚺᚷᚩᚨ"
        );
    }

    #[test]
    fn test_points() {
        let hello = b"Hello";
        assert_eq!(bytes_to_points(hello), vec![8, 10, 25, 24, 6, 22, 29, 13]);
        let hello_str = b"\x05Hello*";
        assert_eq!(
            bytes_to_points(hello_str),
            vec![5, 0, 18, 10, 6, 22, 17, 13, 15, 19, 10, 0]
        );
    }

    #[test]
    fn test_base_32() {
        let test = TestStruct {
            comments: "Hello".to_string(),
            code: 42,
        };
        assert_eq!(
            simple_generate_runes_ascii(&postcard::to_allocvec(&test).unwrap()),
            "FASKGWRNPTKA"
        );
    }

    #[test]
    fn test_create_runes() {
        let test = TestStruct {
            comments: "Hello".to_string(),
            code: 42,
        };
        // Magic!
        assert_eq!(create_runes(&test, FUTHARK), "ᚲᚠᛖᛁᚷᛞᛒᛈᛊᛗᛁᚠ");
        // Faskgwarniptaka!
        assert_eq!(create_runes(&test, ALPHA_NUM), "FASKGWRNPTKA");
        assert_eq!(create_runes(&"C+c", FUTHARK), "ᚨᚡᛏᛞᛖᛒᚢ");
    }
}
