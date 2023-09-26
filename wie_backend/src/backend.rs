pub mod canvas;
mod resource;
pub mod time;
pub mod window;

use alloc::{collections::VecDeque, rc::Rc};
use core::cell::{Ref, RefCell, RefMut};

use wie_base::Event;

use self::{resource::Resource, time::Time};

use self::{
    canvas::{ArgbPixel, Canvas, Image, ImageBuffer},
    window::Window,
};

pub struct Backend {
    resource: Rc<RefCell<Resource>>,
    time: Rc<RefCell<Time>>,
    screen_canvas: Rc<RefCell<Box<dyn Canvas>>>,
    events: Rc<RefCell<VecDeque<Event>>>,
    window: Rc<RefCell<Window>>,
}

impl Default for Backend {
    fn default() -> Self {
        Self::new()
    }
}

impl Backend {
    pub fn new() -> Self {
        let canvas = ImageBuffer::<ArgbPixel>::new(240, 320); // TODO hardcoded size
        let window = Window::new(canvas.width(), canvas.height());

        Self {
            resource: Rc::new(RefCell::new(Resource::new())),
            time: Rc::new(RefCell::new(Time::new())),
            screen_canvas: Rc::new(RefCell::new(Box::new(canvas))),
            events: Rc::new(RefCell::new(VecDeque::new())),
            window: Rc::new(RefCell::new(window)),
        }
    }

    pub fn resource(&self) -> Ref<'_, Resource> {
        (*self.resource).borrow()
    }

    pub fn time(&self) -> Ref<'_, Time> {
        (*self.time).borrow()
    }

    pub fn screen_canvas(&self) -> RefMut<'_, Box<dyn Canvas>> {
        (*self.screen_canvas).borrow_mut()
    }

    pub fn window(&self) -> RefMut<'_, Window> {
        (*self.window).borrow_mut()
    }

    pub fn pop_event(&self) -> Option<Event> {
        (*self.events).borrow_mut().pop_front()
    }

    pub fn push_event(&self, event: Event) {
        (*self.events).borrow_mut().push_back(event);
    }

    pub fn repaint(&self) {
        self.window().paint(&**self.screen_canvas());
    }

    pub fn add_resources_from_zip(&self, zip: &[u8]) -> anyhow::Result<()> {
        (*self.resource).borrow_mut().add_from_zip(zip)
    }
}

impl Clone for Backend {
    fn clone(&self) -> Self {
        Self {
            resource: self.resource.clone(),
            time: self.time.clone(),
            screen_canvas: self.screen_canvas.clone(),
            events: self.events.clone(),
            window: self.window.clone(),
        }
    }
}
