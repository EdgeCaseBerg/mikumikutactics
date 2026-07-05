use crate::Rect;
use crate::grid_layout::GridLayout;

pub trait FontMetrics {
    fn line_height(&self) -> i32;
    fn measure_width(&self, s: &str) -> i32;
}

pub struct BitmapFontMetrics {
    pub layout: GridLayout,
}

impl FontMetrics for BitmapFontMetrics {
    fn line_height(&self) -> i32 {
        let (_, h) = self.layout.cell_size();
        h as i32
    }

    fn measure_width(&self, s: &str) -> i32 {
        let (w, _) = self.layout.cell_size();
        (w * s.len() as isize) as i32
        // get_rects_for_str(s)
        //     .iter()
        //     .fold(0, |acc, cell| acc + cell.width as i32)
    }
}

pub const FONTSHEET_LAYOUT: GridLayout = {
    GridLayout {
        area: Rect {
            x: 0,
            y: 0,
            width: 145,
            height: 1412,
        },
        rows: 83,
        columns: 16,
        cell_gap: 1,
    }
};

pub fn get_rects_for_str(str: &str) -> Vec<Rect> {
    // font sheet starts at space, so -32 from the character's ascii value
    // to get the index. Then we need to convert that index into
    // This is horrifically inefficient, but for testing it should be okay:
    let cells: Vec<Rect> = FONTSHEET_LAYOUT.iter_cells().map(|(_, _, r)| r).collect();
    str.chars()
        .map(|c| {
            let ascii = c as u32;
            if ascii > 127 {
                cells[0].clone()
            } else {
                cells[ascii as usize - 32].clone()
            }
        })
        .collect()
}

pub fn center_font_in_tile(anchor: Rect, font_cell: Rect, str_index: isize) -> Rect {
    let (cx, cy) = anchor.center();
    Rect {
        x: cx + str_index * font_cell.width,
        y: cy - font_cell.height / 2,
        width: font_cell.width,
        height: font_cell.height,
    }
}
