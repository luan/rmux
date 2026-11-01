//! Terminal host defaults transported through the pane-option API.

/// Pane-local option carrying foreground, background and cursor RGB defaults.
pub const TERMINAL_DEFAULT_COLORS_OPTION: &str = "@rmux-terminal-default-colors";

/// Actual terminal host colors, used when an application has not set an OSC color.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TerminalDefaultColors {
    /// Default foreground RGB.
    pub foreground: [u8; 3],
    /// Default background RGB.
    pub background: [u8; 3],
    /// Default cursor RGB.
    pub cursor: [u8; 3],
}

impl TerminalDefaultColors {
    /// Encodes the bounded pane-option value as three six-digit RGB colors.
    #[must_use]
    pub fn encode(self) -> String {
        let [r, g, b] = self.foreground;
        let [br, bg, bb] = self.background;
        let [cr, cg, cb] = self.cursor;
        format!("{r:02x}{g:02x}{b:02x};{br:02x}{bg:02x}{bb:02x};{cr:02x}{cg:02x}{cb:02x}")
    }

    /// Decodes exactly three six-digit RGB colors, rejecting other values.
    #[must_use]
    pub fn decode(value: &str) -> Option<Self> {
        if value.len() != 20
            || !value.is_ascii()
            || value.as_bytes()[6] != b';'
            || value.as_bytes()[13] != b';'
        {
            return None;
        }
        let color = |digits: &str| {
            if !digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                return None;
            }
            let [_, r, g, b] = u32::from_str_radix(digits, 16).ok()?.to_be_bytes();
            Some([r, g, b])
        };
        Some(Self {
            foreground: color(&value[..6])?,
            background: color(&value[7..13])?,
            cursor: color(&value[14..])?,
        })
    }
}
