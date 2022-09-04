use crate::Rect;
use crate::graphics::Texture;

#[derive(Debug, Clone)]
pub struct Font {
    texture: Texture,
    src: String
}

impl Font {
    pub fn new(texture: Texture, src: String) -> Self {
        Self {
            texture,
            src
        }
    }
}

// Sketchy text renderer
pub fn render_text(text: &str, font: &Font, r: &Rect<f32>, scale: f32) -> Vec<(Rect<f32>, Texture)> {
    let mut chars = vec![];

    let tex_char_width = font.texture.width() / font.src.len() as u32;
    let char_width = (tex_char_width as f32 * scale) as u32;
    let tex_char_height = font.texture.height();
    let char_height = (font.texture.height() as f32 * scale) as u32;

    let line_width = r.w as u32 / char_width;
    let mut line_num = 0;
    let mut row_num = 0;

    for word in text.split(" ") {
        if word.len() as u32 > line_width {
            panic!("Word too long: {word}");
        } else if word.len() as u32 > line_width-row_num {
            line_num += 1;
            row_num = 0;
        }

        for c in word.chars() {
            if let Some(index) = font.src.find(c) {
                let rect = Rect::new(
                    r.x + (char_width * row_num) as f32,
                    r.y + (char_height * line_num) as f32,
                    char_width as f32,
                    char_height as f32
                );

                let tex = Texture::new(
                    font.texture.x() + tex_char_width * index as u32,
                    font.texture.y(),
                    tex_char_width,
                    tex_char_height
                );

                chars.push((rect, tex));

                row_num += 1;
            } else {
                panic!("Unsupported character: {c}")
            }
        }

        if row_num == line_width {
            line_num += 1;
            row_num = 0;
        } else {
            // Insert space character
            row_num += 1;
        }
    }

    chars
}
