/// Calculate Levenshtein distance between two strings
pub fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.len();
    let len2 = s2.len();
    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    for (i, row) in matrix.iter_mut().enumerate().take(len1 + 1) {
        row[0] = i;
    }
    for (j, cell) in matrix[0].iter_mut().enumerate().take(len2 + 1) {
        *cell = j;
    }

    for (i, c1) in s1.chars().enumerate() {
        for (j, c2) in s2.chars().enumerate() {
            let cost = usize::from(c1 != c2);
            matrix[i + 1][j + 1] = (matrix[i][j + 1] + 1)
                .min(matrix[i + 1][j] + 1)
                .min(matrix[i][j] + cost);
        }
    }

    matrix[len1][len2]
}

/// Find the closest matching annotation
pub fn find_closest_annotation(input: &str) -> Option<&'static str> {
    const ANNOTATIONS: &[&str] = &[
        "response",
        "example",
        "tag",
        "security",
        "id",
        "hidden",
        "rovo-ignore",
    ];

    let input_lower = input.to_lowercase();
    let mut best_match = None;
    let mut best_distance = usize::MAX;

    for &annotation in ANNOTATIONS {
        let distance = levenshtein_distance(&input_lower, annotation);
        // Only suggest if distance is small (≤ 2 characters different)
        if distance < best_distance && distance <= 2 {
            best_distance = distance;
            best_match = Some(annotation);
        }
    }

    best_match
}
