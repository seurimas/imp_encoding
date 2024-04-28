use std::default;

use serde::Serialize;

const BASE_DIACTRICS_START: u32 = 0x0300;
const BASE_DIACTRICS_END: u32 = 0x036F;
const DIACTRICS_BASE: u32 = 0x70;
const ZWJ: char = '\u{200D}';
const ZWSP: char = '\u{200B}';

fn is_diactric(c: char) -> bool {
    let c = c as u32;
    c >= BASE_DIACTRICS_START && c <= BASE_DIACTRICS_END
}

pub fn bytes_to_diactrics_points(bytes: &[u8]) -> Vec<u8> {
    let mut results = Vec::new();
    let mut my_u32 = 0;
    let mut bits = 0;
    for byte in bytes {
        my_u32 = (my_u32 << 8) | (*byte as u32);
        bits += 8;
        if bits == 32 {
            results.push((my_u32 % DIACTRICS_BASE) as u8);
            my_u32 = my_u32 / DIACTRICS_BASE;
            results.push((my_u32 % DIACTRICS_BASE) as u8);
            my_u32 = my_u32 / DIACTRICS_BASE;
            results.push((my_u32 % DIACTRICS_BASE) as u8);
            my_u32 = my_u32 / DIACTRICS_BASE;
            results.push((my_u32 % DIACTRICS_BASE) as u8);
            my_u32 = my_u32 / DIACTRICS_BASE;
            results.push((my_u32 % DIACTRICS_BASE) as u8);
            my_u32 = 0;
            bits = 0;
        }
    }
    if bits == 24 {
        results.push((my_u32 % DIACTRICS_BASE) as u8);
        let my_u32 = my_u32 / DIACTRICS_BASE;
        results.push((my_u32 % DIACTRICS_BASE) as u8);
        let my_u32 = my_u32 / DIACTRICS_BASE;
        results.push((my_u32 % DIACTRICS_BASE) as u8);
        let my_u32 = my_u32 / DIACTRICS_BASE;
        results.push((my_u32 % DIACTRICS_BASE) as u8);
    } else if bits == 16 {
        results.push((my_u32 % DIACTRICS_BASE) as u8);
        let my_u32 = my_u32 / DIACTRICS_BASE;
        results.push((my_u32 % DIACTRICS_BASE) as u8);
        let my_u32 = my_u32 / DIACTRICS_BASE;
        results.push((my_u32 % DIACTRICS_BASE) as u8);
    } else if bits == 8 {
        results.push((my_u32 % DIACTRICS_BASE) as u8);
        let my_u32 = my_u32 / DIACTRICS_BASE;
        results.push((my_u32 % DIACTRICS_BASE) as u8);
    }
    results
}

pub fn diatric_points_to_bytes(points: Vec<u8>) -> Vec<u8> {
    let mut results = Vec::new();
    let mut my_u32 = 0;
    let mut significance = 0;
    for point in points.iter() {
        my_u32 = my_u32 + (*point as u32) * DIACTRICS_BASE.pow(significance);
        significance += 1;
        if significance == 5 {
            results.push((my_u32 >> 24 & 0xff) as u8);
            results.push((my_u32 >> 16 & 0xff) as u8);
            results.push((my_u32 >> 8 & 0xff) as u8);
            results.push((my_u32 & 0xff) as u8);
            my_u32 = 0;
            significance = 0;
        }
    }
    if significance == 4 {
        results.push((my_u32 >> 16 & 0xff) as u8);
        results.push((my_u32 >> 8 & 0xff) as u8);
        results.push((my_u32 & 0xff) as u8);
    } else if significance == 3 {
        results.push((my_u32 >> 8 & 0xff) as u8);
        results.push((my_u32 & 0xff) as u8);
    } else if significance == 2 {
        results.push((my_u32 & 0xff) as u8);
    } else if significance == 1 {
        panic!("Invalid number of diactrics: {} mod 5 == 1", points.len());
    }
    results
}

pub struct CursedConfig {
    diatrics_break: Option<String>,
    max_diactrics: usize,
}

impl Default for CursedConfig {
    fn default() -> Self {
        Self {
            diatrics_break: Some(ZWJ.to_string()),
            max_diactrics: 3,
        }
    }
}

impl CursedConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_zwsp_break(mut self) -> Self {
        self.diatrics_break = Some(ZWSP.to_string());
        self
    }

    pub fn with_zwj_break(mut self) -> Self {
        self.diatrics_break = Some(ZWJ.to_string());
        self
    }

    pub fn with_no_break(mut self) -> Self {
        self.diatrics_break = None;
        self
    }

    pub fn max_diactrics(mut self, max_diactrics: usize) -> Self {
        self.max_diactrics = max_diactrics;
        self
    }

    pub fn can_curse(&self, text_length: usize, data_length: usize) -> bool {
        if self.diatrics_break.is_some() {
            true
        } else {
            text_length > (data_length * self.max_diactrics)
        }
    }

    pub fn generate_curse(&self, text: &str, data: &[u8]) -> String {
        let points = bytes_to_diactrics_points(data);
        let mut cursed_text = String::new();
        let mut point_index = 0;
        let diatrics_per_letter = usize::div_ceil(points.len(), text.len());
        for c in text.chars() {
            cursed_text.push(c);
            for dia_idx in 0..diatrics_per_letter {
                if point_index < points.len() {
                    cursed_text.push(
                        std::char::from_u32(BASE_DIACTRICS_START + points[point_index] as u32)
                            .unwrap(),
                    );
                    point_index += 1;
                }
                if (dia_idx + 1) % self.max_diactrics == 0 {
                    if let Some(diatrics_break) = &self.diatrics_break {
                        cursed_text.push_str(diatrics_break);
                    } else {
                        panic!("Too many diactrics for no break");
                    }
                }
            }
        }
        cursed_text
    }
}

pub fn create_curse<T: Serialize>(t: &T, config: &CursedConfig, text: &str) -> String {
    let data = postcard::to_allocvec(t).unwrap();
    config.generate_curse(text, data.as_slice())
}

pub fn parse_curse_to_points(text: &str) -> Vec<u8> {
    let mut points = Vec::new();
    for c in text.chars() {
        if is_diactric(c) {
            let point = c as u32 - BASE_DIACTRICS_START;
            points.push(point as u8);
        }
    }
    points
}

pub fn read_from_curse<T: serde::de::DeserializeOwned>(text: &str) -> Option<T> {
    let points = parse_curse_to_points(text);
    let bytes = diatric_points_to_bytes(points);
    postcard::from_bytes(&bytes).ok()
}

#[cfg(test)]
mod cursed_tests {
    use rand::random;

    use super::*;

    #[test]
    fn test_is_diatric() {
        assert!(is_diactric('̀'));
        assert!(is_diactric('ͅ'));
        assert!(!is_diactric('a'));
    }

    #[test]
    fn copy_and_display_test() {
        let s = "T\u{0300}\u{0300}\u{0300}\u{200D}\u{0301}\u{0301}\u{0301}\u{200D}\u{0302}\u{0302}\u{0302}\u{200D}\u{0300}\u{0300}\u{0300}e\u{0300}s\u{200D}t";
        println!("{}", s);
        // From terminal
        let s_2 = "T̀̀̀‍́́́‍̂̂̂‍̀̀̀ès‍t";
        assert_eq!(s, s_2);
        // From Discord
        let s_3 = "T̀̀̀‍́́́‍̂̂̂‍̀̀̀ès‍t";
        assert_eq!(s, s_3);
    }

    #[test]
    fn test_bytes_to_diactrics_points() {
        let points = bytes_to_diactrics_points(&[0, 0, 0, 0]);
        assert_eq!(points, vec![0, 0, 0, 0, 0]);
        let points = bytes_to_diactrics_points(&[0, 0, 0, 0, 0]);
        assert_eq!(points, vec![0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_diatric_points_to_bytes() {
        let bytes = diatric_points_to_bytes(vec![0, 0, 0, 0, 0]);
        assert_eq!(bytes, vec![0, 0, 0, 0]);
        let bytes = diatric_points_to_bytes(vec![0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(bytes, vec![0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_bytes_to_bytes() {
        let bytes = [166];
        let points = bytes_to_diactrics_points(&bytes);
        let bytes_2 = diatric_points_to_bytes(points);
        assert_eq!(bytes.to_vec(), bytes_2);

        let bytes = [98, 205, 238];
        let points = bytes_to_diactrics_points(&bytes);
        let bytes_2 = diatric_points_to_bytes(points);
        assert_eq!(bytes.to_vec(), bytes_2);

        let bytes = [62, 10, 105, 133, 98, 205, 238];
        let points = bytes_to_diactrics_points(&bytes);
        let bytes_2 = diatric_points_to_bytes(points);
        assert_eq!(bytes.to_vec(), bytes_2);
    }

    #[test]
    fn overly_cursed() {
        let curse_config = CursedConfig::new().max_diactrics(100).with_zwsp_break();
        let text = "Comments & code";
        let bytes = (0..255).map(|_| random::<u8>()).collect::<Vec<_>>();
        let curse = curse_config.generate_curse(text, &bytes);
        let points = parse_curse_to_points(&curse);
        let bytes_2 = diatric_points_to_bytes(points);
        assert_eq!(bytes, bytes_2);
        println!("{}", curse);
    }

    #[test]
    fn stress_test() {
        let curse_config = CursedConfig::new();
        for _ in 0..1000 {
            let bytes: Vec<u8> = (0..random::<u8>()).map(|_| random::<u8>()).collect();
            let points = bytes_to_diactrics_points(&bytes);
            let bytes_2 = diatric_points_to_bytes(points);
            assert_eq!(bytes, bytes_2);
            let text = "Comments & code";
            let curse = curse_config.generate_curse(text, &bytes);
            let points_2 = parse_curse_to_points(&curse);
            let bytes_3 = diatric_points_to_bytes(points_2);
            assert_eq!(bytes, bytes_3);
        }
    }
}
