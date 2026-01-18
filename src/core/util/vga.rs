//! フレームバッファ出力

use core::fmt;
use spin::{Mutex, Once};

/// フレームバッファ情報
static FB_INFO: Once<FramebufferInfo> = Once::new();

#[derive(Clone, Copy)]
struct FramebufferInfo {
    addr: u64,
    width: usize,
    height: usize,
    stride: usize,
}

/// フォントの最大グリフ数（BMP）
const GLYPH_COUNT: usize = 0x10000;

/// フォント
static FONT: Once<Font> = Once::new();

#[derive(Clone, Copy)]
struct Glyph {
    width: u8,
    height: u8,
    bitmap: [u16; 16],
}

impl Glyph {
    const fn empty() -> Self {
        Self {
            width: 0,
            height: 0,
            bitmap: [0; 16],
        }
    }
}

struct Font {
    width: usize,
    height: usize,
    glyphs: [Glyph; GLYPH_COUNT],
}

impl Font {
    const fn empty() -> Self {
        Self {
            width: 8,
            height: 16,
            glyphs: [Glyph::empty(); GLYPH_COUNT],
        }
    }

    fn glyph(&self, codepoint: u32) -> &Glyph {
        let idx = codepoint as usize;
        if idx < GLYPH_COUNT {
            &self.glyphs[idx]
        } else {
            &self.glyphs[0]
        }
    }
}

fn parse_hex_u16(s: &str) -> u16 {
    let mut value: u16 = 0;
    for b in s.bytes() {
        value = value.saturating_mul(16);
        value = value.saturating_add(match b {
            b'0'..=b'9' => (b - b'0') as u16,
            b'a'..=b'f' => (b - b'a' + 10) as u16,
            b'A'..=b'F' => (b - b'A' + 10) as u16,
            _ => 0,
        });
    }
    value
}

fn parse_font() -> Font {
    let mut font = Font::empty();
    let data = include_bytes!("../unifont_jp-17.0.03.bdf");
    let text = match core::str::from_utf8(data) {
        Ok(t) => t,
        Err(_) => return font,
    };

    let mut in_glyph = false;
    let mut in_bitmap = false;
    let mut encoding: i32 = -1;
    let mut width: usize = 0;
    let mut height: usize = 0;
    let mut row: usize = 0;

    for line in text.lines() {
        if line.starts_with("FONTBOUNDINGBOX ") {
            let mut parts = line.split_whitespace();
            let _ = parts.next();
            if let (Some(w), Some(h)) = (parts.next(), parts.next()) {
                if let (Ok(w), Ok(h)) = (w.parse::<usize>(), h.parse::<usize>()) {
                    if w > 0 && h > 0 {
                        font.width = w.min(16);
                        font.height = h.min(16);
                    }
                }
            }
            continue;
        }

        if line.starts_with("STARTCHAR") {
            in_glyph = true;
            in_bitmap = false;
            encoding = -1;
            width = 0;
            height = 0;
            row = 0;
            continue;
        }

        if line.starts_with("ENDCHAR") {
            if encoding >= 0 && (encoding as usize) < GLYPH_COUNT {
                let glyph = &mut font.glyphs[encoding as usize];
                if width > 0 {
                    glyph.width = width.min(16) as u8;
                }
                if height > 0 {
                    glyph.height = height.min(16) as u8;
                }
            }
            in_glyph = false;
            in_bitmap = false;
            continue;
        }

        if !in_glyph {
            continue;
        }

        if line.starts_with("ENCODING ") {
            let mut parts = line.split_whitespace();
            let _ = parts.next();
            if let Some(enc) = parts.next() {
                if let Ok(v) = enc.parse::<i32>() {
                    encoding = v;
                }
            }
            continue;
        }

        if line.starts_with("BBX ") {
            let mut parts = line.split_whitespace();
            let _ = parts.next();
            if let (Some(w), Some(h)) = (parts.next(), parts.next()) {
                if let (Ok(w), Ok(h)) = (w.parse::<usize>(), h.parse::<usize>()) {
                    width = w;
                    height = h;
                }
            }
            continue;
        }

        if line == "BITMAP" {
            in_bitmap = true;
            row = 0;
            continue;
        }

        if in_bitmap {
            if encoding >= 0 && (encoding as usize) < GLYPH_COUNT && row < 16 {
                let mut value = parse_hex_u16(line);
                let w = width.min(16);
                if w > 0 && w < 16 {
                    value = value << (16 - w);
                }
                font.glyphs[encoding as usize].bitmap[row] = value;
            }
            row += 1;
        }
    }

    font
}

/// フレームバッファライター
pub struct Writer {
    column: usize,
    row: usize,
    max_cols: usize,
    max_rows: usize,
    font_width: usize,
    font_height: usize,
}

impl Writer {
    fn new(info: &FramebufferInfo, font_width: usize, font_height: usize) -> Self {
        let max_cols = info.width / font_width;
        let max_rows = info.height / font_height;
        Self {
            column: 0,
            row: 0,
            max_cols,
            max_rows,
            font_width,
            font_height,
        }
    }

    /// ピクセルを描画
    fn put_pixel(&self, x: usize, y: usize, color: u32) {
        if let Some(info) = FB_INFO.get() {
            // 境界チェック
            if x >= info.width || y >= info.height {
                return;
            }
            let offset = y * info.stride + x;
            let fb_ptr = info.addr as *mut u32;
            unsafe {
                fb_ptr.add(offset).write_volatile(color);
            }
        }
    }

    /// 文字を描画（BDFフォント）
    fn draw_char(&self, codepoint: u32, x: usize, y: usize, fg: u32, bg: u32) {
        let font = match FONT.get() {
            Some(font) => font,
            None => return,
        };

        let glyph = font.glyph(codepoint);
        let glyph_w = if glyph.width == 0 {
            font.width
        } else {
            glyph.width as usize
        };
        let glyph_h = if glyph.height == 0 {
            font.height
        } else {
            glyph.height as usize
        };

        for row in 0..self.font_height {
            for col in 0..self.font_width {
                let is_set = if row < glyph_h && col < glyph_w {
                    let bits = glyph.bitmap[row];
                    let mask = 1u16 << (15 - col);
                    (bits & mask) != 0
                } else {
                    false
                };
                let color = if is_set { fg } else { bg };
                self.put_pixel(x + col, y + row, color);
            }
        }
    }

    /// 1文字書き込み
    pub fn write_char(&mut self, ch: char) {
        if ch == '\n' {
            self.new_line();
            return;
        }

        if self.column >= self.max_cols {
            self.new_line();
        }

        let x = self.column * self.font_width;
        let y = self.row * self.font_height;
        self.draw_char(ch as u32, x, y, 0xFFFFFF, 0x000000); // 白文字、黒背景

        self.column += 1;
    }

    /// 文字列を書き込み
    pub fn write_string(&mut self, s: &str) {
        for ch in s.chars() {
            self.write_char(ch);
        }
    }

    /// 改行処理
    fn new_line(&mut self) {
        self.row += 1;
        self.column = 0;
        if self.row >= self.max_rows {
            // スクロールの代わりに画面クリア（簡易版）
            self.clear_screen();
        }
    }

    /// 画面全体をクリア
    pub fn clear_screen(&mut self) {
        if let Some(info) = FB_INFO.get() {
            let fb_ptr = info.addr as *mut u32;

            let total_pixels = info.height * info.width;
            unsafe {
                for i in 0..total_pixels {
                    fb_ptr.add(i).write_volatile(0x000000); // 黒
                }
            }
        }
        self.row = 0;
        self.column = 0;
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

/// グローバルライター（遅延初期化）
static WRITER: Once<Mutex<Writer>> = Once::new();

/// フレームバッファを初期化
pub fn init(addr: u64, width: usize, height: usize, stride: usize) {
    FB_INFO.call_once(|| FramebufferInfo {
        addr,
        width,
        height,
        stride,
    });

    FONT.call_once(parse_font);

    if let Some(info) = FB_INFO.get() {
        let (font_w, font_h) = FONT
            .get()
            .map(|font| (font.width, font.height))
            .unwrap_or((8, 16));
        WRITER.call_once(|| Mutex::new(Writer::new(info, font_w, font_h)));

        // 画面をクリア
        if let Some(writer) = WRITER.get() {
            writer.lock().clear_screen();
        }
    }
}

/// フレームバッファに文字列を出力（割り込み対応）
pub fn print(args: fmt::Arguments) {
    use core::fmt::Write;
    if let Some(writer) = WRITER.get() {
        // 割り込みを無効化してロック取得（デッドロック防止）
        x86_64::instructions::interrupts::without_interrupts(|| {
            let _ = writer.lock().write_fmt(args);
        });
    }
}

/// フレームバッファ出力マクロ
#[macro_export]
macro_rules! vprint {
    ($($arg:tt)*) => {
        $crate::util::vga::print(format_args!($($arg)*))
    };
}

/// 改行付きフレームバッファ出力マクロ
#[macro_export]
macro_rules! vprintln {
    () => ($crate::vprint!("\n"));
    ($($arg:tt)*) => {
        $crate::vprint!("{}\n", format_args!($($arg)*))
    };
}
