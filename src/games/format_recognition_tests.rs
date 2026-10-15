// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use super::export::ext_to_format;
use nod::common::Format;

#[test]
fn output_extensions_are_recognized_without_case_sensitivity() {
    for (extension, format) in [
        ("iso", Format::Iso),
        ("ciso", Format::Ciso),
        ("gcz", Format::Gcz),
        ("rvz", Format::Rvz),
        ("wbfs", Format::Wbfs),
        ("wia", Format::Wia),
        ("tgc", Format::Tgc),
    ] {
        assert_eq!(ext_to_format(extension), Some(format));
        assert_eq!(ext_to_format(&extension.to_ascii_uppercase()), Some(format));
        let mut mixed = extension.to_owned();
        mixed[..1].make_ascii_uppercase();
        assert_eq!(ext_to_format(&mixed), Some(format));
    }
    assert_eq!(ext_to_format("CGZ"), None);
    assert_eq!(ext_to_format("zip"), None);
}
