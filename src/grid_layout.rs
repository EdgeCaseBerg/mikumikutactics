use crate::Rect;

pub struct GridLayout {
    pub area: Rect,
    pub rows: usize,
    pub columns: usize,
    pub cell_gap: isize,
}

impl GridLayout {
    pub fn cell_size(&self) -> (isize, isize) {
        let total = (self.area.width, self.area.height);

        // [gap [cell] gap [cell] gap ]
        let gaps = (
            (self.columns as isize + 1) * self.cell_gap,
            (self.rows as isize + 1) * self.cell_gap,
        );

        // (max(vec)) to avoid negative space
        let space_for_cells = ((total.0 - gaps.0).max(0), (total.1 - gaps.1).max(0));

        (
            space_for_cells.0 / self.columns as isize,
            space_for_cells.1 / self.rows as isize,
        )
    }

    // Return the top left of where the cells start within the grid layout (offset by the gap)
    pub fn origin(&self) -> (isize, isize) {
        (self.area.x + self.cell_gap, self.area.y + self.cell_gap)
    }

    pub fn cell_for_mouse(
        &self,
        mouse_position: Option<(f32, f32)>,
    ) -> Option<(usize, usize, Rect)> {
        let (x, y) = mouse_position?;
        self.screen_to_cell(x, y)
    }

    pub fn screen_to_cell(&self, wx: f32, wy: f32) -> Option<(usize, usize, Rect)> {
        let (origin_x, origin_y) = self.origin();
        let (cell_width, cell_height) = self.cell_size();
        let step_x = cell_width + self.cell_gap;
        let step_y = cell_height + self.cell_gap;

        let relative_x = wx as f64 - origin_x as f64;
        let relative_y = wy as f64 - origin_y as f64;

        let c = (relative_x / step_x as f64).floor() as isize;
        let r = (relative_y / step_y as f64).floor() as isize;

        if r < 0 || c < 0 {
            return None;
        }
        let r = r as usize;
        let c = c as usize;
        if r >= self.rows || c >= self.columns {
            return None;
        }

        Some((r, c, self.cell_rect(r, c)))
    }

    pub fn cell_rect(&self, r: usize, c: usize) -> Rect {
        let cell = self.cell_size();

        let origin = self.origin();

        let offset = (
            c as isize * (cell.0 + self.cell_gap),
            r as isize * (cell.1 + self.cell_gap),
        );

        Rect {
            x: origin.0 + offset.0,
            y: origin.1 + offset.1,
            width: cell.0,
            height: cell.1,
        }
    }

    // For me in 3 months: '_ is an anonymous lifetime tied to self.
    // it just means the caller can't let the iterator last longer than this layout
    pub fn iter_cells(&self) -> impl Iterator<Item = (usize, usize, Rect)> + '_ {
        let cell_size = self.cell_size();
        let origin = self.origin();
        let step = (cell_size.0 + self.cell_gap, cell_size.1 + self.cell_gap);

        (0..self.rows).flat_map(move |r| {
            (0..self.columns).map(move |c| {
                let top_left_x = origin.0 + c as isize * step.0;
                let top_left_y = origin.1 + r as isize * step.1;
                let cell = Rect {
                    x: top_left_x,
                    y: top_left_y,
                    width: cell_size.0,
                    height: cell_size.1,
                };
                (r, c, cell)
            })
        })
    }
}
