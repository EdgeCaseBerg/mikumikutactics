use crate::Rect;
use crate::constants::TextureId;

// Colors are defined in sRGB within 0.0 - 1.0
// While SDL provides one of these, for future proofing against playing with other renderers, define our own.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl From<Color> for (u8, u8, u8, u8) {
    fn from(color: Color) -> (u8, u8, u8, u8) {
        let r = (color.r * 255.0) as u8;
        let g = (color.g * 255.0) as u8;
        let b = (color.b * 255.0) as u8;
        let a = (color.a * 255.0) as u8;
        (r, g, b, a)
    }
}

impl Color {
    pub const fn black() -> Color {
        Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    }
    pub const fn white() -> Color {
        Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }
    }
}

// Generic trait that we'll make an SDL version of for fun.
// and in order to be able to write code in a generic way while
// swapping out graphics library for fun as needed.
pub trait Renderer {
    fn name(&self) -> String;
    fn send_command(&mut self, cmd: RenderCommand);
    fn clear(&mut self, color: Color);
    fn present(&mut self);
}

#[derive(Debug)]
pub enum RenderCommand {
    DrawRect {
        texture_id: TextureId,
        source: Rect,
        destination: Rect,
    },
    FillRect {
        color: Color,
        destination: Rect,
    },
}
