use std::ops::{Deref, DerefMut};

pub type NodeId = usize;

#[derive(Clone)]
pub struct Tree<T> {
    nodes: Vec<Node<T>>,
    root_id: Option<NodeId>,
}

impl<T> Tree<T> {
    pub fn new(root: T) -> Self {
        let root_node = Node {
            parent_id: None,
            first_child_id: None,
            last_child_id: None,
            prev_sibling_id: None,
            next_sibling_id: None,
            level: 1,
            data: root
        };
        Tree {
            nodes: vec![root_node],
            root_id: Some(0),
        }
    }

    pub fn clear(&mut self, root: T) {
        self.nodes.clear();
        let root_node = Node {
            parent_id: None,
            first_child_id: None,
            last_child_id: None,
            prev_sibling_id: None,
            next_sibling_id: None,
            level: 1,
            data: root
        };
        self.nodes.push(root_node);
        self.root_id = Some(0);
    }

    pub fn push(&mut self, parent_id: NodeId, data: T) -> Option<NodeId> {
        if let Some(mut parent) = self.get_mut(parent_id) {
            Some(parent.push(data))
        } else {
            None
        }
    }

    pub fn get(&self, id: NodeId) -> Option<NodeRef<T>> {
        if self.nodes.get(id).is_some() {
            Some(NodeRef {
                tree: self,
                id,
            })
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, id: NodeId) -> Option<NodeMut<T>> {
        if self.nodes.get(id).is_some() {
            Some(NodeMut {
                tree: self,
                id,
            })
        } else {
            None
        }
    }

    pub fn dfs_iter(&self) -> Dfs<T> {
        Dfs {
            tree: self,
            stack: vec![0]
        }
    }
}

pub struct NodeRef<'a, T> {
    tree: &'a Tree<T>,
    id: NodeId
}

impl<'a, T> Deref for NodeRef<'a, T> {
    type Target = Node<T>;

    fn deref(&self) -> &Node<T> {
        &self.tree.nodes[self.id]
    }
}

pub struct NodeMut<'a, T> {
    tree: &'a mut Tree<T>,
    id: NodeId,
}

impl <'a, T> Deref for NodeMut<'a, T> {
    type Target = Node<T>;

    fn deref(&self) -> &Node<T> {
        &self.tree.nodes[self.id]
    }
}

impl<'a, T> DerefMut for NodeMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Node<T> {
        &mut self.tree.nodes[self.id]
    }
}

impl<'a, T> NodeMut<'a, T> {
    pub fn id(&self) -> NodeId {
        self.id
    }

    pub fn data(&mut self) -> &mut T {
        &mut self.tree.nodes[self.id].data
    }

    pub fn push(&mut self, data: T) -> NodeId {
        let new_id = self.tree.nodes.len();

        let parent_level: u32;
        let prev_sibling_id: Option<NodeId>;
        {
            let mut parent = self.tree.get_mut(self.id).unwrap();
            parent_level = parent.level;
            prev_sibling_id = parent.last_child_id;

            parent.last_child_id = Some(new_id);
            if parent.first_child_id.is_none() {
                parent.first_child_id = parent.last_child_id;
            }
        }

        if let Some(prev_sibling_id) = prev_sibling_id {
            let prev_sibling = &mut self.tree.nodes[prev_sibling_id];
            prev_sibling.next_sibling_id = Some(new_id);
        }

        let new_node = {
            Node {
                parent_id: Some(self.id),
                first_child_id: None,
                last_child_id: None,
                prev_sibling_id,
                next_sibling_id: None,
                level: parent_level + 1,
                data
            }
        };

         self.tree.nodes.push(new_node);

        new_id
    }

    pub fn parent(&mut self) -> Option<NodeMut<T>> {
        if let Some(parent_id) = self.tree.nodes[self.id].parent_id {
            self.tree.get_mut(parent_id)
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct Node<T> {
    parent_id: Option<NodeId>,
    first_child_id: Option<NodeId>,
    last_child_id: Option<NodeId>,
    prev_sibling_id: Option<NodeId>,
    next_sibling_id: Option<NodeId>,
    level: u32,
    data: T,
}

impl<T> Node<T> {
    pub fn data(&self) -> &T {
        &self.data
    }

    pub fn level(&self) -> u32 {
        self.level
    }
}

pub struct Dfs<'a, T> {
    tree: &'a Tree<T>,
    stack: Vec<NodeId>,
}

impl<'a, T> Iterator for Dfs<'a, T> {
    type Item = NodeRef<'a, T>;

    fn next(&mut self) -> Option<NodeRef<'a, T>> {
        if let Some(node_id) = self.stack.pop() {
            let node = self.tree.get(node_id).unwrap();
            let mut child_id = node.last_child_id;
            while let Some(id) = child_id {
                self.stack.push(id);
                let child = &self.tree.nodes[id];
                child_id = child.prev_sibling_id;
            }
            Some(node)
        } else {
            None
        }
    }
}
