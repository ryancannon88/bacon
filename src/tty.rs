use {
    crate::*,
    anyhow::*,
    std::io::Write,
    termimad::StrFit,
    vte,
};

pub const CSI_RESET: &str = "\u{1b}[0m\u{1b}[0m";
pub const CSI_BOLD: &str = "\u{1b}[1m";
pub const CSI_BOLD_RED: &str = "\u{1b}[1m\u{1b}[38;5;9m";
pub const CSI_BOLD_YELLOW: &str = "\u{1b}[1m\u{1b}[33m";
pub const CSI_BOLD_BLUE: &str = "\u{1b}[1m\u{1b}[38;5;12m";

/// a simple representation of a colored and styled string.
///
/// Note that this works because of a few properties of
/// cargo's output:
/// - styles and colors are always reset on changes
/// - they're always in the same order (bold then fg color)
/// A more generic parsing would have to:
/// - parse the csi params (it's simple enough to map but takes code)
/// - use a simple state machine to keep style (bold, italic, etc.),
///    foreground color, and background color across tstrings
#[derive(Debug, Default)]
pub struct TString {
    pub csi: String,
    pub raw: String,
}
impl TString {
    pub fn push_csi(&mut self, params: &[i64], action: char) {
        self.csi.push('\u{1b}');
        self.csi.push('[');
        for (idx, p) in params.iter().enumerate() {
            self.csi.push_str(&format!("{}", p));
            if idx < params.len() - 1 {
                self.csi.push(';');
            }
        }
        self.csi.push(action);
    }
    pub fn draw(&self, w: &mut W) -> Result<()> {
        if self.csi.is_empty() {
            write!(w, "{}", &self.raw)?;
        } else {
            write!(
                w,
                "{}{}{}",
                &self.csi,
                &self.raw,
                CSI_RESET,
            )?;
        }
        Ok(())
    }
    /// draw the string but without taking more than cols_max cols.
    /// Return the number of cols written
    pub fn draw_in(&self, w: &mut W, cols_max: usize) -> Result<usize> {
        let fit = StrFit::make_cow(&self.raw, cols_max);
        if self.csi.is_empty() {
            write!(w, "{}", &fit.0)?;
        } else {
            write!(
                w,
                "{}{}{}",
                &self.csi,
                &fit.0,
                CSI_RESET,
            )?;
        }
        Ok(fit.1)
    }
    pub fn starts_with(&self, csi: &str, raw: &str) -> bool {
        self.csi == csi && self.raw.starts_with(raw)
    }
    pub fn split_off(&mut self, at: usize) -> Self {
        Self {
            csi: self.csi.clone(),
            raw: self.raw.split_off(at),
        }
    }
}

/// a simple representation of a line made of homogeneous parts.
///
/// Note that this does only manages CSI and SGR components
/// and isn't a suitable representation for an arbitrary
/// terminal input or output.
/// I recommend you to NOT try to reuse this hack in another
/// project unless you perfectly understand it.
#[derive(Debug, Default)]
pub struct TLine {
    pub strings: Vec<TString>,
}

impl TLine {
    pub fn from_tty(tty: &str) -> Self {
        let mut parser = vte::Parser::new();
        let mut builder = TLineBuilder::default();
        for byte in tty.bytes() {
            parser.advance(&mut builder, byte);
        }
        builder.to_tline()
    }
    pub fn draw(&self, w: &mut W) -> Result<()> {
        for ts in &self.strings {
            ts.draw(w)?;
        }
        Ok(())
    }
    /// draw the line but without taking more than cols_max cols.
    /// Return the number of cols written
    pub fn draw_in(&self, w: &mut W, cols_max: usize) -> Result<usize> {
        let mut cols = 0;
        for ts in &self.strings {
            if cols >= cols_max {
                break;
            }
            cols += ts.draw_in(w, cols_max - cols)?;
        }
        Ok(cols)
    }
}

#[derive(Debug, Default)]
pub struct TLineBuilder {
    cur: Option<TString>,
    strings: Vec<TString>,
}
impl TLineBuilder {
    pub fn to_tline(mut self) -> TLine {
        if let Some(cur) = self.cur {
            self.strings.push(cur);
        }
        TLine {
            strings: self.strings,
        }
    }
}
impl vte::Perform for TLineBuilder {
    fn print(&mut self, c: char) {
        self.cur
            .get_or_insert_with(TString::default)
            .raw
            .push(c);
    }
    fn csi_dispatch(&mut self, params: &[i64], _intermediates: &[u8], _ignore: bool, action: char) {
        if *params == [0] {
            if let Some(cur) = self.cur.take() {
                self.strings.push(cur);
            }
            return;
        }
        if let Some(cur) = self.cur.as_mut() {
            if cur.raw.is_empty() {
                cur.push_csi(params, action);
                return;
            }
        }
        if let Some(cur) = self.cur.take() {
            self.strings.push(cur);
        }
        let mut cur = TString::default();
        cur.push_csi(params, action);
        self.cur = Some(cur);
    }
    fn execute(&mut self, _byte: u8) {}
    fn hook(&mut self, _params: &[i64], _intermediates: &[u8], _ignore: bool, _action: char) {}
    fn put(&mut self, _byte: u8) {}
    fn unhook(&mut self) {}
    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {}
    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {}
}
