pub mod canvas;
mod resource;
pub mod time;
pub mod window;

use alloc::{collections::VecDeque, rc::Rc};
use core::cell::{Ref, RefCell, RefMut};

use wie_base::Event;

use self::{canvas::Canvas, resource::Resource, time::Time, window::WindowProxy};

pub struct Backend {
    resource: Rc<RefCell<Resource>>,
    time: Rc<RefCell<Time>>,
    screen_canvas: Rc<RefCell<Box<dyn Canvas>>>,
    events: Rc<RefCell<VecDeque<Event>>>,
    window: Rc<RefCell<WindowProxy>>,
}

impl Backend {
    pub fn new(screen_canvas: Box<dyn Canvas>, window_proxy: WindowProxy) -> Self {
        Self {
            resource: Rc::new(RefCell::new(Resource::new())),
            time: Rc::new(RefCell::new(Time::new())),
            screen_canvas: Rc::new(RefCell::new(screen_canvas)),
            events: Rc::new(RefCell::new(VecDeque::new())),
            window: Rc::new(RefCell::new(window_proxy)),
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

    pub fn window(&self) -> RefMut<'_, WindowProxy> {
        (*self.window).borrow_mut()
    }

    pub fn pop_event(&self) -> Option<Event> {
        (*self.events).borrow_mut().pop_front()
    }

    pub fn push_event(&self, event: Event) {
        (*self.events).borrow_mut().push_back(event);
    }

    pub fn repaint(&self) -> anyhow::Result<()> {
        self.window().repaint(&**self.screen_canvas())
    }

    pub fn add_resource(&self, path: &str, data: Vec<u8>) {
        (*self.resource).borrow_mut().add(path, data);
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
