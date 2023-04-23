use std::fmt;
use std::hash::Hash;
use std::collections::{VecDeque, HashMap};

use crate::timer::*;
use crate::renderer::*;
use crate::{err, error::RuntimeError};



/// Used when moving or changing from one scene to another
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneRequest<SceneID = String>
where SceneID: fmt::Debug + Clone + Eq + Hash {
    Pop,
    Push { id: SceneID },
    Change { id: SceneID },
}



/// A manager that manages all registered scenes.
/// scene ID must not be duplicated.
#[derive(Debug)]
pub struct SceneManager<SceneID = String> 
where SceneID: fmt::Debug + Clone + Eq + Hash {
    stack: VecDeque<SceneID>,
    nodes: HashMap<SceneID, Box<dyn SceneNode<SceneID>>>,
}

impl<SceneID> SceneManager<SceneID> 
where SceneID: fmt::Debug + Clone + Eq + Hash {
    /// Create a new scene manager.
    /// 
    /// # Runtime Error
    /// Return the `RuntimeError` if entry to the starting scene node fails.
    /// 
    /// # Panics
    /// Stop program execution if the starting scene node is not registered.
    /// 
    pub fn new<I>(
        nodes: I,
        entry_point: SceneID,
        renderer: &Renderer,
    ) -> Result<Self, RuntimeError>
    where I: IntoIterator<Item = (SceneID, Box<dyn SceneNode<SceneID>>)>, I::IntoIter: ExactSizeIterator {
        let mut nodes: HashMap<_, _> = nodes.into_iter().collect();
        let node = nodes.get_mut(&entry_point)
            .expect("Logic Error: The scene node's entry point is not registered.");

        node.enter(renderer)?;

        Ok(Self { stack: VecDeque::from([entry_point]), nodes, })
    }

    /// Return the ID of the current scene node.
    /// 
    /// # Panics
    /// Stop program execution if there is no current node.
    /// 
    #[inline]
    fn get_current_id(&self) -> SceneID {
        self.stack.back()
            .expect("Logic Error: There are no scenes currently in use.")
            .clone()
    }

    /// Borrow the scene node.
    /// 
    /// # Panics
    /// Stop program execution if scene node is not registered in scene manager.
    /// 
    #[inline]
    fn mut_scene_node(&mut self, id: &SceneID) -> &mut Box<dyn SceneNode<SceneID>> {
        self.nodes.get_mut(id)
            .expect("Logic Error: Node not registered in scene manager.")
    }

    /// Pause the current scene.
    /// 
    /// # Runtime Error
    /// Return the `RuntimeError` if the error occurs during pause.
    /// 
    /// # Panics
    /// - Stop program execution if there is no current node.
    /// - Stop program execution if scene node is not registered in scene manager.
    /// 
    pub fn pause(&mut self, timer: &Timer, renderer: &Renderer) -> Result<(), RuntimeError> {
        self.mut_scene_node(&self.get_current_id()).pause(timer, renderer)
    }

    /// Resume the current scene.
    /// 
    /// # Runtime Error
    /// Return the `RuntimeError` if the error occurs during resume.
    /// 
    /// # Panics
    /// - Stop program execution if there is no current node.
    /// - Stop program execution if scene node is not registered in scene manager.
    /// 
    pub fn resume(&mut self, timer: &Timer, renderer: &Renderer) -> Result<(), RuntimeError> {
        self.mut_scene_node(&self.get_current_id()).resume(timer, renderer)
    }

    /// Prepares the next frame of the scene and draws it to the screen.
    /// 
    /// # Runtime Error
    /// Return the `RuntimeError` if the error occurs while updating and drawing.
    /// 
    /// # Panics
    /// - Stop program execution if there is no current node.
    /// - Stop program execution if scene node is not registered in scene manager.
    /// 
    pub fn frame_advanced(&mut self, timer: &mut Timer, renderer: &mut Renderer) -> Result<(), RuntimeError> {
        let mut curr_node = self.mut_scene_node(&self.get_current_id());
        if let Some(request) = curr_node.get_request() {
            curr_node = match request {
                SceneRequest::Pop => {
                    curr_node.exit(renderer)?;
                    self.stack.pop_back().unwrap();
                    self.mut_scene_node(&self.get_current_id())
                },
                SceneRequest::Push { id } => {
                    self.stack.push_back(id.clone());
                    let next_node = self.mut_scene_node(&id);
                    next_node.enter(renderer)?;
                    next_node
                },  
                SceneRequest::Change { id } => {
                    curr_node.exit(renderer)?;
                    self.stack.pop_back().unwrap();

                    self.stack.push_back(id.clone());
                    let change_node = self.mut_scene_node(&id);
                    change_node.enter(renderer)?;
                    change_node
                }
            }
        }

        curr_node.update(timer, renderer)?;
        curr_node.draw(renderer)?;
        
        Ok(())
    }
}



/// The scene node's interface.
pub trait SceneNode<SceneID = String> : fmt::Debug
where SceneID: fmt::Debug + Clone + Eq + Hash {
    /// Returns the scene node's request. Default is `None` .
    fn get_request(&self) -> Option<SceneRequest<SceneID>> { None }

    /// This function is called when entering the scene node.
    /// 
    /// # Runtime Error
    /// Return the `RuntimeError` if an error occurs while entering the scene node.
    /// 
    fn enter(&mut self, renderer: &Renderer) -> Result<(), RuntimeError> { Ok(()) }

    /// This function is called when exiting the scene node.
    /// 
    /// # Runtime Error
    /// Return the `RuntimeError` if an error occurs while exiting the scene node.
    /// 
    fn exit(&mut self, renderer: &Renderer) -> Result<(), RuntimeError> { Ok(()) }

    /// This function is called when pausing the scene node.
    /// 
    /// # Runtime Error
    /// Return the `RuntimeError` if an error occurs while pausing the scene node.
    /// 
    fn pause(&mut self, timer: &Timer, renderer: &Renderer) -> Result<(), RuntimeError> { Ok(()) }

    /// This function is called when resuming the scene node.
    /// 
    /// # Runtime Error
    /// Return the `RuntimeError` if an error occurs while resuming the scene node.
    /// 
    fn resume(&mut self, timer: &Timer, renderer: &Renderer) -> Result<(), RuntimeError> { Ok(()) }

    /// This function is called when updating a scene node.
    /// 
    /// # Runtime Error
    /// Return the `RuntimeError` if an error occurs while updating the scene node.
    /// 
    fn update(&mut self, timer: &Timer, renderer: &Renderer) -> Result<(), RuntimeError> { Ok(()) }

    /// This function is called when drawing a scene node.
    /// 
    /// # Runtime Error
    /// Return the `RuntimeError` if an error occurs while drawing the scene node.
    /// 
    fn draw(&mut self, renderer: &mut Renderer) -> Result<(), RuntimeError> { Ok(()) }
}
