use crate::Rect;
use crate::game::GameContext;
use crate::grid_layout::GridLayout;
use crate::renderer::Renderer;
use crate::renderer::{Color, RenderCommand};
use crate::scene::button::Button;
use std::cell::RefCell;
use std::rc::Rc;

pub struct FakeRenderer {
    pub actions: Rc<RefCell<Vec<RenderCommand>>>,
}

impl FakeRenderer {
    pub fn new() -> (Self, Rc<RefCell<Vec<RenderCommand>>>) {
        let actions = Rc::new(RefCell::new(vec![]));
        let r = FakeRenderer {
            actions: actions.clone(),
        };
        (r, actions)
    }
}

impl Renderer for FakeRenderer {
    fn name(&self) -> String {
        "Fake!".to_string()
    }
    fn send_command(&mut self, cmd: RenderCommand) {
        eprintln!("COMMAND ADDED {:?}", cmd);
        self.actions.as_ref().borrow_mut().push(cmd);
    }
    fn clear(&mut self, _color: Color) {}
    fn present(&mut self) {}
}

pub fn center_of_rect_screen_coordinates(
    rect: &Rect,
    game_context: &GameContext,
    layout: &GridLayout,
) -> Option<(f32, f32)> {
    let screen_cell = layout.cell_rect(rect.y as usize, rect.x as usize);
    let x = screen_cell.x as f32 + screen_cell.width as f32 / 2.0;
    let y = screen_cell.y as f32 + screen_cell.height as f32 / 2.0;
    (x, y).into()
}

pub fn click_button_with_left_mouse(
    btn: &Button,
    game_context: &mut GameContext,
    layout: &GridLayout,
) {
    let position = center_of_rect_screen_coordinates(&btn.rect, &game_context, &layout);
    let left_mouse_btn = true;
    let right_mouse_btn = false;
    game_context
        .mouse_context
        .update(left_mouse_btn, right_mouse_btn, position);
}
