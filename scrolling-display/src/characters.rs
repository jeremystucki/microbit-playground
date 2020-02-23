pub fn get_single_column_character(character: char) -> Option<[u8; 5]> {
    match character {
        '1' => Some([1, 1, 1, 1, 1]),
        _ => None,
    }
}

pub fn get_double_column_character(character: char) -> Option<[[u8; 5]; 2]> {
    match character {
        _ => None,
    }
}

pub fn get_triple_column_character(character: char) -> Option<[[u8; 5]; 3]> {
    match character {
        '0' => Some([[1, 1, 1, 1, 1], [1, 0, 0, 0, 1], [1, 1, 1, 1, 1]]),
        '2' => Some([[1, 0, 1, 1, 1], [1, 0, 1, 0, 1], [1, 1, 1, 0, 1]]),
        '3' => Some([[1, 0, 0, 0, 1], [1, 0, 1, 0, 1], [1, 1, 1, 1, 1]]),
        '4' => Some([[1, 1, 1, 0, 0], [0, 0, 1, 0, 0], [1, 1, 1, 1, 1]]),
        '5' => Some([[1, 1, 1, 0, 1], [1, 0, 1, 0, 1], [1, 0, 1, 1, 1]]),
        '6' => Some([[1, 1, 1, 1, 1], [1, 0, 1, 0, 1], [1, 0, 1, 1, 1]]),
        '7' => Some([[1, 0, 0, 0, 0], [1, 0, 0, 0, 0], [1, 1, 1, 1, 1]]),
        '8' => Some([[1, 1, 1, 1, 1], [1, 0, 1, 0, 1], [1, 1, 1, 1, 1]]),
        '9' => Some([[1, 1, 1, 0, 1], [1, 0, 1, 0, 1], [1, 1, 1, 1, 1]]),
        _ => None,
    }
}
