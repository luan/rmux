use rmux_core::TerminalScreen;
use rmux_proto::{TerminalDefaultColors, TerminalSize};

const COLORS: TerminalDefaultColors = TerminalDefaultColors {
    foreground: [0xaa, 0xbb, 0xcc],
    background: [0x1a, 0x1b, 0x26],
    cursor: [0x12, 0x34, 0x56],
};

#[test]
fn host_colors_and_cursor_reports_reply_in_query_order() {
    let mut terminal = TerminalScreen::new(TerminalSize { cols: 80, rows: 24 }, 10);
    terminal.set_default_colors(Some(COLORS));
    terminal.feed(b"\x1b]11;?\x1b\\\x1b[6n\x1b]10;?\x07\x1b]12;?\x1b\\");
    assert_eq!(terminal.take_replies(), b"\x1b]11;rgb:1a1a/1b1b/2626\x1b\\\x1b[1;1R\x1b]10;rgb:aaaa/bbbb/cccc\x07\x1b]12;rgb:1212/3434/5656\x1b\\");
}

#[test]
fn application_colors_override_live_host_defaults_and_reset_to_them() {
    let mut terminal = TerminalScreen::new(TerminalSize { cols: 80, rows: 24 }, 10);
    terminal.set_default_colors(Some(COLORS));
    terminal.feed(b"\x1b]11;#224466\x1b\\");
    terminal.set_default_colors(Some(TerminalDefaultColors {
        background: [3, 4, 5],
        ..COLORS
    }));
    terminal.feed(b"\x1b]11;?\x1b\\\x1b]111\x1b\\\x1b]11;?\x1b\\");
    assert_eq!(
        terminal.take_replies(),
        b"\x1b]11;#224466\x1b\\\x1b]11;rgb:0303/0404/0505\x1b\\"
    );
    terminal.set_default_colors(None);
    terminal.feed(b"\x1b]11;?\x1b\\\x1b[6n");
    assert_eq!(terminal.take_replies(), b"\x1b[1;1R");
}

#[test]
fn default_color_metadata_round_trips_and_rejects_untrusted_values() {
    assert_eq!(
        TerminalDefaultColors::decode(&COLORS.encode()),
        Some(COLORS)
    );
    for invalid in [
        "",
        "+aaaaa;1a1b26;123456",
        "aabbcc;1a1b26;1234567",
        "aabbcc;zzzzzz;123456",
        "aabbcc;1a1b26;123456\x1b",
        "aabbcc,1a1b26,123456",
    ] {
        assert_eq!(TerminalDefaultColors::decode(invalid), None, "{invalid:?}");
    }
}
