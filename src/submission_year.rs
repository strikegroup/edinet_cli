// SPDX-FileCopyrightText: 2026 Strike Co., Ltd.
// SPDX-FileCopyrightText: 2026 Strike Group Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

/// CLI で指定する4桁の提出年。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SubmissionYear(u16);

impl SubmissionYear {
    pub fn get(self) -> u16 {
        self.0
    }
}

impl std::str::FromStr for SubmissionYear {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.len() != 4 || !value.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err("year must be a four-digit value such as 2025".to_owned());
        }

        let year = value
            .parse::<u16>()
            .map_err(|_| "year must be a four-digit value such as 2025".to_owned())?;
        if !(1000..=9999).contains(&year) {
            return Err("year must be between 1000 and 9999".to_owned());
        }

        Ok(Self(year))
    }
}

#[cfg(test)]
mod tests {
    use super::SubmissionYear;

    #[test]
    fn parses_four_digit_year() {
        assert_eq!("2025".parse::<SubmissionYear>().unwrap().get(), 2025);
    }

    #[test]
    fn rejects_non_four_digit_year() {
        for value in ["25", "02025", "year", "0000"] {
            assert!(value.parse::<SubmissionYear>().is_err());
        }
    }
}
