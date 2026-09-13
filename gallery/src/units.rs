//! Byte figures, formatted for a screen.

use alloc::format;
use alloc::string::{String, ToString};

/// How a byte count is written out.
///
/// A screen showing several figures holds one `Units` for all of them: sizes
/// are read against each other, so they have to share a scale. Cycling it is
/// the whole interaction — there is no separate unit picker.
///
/// ```rust
/// # use gallery::Units;
/// let units = Units::default();
/// assert_eq!(units.format(1_048_576), "1,024 KB");
/// assert_eq!(units.next().format(1_048_576), "1,048,576 B");
/// ```
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Units {
    /// Whole bytes, grouped in thousands.
    Bytes,
    /// Kilobytes, rounded to nearest.
    #[default]
    Kilobytes,
}

impl Units {
    /// The next scale in the cycle.
    pub fn next(self) -> Self {
        match self {
            Units::Bytes => Units::Kilobytes,
            Units::Kilobytes => Units::Bytes,
        }
    }

    /// Formats a byte count, always with its unit so the two scales can never
    /// be confused for one another. KB rounds to nearest rather than
    /// truncating, so a figure just under a kilobyte does not read as zero.
    pub fn format(self, bytes: i32) -> String {
        match self {
            Units::Bytes => format!("{} B", Self::grouped(bytes)),
            Units::Kilobytes => format!("{} KB", Self::grouped(Self::kilobytes(bytes))),
        }
    }

    /// To the nearest kilobyte, half away from zero. Integer division truncates
    /// toward zero, so adding half before dividing rounds a negative figure the
    /// wrong way.
    fn kilobytes(bytes: i32) -> i32 {
        let half = if bytes < 0 { -512 } else { 512 };
        bytes.saturating_add(half) / 1024
    }

    /// Digit grouping. `core` has no locale formatting, and a six-figure heap
    /// number is unreadable without separators.
    fn grouped(value: i32) -> String {
        let digits = value.unsigned_abs().to_string();
        let mut out = String::new();

        for (index, digit) in digits.chars().enumerate() {
            if index > 0 && (digits.len() - index).is_multiple_of(3) {
                out.push(',');
            }
            out.push(digit);
        }

        if value < 0 {
            return format!("-{out}");
        }
        out
    }
}
