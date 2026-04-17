use serde::{Deserialize, Serialize};

/// UI color palette persisted in `wsdd-config.json`.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub enum AppTheme {
    /// Neutral dark gray palette.
    #[default]
    NeutralDark,
    /// Dark palette with an indigo tint.
    BlueTint,
    /// Warm dark palette.
    WarmDark,
    /// Light operating-system style palette.
    Light,
}

impl AppTheme {
    /// Returns the label shown in the theme selector.
    pub fn display_name(self) -> &'static str {
        match self {
            Self::NeutralDark => "Dark Neutral",
            Self::BlueTint => "Dark Blue",
            Self::WarmDark => "Dark Warm",
            Self::Light => "Light",
        }
    }

    /// Returns all supported UI themes.
    pub fn all() -> &'static [Self] {
        &[
            Self::NeutralDark,
            Self::BlueTint,
            Self::WarmDark,
            Self::Light,
        ]
    }
}
