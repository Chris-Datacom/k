//! Compilation targets supported by the K compiler.

use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Target {
    X86_64SystemV,
    X86_64KrumpyOs,
    Aarch64KrumpyOs,
}

impl Target {
    pub fn name(self) -> &'static str {
        match self {
            Self::X86_64SystemV => "x86_64-unknown-linux-gnu",
            Self::X86_64KrumpyOs => "x86_64-krumpyos",
            Self::Aarch64KrumpyOs => "aarch64-krumpyos",
        }
    }

    pub fn is_implemented(self) -> bool {
        matches!(self, Self::X86_64SystemV | Self::X86_64KrumpyOs)
    }
}

impl Default for Target {
    fn default() -> Self {
        Self::X86_64SystemV
    }
}

impl FromStr for Target {
    type Err = TargetParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "x86_64-unknown-linux-gnu" | "linux-x86_64" => Ok(Self::X86_64SystemV),
            "x86_64-krumpyos" | "krumpyos-x86_64" => Ok(Self::X86_64KrumpyOs),
            "aarch64-krumpyos" | "krumpyos-aarch64" => Ok(Self::Aarch64KrumpyOs),
            _ => Err(TargetParseError {
                value: value.to_owned(),
            }),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetParseError {
    value: String,
}

impl fmt::Display for TargetParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "unknown target `{}`", self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::Target;
    use std::str::FromStr;

    #[test]
    fn parses_supported_target_names() {
        assert_eq!(
            Target::from_str("x86_64-krumpyos").unwrap(),
            Target::X86_64KrumpyOs
        );
        assert_eq!(
            Target::from_str("aarch64-krumpyos").unwrap(),
            Target::Aarch64KrumpyOs
        );
    }

    #[test]
    fn marks_aarch64_as_not_implemented_yet() {
        assert!(!Target::Aarch64KrumpyOs.is_implemented());
        assert!(Target::X86_64KrumpyOs.is_implemented());
    }
}
