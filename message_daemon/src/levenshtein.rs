use std::cmp::min;

pub fn levenshtein(a: &str, b: &str) -> usize {
    let a_size = a.chars().count();
    let b_size = b.chars().count();

    let Some(a_start) = a.chars().nth(0) else {
        return b_size;
    };
    let Some(b_start) = b.chars().nth(0) else {
        return a_size;
    };

    let mut scores = vec![0; a_size * b_size];

    for (i, b_c) in b.chars().enumerate() {
        scores[i] = i + 1 - (a_start == b_c) as usize;
    }

    for (i, a_c) in a.chars().enumerate() {
        scores[i * b_size] = i + 1 - (b_start == a_c) as usize;
    }

    for (i, a_c) in a.chars().enumerate().skip(1) {
        for (j, b_c) in b.chars().enumerate().skip(1) {
            scores[i * b_size + j] = min(
                1 + min(scores[i * b_size + j - 1], scores[i * b_size + j - b_size]),
                (a_c != b_c) as usize + scores[i * b_size + j - b_size - 1],
            );
        }
    }

    println!("{} {} {:?}", a, b, scores);

    scores[a_size * b_size - 1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levenshtein_works_with_empty_strings() {
        assert_eq!(levenshtein("", "kitten"), "kitten".len());
        assert_eq!(levenshtein("kitten", ""), "kitten".len());
        assert_eq!(levenshtein("", ""), 0);
    }

    #[test]
    fn levenshtein_works_with_ascii() {
        assert_eq!(levenshtein("sitting", "kitten"), 3);
    }

    #[test]
    fn levenshtein_works_with_utf() {
        assert_eq!(levenshtein("Kamil", "Kamił"), 1);
    }

    #[test]
    fn levenshtein_works_with_substrings() {
        assert_eq!(levenshtein("alamakota", "kotamaala"), 6);
    }

    #[test]
    fn levenshtein_works_with_different_strings() {
        assert_eq!(levenshtein("aaa", "bb"), 3);
    }

    #[test]
    fn levenshtein_works_with_equal_strings() {
        assert_eq!(
            levenshtein("lorem ipsum sit dolorem", "lorem ipsum sit dolorem"),
            0
        );
    }
}
