use unicode_segmentation::UnicodeSegmentation;

const TOP_LEFT: &str = "\u{250C}\u{250D}\u{250E}\u{250F}";
const TOP_RIGHT: &str = "\u{2510}\u{2511}\u{2512}\u{2513}";
const BOTTOM_LEFT: &str = "\u{2514}\u{2515}\u{2516}\u{2517}";
const BOTTOM_RIGHT: &str = "\u{2518}\u{2519}\u{251A}\u{251B}";
const LEFT: &str = "\u{251C}\u{251D}\u{251E}\u{251F}\u{2520}\u{2521}\u{2522}\u{2523}";
const RIGHT: &str = "\u{2524}\u{2525}\u{2526}\u{2527}\u{2528}\u{2529}\u{252A}\u{252B}";
const TOP: &str = "\u{252C}\u{252D}\u{252E}\u{252F}\u{2530}\u{2531}\u{2532}\u{2533}";
const BOTTOM: &str = "\u{2534}\u{2535}\u{2536}\u{2537}\u{2538}\u{2539}\u{253A}\u{253B}";
const CROSS: &str = "\u{253C}\u{253D}\u{253E}\u{253F}\u{2540}\u{2541}\u{2542}\u{2543}\u{2544}\u{2545}\u{2546}\u{2547}\u{2548}\u{2549}\u{254A}\u{254B}";
const HORIZONTAL: &str = "\u{2500}\u{257C}\u{2501}\u{257E}";
const VERTICAL: &str = "\u{2502}\u{257D}\u{2503}\u{257F}";

#[derive(Default, Debug)]
pub struct BoxLayoutConfig {
    pub min_width: Option<usize>,
    pub max_width: Option<usize>,
    pub min_height: Option<usize>,
    pub max_height: Option<usize>,
    pub aspect_ratio: Option<f32>,
    pub blackouts: Vec<(usize, usize, String)>,
}

const FILLED: &str = "#";

/**
Defines a 2d layout of data vertices.
*/
pub struct BoxLayout(pub Vec<Vec<String>>);

impl BoxLayout {
    pub fn new(width: usize, height: usize) -> Self {
        BoxLayout(vec![vec![FILLED.to_string(); width]; height])
    }

    pub fn width(&self) -> usize {
        self.0[0].len()
    }

    pub fn height(&self) -> usize {
        self.0.len()
    }

    pub fn calculate_bits(&self) -> usize {
        let mut bit_count = 0;
        for (y, row) in self.0.iter().enumerate() {
            for (x, value) in row.iter().enumerate() {
                if value == FILLED {
                    let mut active_neighbors = 0;
                    // We only count towards cells we can't seen yet.
                    // Each neighbor has two bits: one for the neighbor, and one for the current cell.
                    if x < self.0[y].len() - 1 && self.0[y][x + 1] == FILLED {
                        active_neighbors += 2;
                    }
                    if y < self.0.len() - 1 && self.0[y + 1][x] == FILLED {
                        active_neighbors += 2;
                    }
                    bit_count += active_neighbors;
                }
            }
        }
        bit_count
    }

    pub fn estimate_bits(width: usize, height: usize) -> usize {
        let length_wise = (width - 1) * 2 * height;
        let height_wise = (height - 1) * 2 * width;
        length_wise + height_wise
    }

    pub fn is_filled(&self, x: usize, y: usize) -> bool {
        self.0[y][x] == FILLED
    }

    pub fn get_blackout_at(&self, x: usize, y: usize) -> Option<&str> {
        self.0.get(y).and_then(|row| row.get(x)).and_then(|s| {
            if s.eq(FILLED) {
                None
            } else {
                Some(s.as_str())
            }
        })
    }

    pub fn get_connections_at(&self, x: usize, y: usize) -> Option<Connections> {
        if !self.is_filled(x, y) {
            return None;
        }
        let right = x < self.width() - 1 && self.is_filled(x + 1, y);
        let left = x > 0 && self.is_filled(x - 1, y);
        let down = y < self.height() - 1 && self.is_filled(x, y + 1);
        let up = y > 0 && self.is_filled(x, y - 1);
        match (right, left, down, up) {
            (true, true, false, false) => Some(Connections::RightLeft),
            (false, false, true, true) => Some(Connections::DownUp),
            (true, false, true, false) => Some(Connections::RightDown),
            (false, true, true, false) => Some(Connections::LeftDown),
            (true, false, false, true) => Some(Connections::RightUp),
            (false, true, false, true) => Some(Connections::LeftUp),
            (true, true, true, false) => Some(Connections::RightLeftDown),
            (true, true, false, true) => Some(Connections::RightLeftUp),
            (true, false, true, true) => Some(Connections::RightDownUp),
            (false, true, true, true) => Some(Connections::LeftDownUp),
            (true, true, true, true) => Some(Connections::All),
            _ => None,
        }
    }

    // Unlike Base32 futhark encoding, we have variable bits per point.
    pub fn bytes_to_points(&self, bytes: &[u8]) -> Vec<u8> {
        let mut results = Vec::new();
        let mut bits: u32 = 0;
        let mut offset = 0;
        let mut x = 0;
        let mut y = 0;
        for byte in bytes {
            bits |= (*byte as u32) << offset;
            offset += 8;
            'push_bits: loop {
                if let Some(connection) = self.get_connections_at(x, y) {
                    let connection_bits = connection.get_bits();
                    if offset >= connection_bits {
                        results.push((bits & ((1 << connection_bits) - 1)) as u8);
                        bits >>= connection_bits;
                        offset -= connection_bits;
                    } else {
                        break 'push_bits;
                    }
                }
                if x < self.width() - 1 {
                    x += 1;
                } else {
                    x = 0;
                    y += 1;
                }
                if offset == 0 {
                    break 'push_bits;
                }
            }
        }
        results
    }

    pub fn display_bytes(&self, bytes: &[u8]) -> String {
        let mut result = String::new();
        let points = self.bytes_to_points(bytes);
        let mut x = 0;
        let mut y = 0;
        for point in points {
            let mut pushed_point = false;
            while !pushed_point {
                if let Some(blackout) = self.get_blackout_at(x, y) {
                    result.push_str(blackout);
                } else if let Some(connection) = self.get_connections_at(x, y) {
                    result.push(connection.get_character(point));
                    pushed_point = true;
                }
                if x < self.width() - 1 {
                    x += 1;
                } else {
                    x = 0;
                    y += 1;
                    if y < self.height() {
                        result.push('\n');
                    }
                }
            }
        }
        while y < self.height() {
            'push_str: loop {
                if let Some(connection) = self.get_connections_at(x, y) {
                    result.push(connection.get_character(0));
                } else if let Some(blackout) = self.get_blackout_at(x, y) {
                    result.push_str(blackout);
                } else {
                    result.push(' ');
                }
                if x < self.width() - 1 {
                    x += 1;
                } else {
                    break 'push_str;
                }
            }
            x = 0;
            y += 1;
            if y < self.height() {
                result.push('\n');
            }
        }
        result
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Connections {
    RightDown,
    LeftDown,
    RightUp,
    LeftUp,
    DownUp,
    RightLeft,
    RightLeftDown,
    RightLeftUp,
    RightDownUp,
    LeftDownUp,
    All,
}

impl Connections {
    pub fn get_bits(self) -> usize {
        match self {
            Connections::RightDown => 2,
            Connections::LeftDown => 2,
            Connections::RightUp => 2,
            Connections::LeftUp => 2,
            Connections::DownUp => 2,
            Connections::RightLeft => 2,
            Connections::RightLeftDown => 3,
            Connections::RightLeftUp => 3,
            Connections::RightDownUp => 3,
            Connections::LeftDownUp => 3,
            Connections::All => 4,
        }
    }

    pub fn get_character(self, point: u8) -> char {
        let my_chars = match self {
            Connections::RightDown => TOP_LEFT,
            Connections::LeftDown => TOP_RIGHT,
            Connections::RightUp => BOTTOM_LEFT,
            Connections::LeftUp => BOTTOM_RIGHT,
            Connections::DownUp => VERTICAL,
            Connections::RightLeft => HORIZONTAL,
            Connections::RightLeftDown => TOP,
            Connections::RightLeftUp => BOTTOM,
            Connections::RightDownUp => LEFT,
            Connections::LeftDownUp => RIGHT,
            Connections::All => CROSS,
        };
        my_chars
            .graphemes(true)
            .nth(point as usize)
            .unwrap()
            .chars()
            .next()
            .unwrap()
    }
}

pub fn layout_byte_length(length: usize, config: Option<BoxLayoutConfig>) -> Option<BoxLayout> {
    let bit_length = length * 8;
    let mut min_width = config.as_ref().and_then(|c| c.min_width).unwrap_or(2);
    let mut min_height = config.as_ref().and_then(|c| c.min_height).unwrap_or(2);
    let max_width = config
        .as_ref()
        .and_then(|c| c.max_width)
        .unwrap_or(bit_length);
    let max_height = config
        .as_ref()
        .and_then(|c| c.max_height)
        .unwrap_or(bit_length);
    let aspect_ratio = config.as_ref().and_then(|c| c.aspect_ratio).unwrap_or(1.0);
    for (left, top, value) in config
        .as_ref()
        .and_then(|c| Some(c.blackouts.clone()))
        .unwrap_or_default()
    {
        // We want to have a box around any text, so we need to add 1 past that.
        // If the user wants to center the text, they can add their own whitespace.
        min_width = min_width.max(left + value.len() + 1);
        min_height = min_height.max(top + 1);
    }
    // We establish the base layout, with everything filled in...
    let mut layout = BoxLayout::new(min_width, min_height);
    // And then we blackout the areas that the user wants to blackout.
    for (left, top, value) in config
        .as_ref()
        .and_then(|c| Some(c.blackouts.clone()))
        .unwrap_or_default()
    {
        for (i, c) in value.chars().enumerate() {
            layout.0[top][left + i] = c.to_string();
        }
    }
    while layout.calculate_bits() < bit_length
        && !(layout.height() >= max_height && layout.width() >= max_width)
    {
        let current_aspect_ratio = layout.height() as f32 / layout.width() as f32;
        let new_row = (current_aspect_ratio < aspect_ratio && layout.height() < max_height)
            || layout.width() >= max_width;
        if new_row {
            // We need to add a row.
            layout.0.push(vec![FILLED.to_string(); layout.width()]);
        } else {
            // We need to add a column.
            for row in layout.0.iter_mut() {
                row.push(FILLED.to_string());
            }
        }
    }
    if layout.calculate_bits() >= bit_length
        && layout.height() <= max_height
        && layout.width() <= max_width
    {
        Some(layout)
    } else {
        None
    }
}

pub fn generate_boxes(bytes: &[u8], config: Option<BoxLayoutConfig>) -> String {
    let layout = layout_byte_length(bytes.len(), config).unwrap();
    layout.display_bytes(bytes)
}

pub fn create_boxes<T: serde::Serialize>(t: &T, config: Option<BoxLayoutConfig>) -> String {
    let data = postcard::to_allocvec(t).unwrap();
    generate_boxes(data.as_slice(), config)
}

pub fn create_boxes_with_layout<T: serde::Serialize>(t: &T, layout: BoxLayout) -> String {
    let data = postcard::to_allocvec(t).unwrap();
    layout.display_bytes(data.as_slice())
}

fn point_from_grapheme_in_set(grapheme: &str, set: &str) -> u8 {
    set.graphemes(true)
        .position(|g| g == grapheme)
        .map(|p| p as u8)
        .unwrap()
}

pub fn parse_boxes_to_points(s: &str) -> Vec<(u8, usize)> {
    let mut points = Vec::new();
    for grapheme in s.graphemes(true) {
        if CROSS.contains(grapheme) {
            points.push((point_from_grapheme_in_set(grapheme, CROSS), 4));
        } else if LEFT.contains(grapheme) {
            points.push((point_from_grapheme_in_set(grapheme, LEFT), 3));
        } else if RIGHT.contains(grapheme) {
            points.push((point_from_grapheme_in_set(grapheme, RIGHT), 3));
        } else if TOP.contains(grapheme) {
            points.push((point_from_grapheme_in_set(grapheme, TOP), 3));
        } else if BOTTOM.contains(grapheme) {
            points.push((point_from_grapheme_in_set(grapheme, BOTTOM), 3));
        } else if TOP_LEFT.contains(grapheme) {
            points.push((point_from_grapheme_in_set(grapheme, TOP_LEFT), 2));
        } else if TOP_RIGHT.contains(grapheme) {
            points.push((point_from_grapheme_in_set(grapheme, TOP_RIGHT), 2));
        } else if BOTTOM_LEFT.contains(grapheme) {
            points.push((point_from_grapheme_in_set(grapheme, BOTTOM_LEFT), 2));
        } else if BOTTOM_RIGHT.contains(grapheme) {
            points.push((point_from_grapheme_in_set(grapheme, BOTTOM_RIGHT), 2));
        } else if HORIZONTAL.contains(grapheme) {
            points.push((point_from_grapheme_in_set(grapheme, HORIZONTAL), 2));
        } else if VERTICAL.contains(grapheme) {
            points.push((point_from_grapheme_in_set(grapheme, VERTICAL), 2));
        }
        // Ignore non-box characters.
    }
    points
}

pub fn box_points_to_bytes(points: &[(u8, usize)]) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut bits = 0;
    let mut offset = 0;
    for (point, bits_per_point) in points {
        if offset == 0 {
            bits = *point as u32;
            offset = *bits_per_point;
        } else {
            bits |= (*point as u32) << offset;
            offset += *bits_per_point;
        }
        while offset >= 8 {
            bytes.push((bits & 0xff) as u8);
            bits >>= 8;
            offset -= 8;
        }
    }
    bytes
}

pub fn parse_boxes<T: serde::de::DeserializeOwned>(s: &str) -> Result<T, postcard::Error> {
    let points = parse_boxes_to_points(s);
    let bytes = box_points_to_bytes(&points);
    postcard::from_bytes(&bytes)
}

pub fn gen_layout(value: &str) -> BoxLayout {
    BoxLayout(
        value
            .split('\n')
            .map(|row| {
                row.chars()
                    .filter(|c| !c.is_whitespace())
                    .map(|c| c.to_string())
                    .collect()
            })
            .collect(),
    )
}

#[cfg(test)]
mod boxes_tests {
    use super::*;

    #[test]
    fn test_box_chars() {
        assert_eq!(TOP_LEFT, "┌┍┎┏");
        assert_eq!(TOP_RIGHT, "┐┑┒┓");
        assert_eq!(BOTTOM_LEFT, "└┕┖┗");
        assert_eq!(BOTTOM_RIGHT, "┘┙┚┛");
        assert_eq!(LEFT, "├┝┞┟┠┡┢┣");
        assert_eq!(RIGHT, "┤┥┦┧┨┩┪┫");
        assert_eq!(TOP, "┬┭┮┯┰┱┲┳");
        assert_eq!(BOTTOM, "┴┵┶┷┸┹┺┻");
        assert_eq!(CROSS, "┼┽┾┿╀╁╂╃╄╅╆╇╈╉╊╋");
        assert_eq!(HORIZONTAL, "─╼━╾");
        assert_eq!(VERTICAL, "│╽┃╿");
    }

    #[test]
    fn test_calculate_bits_in_layout() {
        let layout = gen_layout(
            "##\n\
             ##",
        );
        assert_eq!(layout.calculate_bits(), 8);
        assert_eq!(BoxLayout::estimate_bits(2, 2), 8);
        let layout = gen_layout(
            "####\n\
             #XX#\n\
             ####",
        );
        assert_eq!(layout.calculate_bits(), 20);
        let layout = gen_layout(
            "####\n\
             ####\n\
             ####",
        );
        assert_eq!(layout.calculate_bits(), 34);
        assert_eq!(BoxLayout::estimate_bits(4, 3), 34);
    }

    #[test]
    fn test_bytes_to_points() {
        let layout = gen_layout(
            "##\n\
             ##",
        );
        assert_eq!(layout.bytes_to_points(&[0b01010101]), vec![1, 1, 1, 1]);
        assert_eq!(layout.bytes_to_points(&[0b11110000]), vec![0, 0, 3, 3]);
        let layout = gen_layout(
            "####\n\
             #XX#\n\
             ####",
        );
        assert_eq!(
            layout.bytes_to_points(&[0b01010101, 0b01010101]),
            vec![1, 1, 1, 1, 1, 1, 1, 1]
        );
        assert_eq!(
            layout.bytes_to_points(&[0b11110000, 0b11110000]),
            vec![0, 0, 3, 3, 0, 0, 3, 3]
        );
    }

    #[test]
    fn test_display_bytes() {
        let layout = gen_layout(
            "##\n\
             ##",
        );
        assert_eq!(layout.display_bytes(&[0]), "┌┐\n└┘");
        assert_eq!(layout.display_bytes(&[0b01010101]), "┍┑\n┕┙");
        assert_eq!(layout.display_bytes(&[0b11110000]), "┌┐\n┗┛");
        let layout = gen_layout(
            "####\n\
             #XX#\n\
             ####",
        );
        assert_eq!(
            layout.display_bytes(&[0b01010101, 0b01010101]),
            "┍╼╼┑\n╽XX╽\n┕╼─┘"
        );
        let layout = gen_layout(
            "####\n\
             ####\n\
             #XX#",
        );
        assert_eq!(
            layout.display_bytes(&[0b11110000, 0b11110000]),
            "┌┰┳┐\n┠┻┴┤\n XX "
        );
        let layout = gen_layout(
            "####\n\
             ####\n\
             ##XX",
        );
        assert_eq!(
            layout.display_bytes(&[0b11110000, 0b11110000]),
            "┌┰┳┐\n┠┼┴┘\n└┘XX"
        );
    }

    #[test]
    fn test_layout_data() {
        let config = BoxLayoutConfig {
            min_width: Some(4),
            min_height: Some(3),
            max_width: None,
            max_height: None,
            aspect_ratio: Some(1.0),
            blackouts: vec![(1, 1, "Hello".to_string())],
        };
        let layout = layout_byte_length(8, Some(config)).unwrap();
        assert_eq!(layout.width(), 7);
        assert_eq!(layout.height(), 5);
        assert_eq!(layout.calculate_bits(), 84);

        let layout = layout_byte_length(8, None).unwrap();
        assert_eq!(layout.width(), 5);
        assert_eq!(layout.height(), 5);
        assert_eq!(layout.calculate_bits(), 80);

        let config = BoxLayoutConfig {
            max_width: Some(80),
            max_height: Some(5),
            ..Default::default()
        };
        let layout = layout_byte_length(150, Some(config)).unwrap();
        assert_eq!(layout.width(), 68);
        assert_eq!(layout.height(), 5);
        assert_eq!(layout.calculate_bits(), 1214);
    }

    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct TestStruct {
        comments: String,
        code: u32,
    }

    #[test]
    fn test_create_boxes() {
        let test = TestStruct {
            comments: "Hello".to_string(),
            code: 42,
        };
        let config = BoxLayoutConfig {
            min_width: Some(4),
            min_height: Some(3),
            max_width: None,
            max_height: None,
            aspect_ratio: Some(1.0),
            blackouts: vec![(1, 1, " C+c ".to_string())],
        };
        let boxes = create_boxes(&test, Some(config));
        assert_eq!(
            boxes,
            "┍╼───━┐\n\
             ╽ C+c ╽\n\
             ┝┯┰┱┭┲┪\n\
             ┖┻┺┸┶┵┘"
        );
    }

    #[test]
    fn test_parse_boxes_to_points() {
        let boxes = "┍╼───━┐\n\
                           ╽ C+c ╽\n\
                           ┝┯┰┱┭┲┪\n\
                           ┖┻┺┸┶┵┘";
        assert_eq!(
            parse_boxes_to_points(boxes),
            vec![
                (1, 2),
                (1, 2),
                (0, 2),
                (0, 2),
                (0, 2),
                (2, 2),
                (0, 2),
                (1, 2),
                (1, 2),
                (1, 3),
                (3, 3),
                (4, 3),
                (5, 3),
                (1, 3),
                (6, 3),
                (6, 3),
                (2, 2),
                (7, 3),
                (6, 3),
                (4, 3),
                (2, 3),
                (1, 3),
                (0, 2)
            ]
        );
    }

    #[test]
    fn test_box_points_to_bytes() {
        let box_points = parse_boxes_to_points("┌┐\n└┘");
        assert_eq!(box_points_to_bytes(&box_points), vec![0b00000000]);
        let box_points = parse_boxes_to_points("┍╼╼┑\n╽XX╽\n┕╼─┘");
        assert_eq!(box_points_to_bytes(&box_points), [0b01010101, 0b01010101]);
    }

    #[test]
    fn test_parse_boxes() {
        let boxes = "┍╼───━┐\n\
                           ╽ C+c ╽\n\
                           ┝┯┰┱┭┲┪\n\
                           ┖┻┺┸┶┵┘";
        let test: TestStruct = parse_boxes(boxes).unwrap();
        assert_eq!(
            test,
            TestStruct {
                comments: "Hello".to_string(),
                code: 42
            }
        );
    }
}

fn simple_get_connections_at(
    width: usize,
    height: usize,
    x: usize,
    y: usize,
) -> Option<Connections> {
    let right = x < width - 1;
    let left = x > 0;
    let down = y < height - 1;
    let up = y > 0;
    match (right, left, down, up) {
        (true, false, true, false) => Some(Connections::RightDown),
        (false, true, true, false) => Some(Connections::LeftDown),
        (true, false, false, true) => Some(Connections::RightUp),
        (false, true, false, true) => Some(Connections::LeftUp),
        (true, true, true, false) => Some(Connections::RightLeftDown),
        (true, true, false, true) => Some(Connections::RightLeftUp),
        (true, false, true, true) => Some(Connections::RightDownUp),
        (false, true, true, true) => Some(Connections::LeftDownUp),
        (true, true, true, true) => Some(Connections::All),
        _ => None,
    }
}

fn simple_bytes_to_points(width: usize, height: usize, bytes: &[u8]) -> Vec<u8> {
    let mut results = Vec::new();
    let mut bits: u32 = 0;
    let mut offset = 0;
    let mut x = 0;
    let mut y = 0;
    for byte in bytes {
        bits |= (*byte as u32) << offset;
        offset += 8;
        push_points_and_move_cursor(
            width,
            height,
            &mut x,
            &mut y,
            &mut offset,
            &mut results,
            &mut bits,
        );
    }
    results
}

fn push_points_and_move_cursor(
    width: usize,
    height: usize,
    x: &mut usize,
    y: &mut usize,
    offset: &mut usize,
    results: &mut Vec<u8>,
    bits: &mut u32,
) {
    loop {
        if let Some(connection) = simple_get_connections_at(width, height, *x, *y) {
            let connection_bits = connection.get_bits();
            if *offset >= connection_bits {
                results.push((*bits & ((1 << connection_bits) - 1)) as u8);
                *bits >>= connection_bits;
                *offset -= connection_bits;
            } else {
                break;
            }
        }
        if *x < width - 1 {
            *x += 1;
        } else {
            *x = 0;
            *y += 1;
        }
        if *offset == 0 {
            break;
        }
    }
}

fn simple_display_bytes(width: usize, height: usize, bytes: &[u8]) -> String {
    let mut result = String::new();
    let points = simple_bytes_to_points(width, height, bytes);
    let mut x = 0;
    let mut y = 0;
    for point in points {
        let mut pushed_point = false;
        while !pushed_point {
            if let Some(connection) = simple_get_connections_at(width, height, x, y) {
                result.push(connection.get_character(point));
                pushed_point = true;
            }
            if x < width - 1 {
                x += 1;
            } else {
                x = 0;
                y += 1;
                if y < height {
                    result.push('\n');
                }
            }
        }
    }
    while y < height {
        'push_str: loop {
            if let Some(connection) = simple_get_connections_at(width, height, x, y) {
                result.push(connection.get_character(0));
            } else {
                result.push(' ');
            }
            if x < width - 1 {
                x += 1;
            } else {
                break 'push_str;
            }
        }
        x = 0;
        y += 1;
        if y < height {
            result.push('\n');
        }
    }
    result
}

#[cfg(test)]
mod cac_tests {
    use super::*;
    #[test]
    fn draw_some_boxes() {
        let layout = gen_layout(
            "##\n\
             ##",
        );
        println!("{}", layout.display_bytes(&[0]));
        let layout = gen_layout(
            "###\n\
             #.#\n\
             ###",
        );
        println!("{}", layout.display_bytes(&[0, 0]));
        let layout = gen_layout(
            "#####\n\
             #####\n\
             ###..\n\
             ###..",
        );
        // Editted in article.
        println!("{}", layout.display_bytes(&[255, 255, 255, 255]));
        let layout = gen_layout(
            "#####\n\
             #####\n\
             #####\n\
             #####\n\
             #####",
        );
        println!("{}", layout.display_bytes(&[255, 255, 255, 255]));
        let layout = gen_layout(
            "###\n\
             ###\n\
             ###",
        );
        println!("{}", layout.display_bytes(&[0, 0]));
    }
}
