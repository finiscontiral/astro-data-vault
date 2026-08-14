use std::io;

pub mod line1 {
    /// LINE1の最大行数
    pub const MIN_LEN: usize = 69;

    /// ' ' が入るインデックス
    pub const SPACE_INDICES: [usize; 7] = [8, 17, 32, 43, 52, 61, 63];

    /// '.'が入るインデックス
    pub const DOT_INDICES: [usize; 2] = [23, 34];

    /// '+', '-', ' 'のどれかが入るインデックス
    pub const SIGN_OR_SPACE_INDICES: [usize; 2] = [33, 44];

    /// 指数部の符号が入るインデックス
    /// ここに'+' '-'がない場合は該当インデックスとその次のインデックスが空欄になる
    pub const EXPONENT_INDICES: [usize; 2] = [50, 59];
}

pub mod line2 {
    /// LINE2の最大行数
    pub const MIN_LEN: usize = 69;

    /// ' ' が入るインデックス
    pub const SPACE_INDICES: [usize; 6] = [7, 16, 25, 33, 42, 51];

    /// '.'が入るインデックス
    pub const DOT_INDICES: [usize; 5] = [11, 20, 37, 46, 54];
}

/// TLE (Two-Line Element Set) representation
///
/// # Note
/// - **TODO**: Alpha-5 format is not currently supported. (Planned for future update)
///
/// # Reference
/// Created in accordance with the [Space-Track TLE documentation](https://www.space-track.org/documentation#/tle).
#[derive(Debug)]
pub struct Tle {
    pub name: String,
    pub line1: String,
    pub line2: String,
}

impl Tle {
    pub fn try_new(
        name: impl AsRef<str>,
        line1: impl AsRef<str>,
        line2: impl AsRef<str>,
    ) -> io::Result<Self> {
        let name = name.as_ref().trim().to_string();
        let line1 = line1.as_ref().trim().to_string();
        let line2 = line2.as_ref().trim().to_string();

        Self::validate_fmt(&line1, &line2)?;

        Ok(Self { name, line1, line2 })
    }

    pub fn try_from_str(input: impl AsRef<str>) -> io::Result<Self> {
        let lines: Vec<String> = input
            .as_ref()
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();

        match lines.len() {
            3 => Ok(Self::try_new(&lines[0], &lines[1], &lines[2])?),
            2 => Ok(Self::try_new("UNKNOWN", &lines[0], &lines[1])?),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Invalid TLE format: expected 2 or 3 lines, got {}",
                    lines.len()
                ),
            )),
        }
    }

    pub fn validate_fmt(line1: &str, line2: &str) -> io::Result<()> {
        Self::validate_line1(line1)?;
        Self::validate_line2(line2)?;
        Ok(())
    }

    /// line1のFMTの確認
    pub fn validate_line1(line: &str) -> io::Result<()> {
        let b = line.as_bytes();

        if b.len() < line1::MIN_LEN {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Line 1 is too short",
            ));
        }

        // check starts

        if !b.starts_with(b"1 ") {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid TLE line 1: must start with '1 '",
            ));
        };

        Self::check_expected_char(b, &line1::SPACE_INDICES, b' ').map_err(|idx| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid TLE line 1: expected space at column {}", idx + 1),
            )
        })?;

        Self::check_expected_char(b, &line1::DOT_INDICES, b'.').map_err(|idx| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid TLE line 1: expected . at column {}", idx + 1),
            )
        })?;

        Self::check_any_char(b, &line1::SIGN_OR_SPACE_INDICES, &[b' ', b'+', b'-']).map_err(
            |idx| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "Invalid TLE line 1: expected '+', '-', or space at column {}",
                        idx + 1
                    ),
                )
            },
        )?;
        for &idx in &line1::EXPONENT_INDICES {
            if b[idx] == b' ' {
                if b[idx + 1] != b' ' {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "Invalid TLE line 1: expected consecutive spaces starting at column {}",
                            idx + 1
                        ),
                    ));
                }
            }
        }

        Self::validate_checksum(b)?;

        Ok(())
    }

    /// line2のFMT確認
    pub fn validate_line2(line2: &str) -> io::Result<()> {
        let b = line2.as_bytes();

        if b.len() < line2::MIN_LEN {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Line 2 is too short",
            ));
        }

        // check starts

        if !b.starts_with(b"2 ") {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid TLE line 2: must start with '2 '",
            ));
        };

        Self::check_expected_char(b, &line2::SPACE_INDICES, b' ').map_err(|idx| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid TLE line 2: expected space at column {}", idx + 1),
            )
        })?;

        Self::check_expected_char(b, &line2::DOT_INDICES, b'.').map_err(|idx| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid TLE line 2: expected . at column {}", idx + 1),
            )
        })?;

        Self::validate_checksum(b)?;

        Ok(())
    }

    fn validate_checksum(bytes: &[u8]) -> io::Result<()> {
        if bytes.is_empty() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Line is empty"));
        }

        let last_bytes = bytes[bytes.len() - 1];
        if !last_bytes.is_ascii_digit() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Checksum must be a digit",
            ));
        }

        let expected = last_bytes - b'0';
        let actual = Self::compute_checksum(&bytes[..bytes.len() - 1]);

        if actual != expected {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Checksum mismatch: expected {}, got {}", expected, actual),
            ));
        }
        Ok(())
    }

    fn compute_checksum(bytes: &[u8]) -> u8 {
        let sum: u32 = bytes
            .iter()
            .map(|b| match b {
                b'0'..=b'9' => (b - b'0') as u32,
                b'-' => 1,
                _ => 0,
            })
            .sum();

        (sum % 10) as u8
    }

    /// `bytes`内に指定した`expected`が`indices`の位置にあるか確認する関数
    fn check_expected_char(bytes: &[u8], indices: &[usize], expected: u8) -> Result<(), usize> {
        for &idx in indices {
            if bytes[idx] != expected {
                return Err(idx);
            }
        }
        Ok(())
    }

    /// `bytes`内に指定した`expected_chars`が`indices`の位置にあるか確認する関数
    fn check_any_char(bytes: &[u8], indices: &[usize], expected_chars: &[u8]) -> Result<(), usize> {
        for &idx in indices {
            if !expected_chars.contains(&bytes[idx]) {
                return Err(idx);
            }
        }
        Ok(())
    }
}

impl TryFrom<&str> for Tle {
    type Error = io::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Tle::try_from_str(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test TLE sample.
    /// Source [SPACE-TRACK.ORG](https://www.space-track.org/documentation#/tle)
    /// TLE data listed in the Basic Info section.
    const SAMPLE_TLE_NAME: &str = "ISS (ZARYA)";
    const SAMPLE_TLE_LINE1: &str =
        "1 25544U 98067A   04236.56031392  .00020137  00000-0  16538-3 0  9993";
    const SAMPLE_TLE_LINE2: &str =
        "2 25544  51.6335 344.7760 0007976 126.2523 325.9359 15.70406856328906";

    mod validation {
        use super::*;

        #[test]
        fn test_validate_checksum() {
            let line1_result = Tle::validate_checksum(SAMPLE_TLE_LINE1.as_bytes());
            assert!(line1_result.is_ok());

            let line2_result = Tle::validate_checksum(SAMPLE_TLE_LINE2.as_bytes());
            assert!(line2_result.is_ok());
        }

        #[test]
        fn test_validate_line1() {
            let result = Tle::validate_line1(SAMPLE_TLE_LINE1);
            assert!(
                result.is_ok(),
                "Failed to validate line1: {:?}",
                result.err()
            );
        }

        #[test]
        fn test_validate_line2() {
            let result = Tle::validate_line2(SAMPLE_TLE_LINE2);
            assert!(
                result.is_ok(),
                "Failed to validate line2: {:?}",
                result.err()
            );
        }

        #[test]
        fn test_validate_fmt() {
            let result = Tle::validate_fmt(SAMPLE_TLE_LINE1, SAMPLE_TLE_LINE2);
            assert!(result.is_ok());
        }
    }

    mod compute_checksum {
        use super::*;

        #[test]
        fn test_empty() {
            let bytes = b"";
            let checksum = Tle::compute_checksum(bytes);
            assert_eq!(checksum, 0);
        }

        #[test]
        fn test_space() {
            let bytes = b" ";
            let checksum = Tle::compute_checksum(bytes);
            assert_eq!(checksum, 0);
        }

        #[test]
        fn test_plus() {
            let bytes = b"+";
            let checksum = Tle::compute_checksum(bytes);
            assert_eq!(checksum, 0);
        }

        #[test]
        fn test_minus() {
            let bytes = b"-";
            let checksum = Tle::compute_checksum(bytes);
            assert_eq!(checksum, 1);
        }

        #[test]
        fn test_digits() {
            let bytes = b"0123456789";
            let checksum = Tle::compute_checksum(bytes);
            assert_eq!(checksum, 5);
        }

        #[test]
        fn test_alphabet() {
            let bytes = b"qwertyuiopasdfghjklzxcvbnm";
            let checksum = Tle::compute_checksum(bytes);
            assert_eq!(checksum, 0);
        }

        #[test]
        fn test_exact_ten() {
            // '5' (5) + '5' (5) = 10 -> 10 % 10 = 0
            let bytes = b"55";
            let checksum = Tle::compute_checksum(bytes);
            assert_eq!(checksum, 0);
        }

        #[test]
        fn test_modulo_ten() {
            // '9' (9) + '9' (9) = 18 -> 18 % 10 = 8
            let bytes = b"99";
            let checksum = Tle::compute_checksum(bytes);
            assert_eq!(checksum, 8);
        }
    }
}
