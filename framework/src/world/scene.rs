use std::hash::Hash;
use std::collections::{VecDeque, HashMap};

use vulkano::command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer};

pub use crate::timer::*;
pub use crate::renderer::*;
pub use crate::{err, error::*};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SceneRequest<ID>
where ID: Clone + Copy + PartialEq + Eq + Hash {
    Pop,
    Push { id: ID },
    Change { id: ID },
}

pub struct SceneManager<ID> 
where ID: Clone + Copy + PartialEq + Eq + Hash {
    stack: VecDeque<ID>,
    nodes: HashMap<ID, Box<dyn SceneNode<ID>>>,
}

impl<ID> SceneManager<ID> 
where ID: Clone + Copy + PartialEq + Eq + Hash {
    pub fn new(nodes: Vec<(ID, Box<dyn SceneNode<ID>>)>) -> Self {
        assert!(!nodes.is_empty(), "At least one scene node is required.");
        Self { 
            stack: VecDeque::from([nodes.first().unwrap().0]), 
            nodes: nodes.into_iter().collect(), 
        }
    }

    pub fn pause(&mut self, timer: &Timer, renderer: &Renderer) -> Result<(), RuntimeError> {
        let curr = match self.stack.back() {
            Some(id) => id.clone(),
            None => return Err(err!("Logic Error: There are no scenes currently in use."))
        };
        let curr_node = match self.nodes.get_mut(&curr) {
            Some(node) => node,
            None => return Err(err!("Logic Error: Node not registered in scene manager."))
        };

        curr_node.pause(timer, renderer)
    }

    pub fn resume(&mut self, timer: &Timer, renderer: &Renderer) -> Result<(), RuntimeError> {
        let curr = match self.stack.back() {
            Some(id) => id.clone(),
            None => return Err(err!("Logic Error: There are no scenes currently in use."))
        };
        let curr_node = match self.nodes.get_mut(&curr) {
            Some(node) => node,
            None => return Err(err!("Logic Error: Node not registered in scene manager."))
        };

        curr_node.resume(timer, renderer)
    }

    pub fn frame_advanced(&mut self, timer: &mut Timer, renderer: &mut Renderer) -> Result<(), RuntimeError> {
        let curr = match self.stack.back() {
            Some(id) => id.clone(),
            None => return Err(err!("Logic Error: There are no scenes currently in use."))
        };
        let mut curr_node = match self.nodes.get_mut(&curr) {
            Some(node) => node,
            None => return Err(err!("Logic Error: Node not registered in scene manager."))
        };
        
        if let Some(request) = curr_node.get_request() {
            curr_node = match request {
                SceneRequest::Pop => {
                    curr_node.exit(renderer)?;
                    self.stack.pop_back().unwrap();
                    let prev = match self.stack.back() {
                        Some(id) => id.clone(),
                        None => return Err(err!("Logic Error: There are no scenes currently in use."))
                    };
                    let prev_node = match self.nodes.get_mut(&prev) {
                        Some(node) => node,
                        None => return Err(err!("Logic Error: Node not registered in scene manager."))
                    };
                    prev_node
                },
                SceneRequest::Push { id } => {
                    self.stack.push_back(id);
                    let next_node = match self.nodes.get_mut(&id) {
                        Some(node) => node,
                        None => return Err(err!("Logic Error: Node not registered in scene manager."))
                    };
                    next_node.enter(renderer)?;
                    next_node
                },  
                SceneRequest::Change { id } => {
                    curr_node.exit(renderer)?;
                    self.stack.pop_back().unwrap();

                    self.stack.push_back(id);
                    let change_node = match self.nodes.get_mut(&id) {
                        Some(node) => node,
                        None => return Err(err!("Logic Error: Node not registered in scene manager."))
                    };
                    change_node.enter(renderer)?;
                    change_node
                }
            }
        }

        curr_node.update(timer, &renderer)?;
        let acquire_future = renderer.wait_for_next_frame()?;
        if let Some(acquire_future) = acquire_future {
            let command_buffer_builder = renderer.primary_command_buffer(None)?;
            let mut builder = renderer.begin_render_pass(
                vec![
                    Some(ClearValue::Float(rgba(255, 255, 255, 255))),
                    Some(ClearValue::DepthStencil((1.0, 0))),
                ], 
                None, 
                None, 
                SubpassContents::SecondaryCommandBuffers, 
                command_buffer_builder
            )?;
            curr_node.draw(renderer, &mut builder)?;

            let command_buffer = builder
                .end_render_pass()
                .map_err(|e| err!("Vk Command Buffer Recoding Error: {}", e.to_string()))?
                .build()
                .map_err(|e| err!("Vk Command Buffer Building Error: {}", e.to_string()))?;
            renderer.queue_submit_and_present(acquire_future, command_buffer)?;
        }
        
        Ok(())
    }
}

pub trait SceneNode<ID> 
where ID: Clone + Copy + PartialEq + Eq + Hash {
    fn get_request(&self) -> Option<SceneRequest<ID>> { None }

    fn enter(&mut self, renderer: &Renderer) -> Result<(), RuntimeError> { Ok(()) }
    fn exit(&mut self, renderer: &Renderer) -> Result<(), RuntimeError> { Ok(()) }

    fn pause(&mut self, timer: &Timer, renderer: &Renderer) -> Result<(), RuntimeError> { Ok(()) }
    fn resume(&mut self, timer: &Timer, renderer: &Renderer) -> Result<(), RuntimeError> { Ok(()) }

    fn update(&mut self, timer: &Timer, renderer: &Renderer) -> Result<(), RuntimeError> { Ok(()) }
    fn draw(&mut self, renderer: &Renderer, builder: &mut CmdBufBeginRenderPassGuard) -> Result<(), RuntimeError> { Ok(()) }
}
