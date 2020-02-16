use std::error::Error;

enum State {
    Normal,
    Hex,
    Extract,
}

pub fn shell_interpolate(
    raw: &str,
    f: &dyn Fn(&str) -> Result<String, Box<dyn Error>>,
) -> Result<String, Box<dyn Error>> {
    use State::*;

    let mut interpolated = String::new();
    let mut extracted = String::new();
    let mut state = Normal;

    for c in raw.chars() {
        match c {
            '#' => match state {
                Normal => state = Hex,
                Hex => {
                    interpolated.push('#');
                    state = Normal;
                }
                Extract => {
                    extracted.push('#');
                }
            },
            '{' => match state {
                Normal => interpolated.push('{'),
                Hex => state = Extract,
                Extract => extracted.push('{'),
            },
            '}' => match state {
                Normal => interpolated.push('}'),
                Hex => {
                    state = Normal;
                    interpolated.push_str("#}");
                }
                Extract => {
                    state = Normal;
                    interpolated.push_str(&f(&extracted)?);
                    extracted.clear();
                }
            },
            _ => match state {
                Normal => interpolated.push(c),
                Hex => {
                    state = Normal;
                    interpolated.push('#');
                    interpolated.push(c);
                }
                Extract => extracted.push(c),
            },
        }
    }

    Ok(interpolated)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_interpolate_single() {
        let interpolated = shell_interpolate("#{abc}", &|extract| {
            Ok(if extract == "abc" { "YES".to_string() } else { "NO".to_string() })
        }).unwrap();

        assert_eq!(interpolated, "YES");
    }

    #[test]
    fn test_shell_interpolate_multiple() {
        let interpolated = shell_interpolate("#{abc}:#{def}", &|extract| {
            Ok(if extract == "abc" { "YES".to_string() } else { "NO".to_string() })
        }).unwrap();

        assert_eq!(interpolated, "YES:NO");
    }
}
