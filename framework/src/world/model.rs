use std::fmt;
use std::hash::Hash;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use vulkano::command_buffer::AutoCommandBufferBuilder;

use crate::math::*;
use crate::renderer::*;
use crate::world::mesh::Mesh;
use crate::world::shader::GraphicsShader;
use crate::{err, error::RuntimeError};


/// The data types of the nodes that make up the model.
#[derive(Clone)]
pub struct ModelNode<NodeID = String> 
where NodeID: fmt::Debug + Clone + Eq + Hash {
    pub id: NodeID,
    pub transform: Mat4x4,
    pub world_matrix: Mat4x4,
    pub mesh: Option<Arc<Mesh>>,
    pub shader: Option<Arc<GraphicsShader>>,
    pub parent: Option<NodeID>,
    pub sibling: Option<NodeID>,
    pub child: Option<NodeID>
}

impl<NodeID> ModelNode<NodeID>
where NodeID: fmt::Debug + Clone + Eq + Hash {
    /// Returns the relative position of a node.
    #[inline]
    fn get_position(&self) -> Vec3 {
        Vec3::new_vector(
            self.transform.r4c1, 
            self.transform.r4c2, 
            self.transform.r4c3
        )
    }

    /// Returns the relative rotation of a node.
    #[inline]
    fn get_quaternion(&self) -> Quat {
        self.transform.into_quat()
    }

    /// Returns the relative right vector of a node.
    #[inline]
    fn get_right_vector(&self) -> Vec3 {
        Vec3::new_vector(
            self.transform.r1c1, 
            self.transform.r1c2, 
            self.transform.r1c3
        )
    }

    /// Returns the relative up vector of a node.
    #[inline]
    fn get_up_vector(&self) -> Vec3 {
        Vec3::new_vector(
            self.transform.r2c1, 
            self.transform.r2c2, 
            self.transform.r2c3
        )
    }
    
    /// Returns the relative look vector of a node.
    #[inline]
    fn get_look_vector(&self) -> Vec3 {
        Vec3::new_vector(
            self.transform.r3c1, 
            self.transform.r3c2, 
            self.transform.r3c3
        )
    }

    /// Sets the relative position of a node.
    #[inline]
    fn set_position(&mut self, position: Vec3) {
        self.transform.r4c1 = position.x;
        self.transform.r4c2 = position.y;
        self.transform.r4c3 = position.z;
    }

    /// Sets the relative rotation of a node.
    #[inline]
    fn set_quaternion(&mut self, quaternion: Quat) {
        let m = quaternion.normalize().into_matrix3x3();

        self.transform.r1c1 = m.r1c1;
        self.transform.r1c2 = m.r1c2;
        self.transform.r1c3 = m.r1c3;

        self.transform.r2c1 = m.r2c1;
        self.transform.r2c2 = m.r2c2;
        self.transform.r2c3 = m.r2c3;

        self.transform.r3c1 = m.r3c1;
        self.transform.r3c2 = m.r3c2;
        self.transform.r3c3 = m.r3c3;
    }

    /// Sets the releative rotation of a node.
    #[inline]
    fn set_look_at_point(&mut self, point: Vec3) {
        let up = self.get_up_vector().normalize();
        let look = (point - self.get_position()).normalize();
        let right = up.cross(&look).normalize();
        let up = look.cross(&right).normalize();

        self.transform.r1c1 = right.x;
        self.transform.r1c2 = right.y;
        self.transform.r1c3 = right.z;

        self.transform.r2c1 = up.x;
        self.transform.r2c2 = up.y;
        self.transform.r2c3 = up.z;

        self.transform.r3c1 = look.x;
        self.transform.r3c2 = look.y;
        self.transform.r3c3 = look.z;
    }

    /// Moves the position of a node relative to the node's coordinate system.
    #[inline]
    fn translate_local(&mut self, distance: Vec3) {
        let x = self.get_right_vector().normalize() * distance.x;
        let y = self.get_up_vector().normalize() * distance.y;
        let z = self.get_look_vector().normalize() * distance.z;
        self.translate_world(x + y + z);
    }

    /// Moves the position of a node relative to the world's coordinate system.
    #[inline]
    fn translate_world(&mut self, distance: Vec3) {
        self.transform.r4c1 += distance.x;
        self.transform.r4c2 += distance.y;
        self.transform.r4c3 += distance.z;
    }

    /// Rotates the orientation of a node by an angle with a given axis.
    #[inline]
    fn rotate_from_angle_axis(&mut self, angle_radian: f32, axis: Vec3) {
        self.rotate_from_quaternion(Quat::from_angle_axis(angle_radian, axis.normalize()));
    }

    /// Rotates the orientation of a node by a given quaternion.
    #[inline]
    fn rotate_from_quaternion(&mut self, quaternion: Quat) {
        let rotation_matrix = quaternion.normalize().into_matrix4x4();
        self.transform = rotation_matrix * self.transform;
    }

    /// Update the transform of nodes.
    #[inline]
    fn update_transform(&mut self, parent_matrix: Option<Mat4x4>) -> (Mat4x4, Option<NodeID>, Option<NodeID>) {
        if let Some(parent_matrix) = parent_matrix {
            self.world_matrix = self.transform * parent_matrix;
        }
        return (
            self.world_matrix, 
            self.sibling.clone(), 
            self.child.clone()
        )
    }
}



/// A model data type consisting of a set of nodes.
pub struct Model<NodeID = String>
where NodeID: fmt::Debug + Clone + Eq + Hash {
    name: String,
    root_id: NodeID,
    nodes: Vec<ModelNode<NodeID>>,
    id_index_map: HashMap<NodeID, usize>,
}

impl<NodeID> Model<NodeID> 
where NodeID: fmt::Debug + Clone + Eq + Hash {
    pub fn from_nodes<I>(
        name: &str, 
        root_id: NodeID,
        nodes: I,
    ) -> Result<Self, RuntimeError> 
    where 
        I: IntoIterator<Item = ModelNode<NodeID>>,
        I::IntoIter: ExactSizeIterator,
    {
        let name = name.to_string();
        let nodes: Vec<_> = nodes.into_iter().collect();
        let id_index_map: HashMap<_, _> = nodes
            .iter()
            .enumerate()
            .map(|(idx, node)| {
                (node.id.clone(), idx)
            })
            .collect();

        if id_index_map.get(&root_id).is_none() {
            Err(err!("Invalid root ID."))
        }
        else {
            Ok(Self { name, root_id, nodes, id_index_map })
        }
    }

    /// Get the node's index with the given node's ID.
    /// 
    /// # Panics
    /// Stop program execution if the ID of the given node does not belong to the set of nodes in the model.
    /// 
    #[inline]
    fn get_index(&self, id: &NodeID) -> usize {
        match self.id_index_map.get(id) {
            Some(&index) => index,
            None => panic!("Logic Error: Node not registered in Model. (model name: {})", self.name),
        }
    }

    /// Borrow a model node with the given index.
    /// 
    /// # Panics
    /// Stop program execution if there is no node corresponding to the given index.
    /// 
    #[inline]
    fn ref_node(&self, index: usize) -> &ModelNode<NodeID> {
        match self.nodes.get(index) {
            Some(node) => node,
            None => panic!("Logic Error: ModelNode out of range. (model name: {})", self.name),
        }
    }

    /// Borrow a model node with the given index. (mutable)
    /// 
    /// # Panics
    /// Stop program execution if there is no node corresponding to the given index.
    /// 
    #[inline]
    fn mut_node(&mut self, index: usize) -> &mut ModelNode<NodeID> {
        match self.nodes.get_mut(index) {
            Some(node) => node,
            None => panic!("Logic Error: ModelNode out of range. (model name: {})", self.name),
        }
    } 

    /// Returns the relative position of a node with the given ID.
    /// 
    /// # Panics
    /// - Stop program execution if the ID of the given node does not belong to the set of nodes in the model.
    /// - Stop program execution if there is no node corresponding to the given index.
    /// 
    pub fn get_position(&self, id: &NodeID) -> Vec3 {
        self.ref_node(self.get_index(id)).get_position()
    }

    /// Returns the relative rotation of a node with the given ID.
    /// 
    /// # Panics
    /// - Stop program execution if the ID of the given node does not belong to the set of nodes in the model.
    /// - Stop program execution if there is no node corresponding to the given index.
    /// 
    pub fn get_quaternion(&self, id: &NodeID) -> Quat {
        self.ref_node(self.get_index(id)).get_quaternion()
    }

    /// Returns the relative right vector of a node with the given ID.
    /// 
    /// # Panics
    /// - Stop program execution if the ID of the given node does not belong to the set of nodes in the model.
    /// - Stop program execution if there is no node corresponding to the given index.
    /// 
    pub fn get_local_right_vector(&self, id: &NodeID) -> Vec3 {
        self.ref_node(self.get_index(id)).get_right_vector()
    }

    /// Returns the relative up vector of a node with the given ID.
    /// 
    /// # Panics
    /// - Stop program execution if the ID of the given node does not belong to the set of nodes in the model.
    /// - Stop program execution if there is no node corresponding to the given index.
    /// 
    pub fn get_local_up_vector(&self, id: &NodeID) -> Vec3 {
        self.ref_node(self.get_index(id)).get_up_vector()
    }

    /// Returns the relative look vector of a node with the given ID.
    /// 
    /// # Panics
    /// - Stop program execution if the ID of the given node does not belong to the set of nodes in the model.
    /// - Stop program execution if there is no node corresponding to the given index.
    /// 
    pub fn get_local_look_vector(&self, id: &NodeID) -> Vec3 {
        self.ref_node(self.get_index(id)).get_look_vector()
    }

    /// Sets the relative position of a node with the given ID.
    /// 
    /// # Panics
    /// - Stop program execution if the ID of the given node does not belong to the set of nodes in the model.
    /// - Stop program execution if there is no node corresponding to the given index.
    /// 
    pub fn set_position(&mut self, id: &NodeID, position: Vec3) {
        self.mut_node(self.get_index(id)).set_position(position);
        self.update_transform(id, None);
    }

    /// Sets the relative rotation of a node with the given ID.
    /// 
    /// # Panics
    /// - Stop program execution if the ID of the given node does not belong to the set of nodes in the model.
    /// - Stop program execution if there is no node corresponding to the given index.
    /// 
    pub fn set_quaternion(&mut self, id: &NodeID, quaternion: Quat) {
        self.mut_node(self.get_index(id)).set_quaternion(quaternion);
        self.update_transform(id, None);
    }

    /// Sets the relative rotation of a node with the given ID.
    /// 
    /// # Panics
    /// - Stop program execution if the ID of the given node does not belong to the set of nodes in the model.
    /// - Stop program execution if there is no node corresponding to the given index.
    /// 
    pub fn set_look_at_point(&mut self, id: &NodeID, point: Vec3) {
        self.mut_node(self.get_index(id)).set_look_at_point(point);
        self.update_transform(id, None);
    }

    /// Moves the position of a node relative to the node's coordinate system.
    /// 
    /// # Panics
    /// - Stop program execution if the ID of the given node does not belong to the set of nodes in the model.
    /// - Stop program execution if there is no node corresponding to the given index.
    /// 
    pub fn translate_local(&mut self, id: &NodeID, distance: Vec3) {
        self.mut_node(self.get_index(id)).translate_local(distance);
        self.update_transform(id, None);
    }

    /// Moves the position of a node relative to the world's coordinate system.
    /// 
    /// # Panics
    /// - Stop program execution if the ID of the given node does not belong to the set of nodes in the model.
    /// - Stop program execution if there is no node corresponding to the given index.
    /// 
    pub fn translate_world(&mut self, id: &NodeID, distance: Vec3) {
        self.mut_node(self.get_index(id)).translate_world(distance);
        self.update_transform(id, None);
    }

    /// Rotates the orientation of a node by an angle with a given axis.
    ///  
    /// # Panics
    /// - Stop program execution if the ID of the given node does not belong to the set of nodes in the model.
    /// - Stop program execution if there is no node corresponding to the given index.
    /// 
    pub fn rotate_from_angle_axis(&mut self, id: &NodeID, angle_radian: f32, axis: Vec3) {
        self.mut_node(self.get_index(id)).rotate_from_angle_axis(angle_radian, axis);
        self.update_transform(id, None);
    }

    /// Rotates the orientation of a node by a given quaternion.
    /// 
    /// # Panics
    /// - Stop program execution if the ID of the given node does not belong to the set of nodes in the model.
    /// - Stop program execution if there is no node corresponding to the given index.
    /// 
    pub fn rotate_from_quaternion(&mut self, id: &NodeID, quaternion: Quat) {
        self.mut_node(self.get_index(id)).rotate_from_quaternion(quaternion);
        self.update_transform(id, None);
    }

    /// Update the transform of nodes.
    /// 
    /// # Panics
    /// - Stop program execution if the ID of the given node does not belong to the set of nodes in the model.
    /// - Stop program execution if there is no node corresponding to the given index.
    /// 
    pub fn update_transform(&mut self, id: &NodeID, parent_matrix: Option<Mat4x4>) {
        let (world_matrix, sibling, child) = {
            self.mut_node(self.get_index(id)).update_transform(parent_matrix)
        };

        if let Some(sibling) = &sibling {
            self.update_transform(sibling, parent_matrix);
        }

        if let Some(child) = &child {
            self.update_transform(child, Some(world_matrix));
        }
    }

    #[inline]
    pub fn ref_nodes(&self) -> Vec<&ModelNode<NodeID>> {
        let mut nodes = Vec::with_capacity(self.nodes.capacity());
        self.ref_nodes_recursion(&mut nodes, &self.root_id);
        return nodes;
    }

    fn ref_nodes_recursion<'a>(&'a self, nodes: &mut Vec<&'a ModelNode<NodeID>>, id: &NodeID) {
        let node = self.ref_node(self.get_index(id));
        nodes.push(node);

        if let Some(sibling) = &node.sibling {
            self.ref_nodes_recursion(nodes, sibling);
        }

        if let Some(child) = &node.child {
            self.ref_nodes_recursion(nodes, child);
        }
    }
}
