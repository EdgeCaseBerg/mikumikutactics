use crate::renderer::Renderer;
use crate::renderer::{Color, RenderCommand};
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
