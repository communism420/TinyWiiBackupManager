// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use nod::common::Format;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum SaveFormat {
    #[default]
    Rvz,
    Iso,
    Wbfs,
    Ciso,
    Gcz,
    Wia,
}

impl SaveFormat {
    pub(super) const ALL: [Self; 6] = [
        Self::Rvz,
        Self::Iso,
        Self::Wbfs,
        Self::Ciso,
        Self::Gcz,
        Self::Wia,
    ];

    pub(super) fn extension(self) -> &'static str {
        match self {
            Self::Rvz => "rvz",
            Self::Iso => "iso",
            Self::Wbfs => "wbfs",
            Self::Ciso => "ciso",
            Self::Gcz => "gcz",
            Self::Wia => "wia",
        }
    }

    fn format(self) -> Format {
        match self {
            Self::Rvz => Format::Rvz,
            Self::Iso => Format::Iso,
            Self::Wbfs => Format::Wbfs,
            Self::Ciso => Format::Ciso,
            Self::Gcz => Format::Gcz,
            Self::Wia => Format::Wia,
        }
    }
}

impl std::fmt::Display for SaveFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format().fmt(f)
    }
}
