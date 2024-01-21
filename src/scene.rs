use std::any::Any;
use std::any::TypeId;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

static ALLOCATOR: AtomicUsize = AtomicUsize::new(1);

/// # Component
pub trait Component: 'static + Clone {}

/// # Node
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Node {
    id: usize,
}

impl Node {
    fn new() -> Self {
        Self {
            id: ALLOCATOR.fetch_add(1, Ordering::Relaxed),
        }
    }
}

struct ComponentIndex {
    type_id: TypeId,
    index: usize,
}

struct NodeIndex {
    node: Node,
    index: usize,
}

trait DynamicComponentTable {
    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn remove(&mut self, node: Node);
}

struct ComponentTable<T> {
    node_indexes: Vec<NodeIndex>,
    items: Vec<T>,
}

impl<T> ComponentTable<T> {
    fn new() -> Self {
        Self {
            node_indexes: Vec::new(),
            items: Vec::new(),
        }
    }

    fn add(&mut self, node: Node, value: T) {
        if self.node_index(node).is_none() {
            let index = self.items.len();
            self.node_indexes.push(NodeIndex { node, index });
            self.items.push(value);
        }
    }

    fn get(&self, node: Node) -> Option<&T> {
        self.node_index(node).map(|index| &self.items[index])
    }

    fn set(&mut self, node: Node, value: T) {
        if let Some(index) = self.node_index(node) {
            self.items[index] = value;
        }
    }

    fn remove(&mut self, node: Node) {
        let index = self
            .node_indexes
            .binary_search_by_key(&node.id, |n| n.node.id);
        if let Ok(index) = index {
            let node_index = self.node_indexes[index].index;
            self.items.swap_remove(node_index);
            self.node_indexes.remove(index);

            let moved_node_index = self.items.len();
            if moved_node_index != node_index {
                for n in &mut self.node_indexes {
                    if n.index == moved_node_index {
                        n.index = node_index;
                        break;
                    }
                }
            }
        }
    }

    fn node_index(&self, node: Node) -> Option<usize> {
        self.node_indexes
            .binary_search_by_key(&node.id, |n| n.node.id)
            .map(|index| self.node_indexes[index].index)
            .ok()
    }
}

impl<T: Component> DynamicComponentTable for ComponentTable<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn remove(&mut self, node: Node) {
        self.remove(node);
    }
}

/// # Scene
pub struct Scene {
    nodes: Vec<Node>,
    component_indexes: Vec<ComponentIndex>,
    component_tables: Vec<Box<dyn DynamicComponentTable>>,
}

impl Scene {
    /// Returns an empty scene.
    pub const fn new() -> Self {
        Self {
            nodes: Vec::new(),
            component_indexes: Vec::new(),
            component_tables: Vec::new(),
        }
    }

    /// Returns true if the scene contains the given node.
    pub fn contains(&self, node: Node) -> bool {
        self.nodes.binary_search_by_key(&node.id, |n| n.id).is_ok()
    }

    /// Creates a new node and adds it to the scene.
    pub fn spawn(&mut self) -> Node {
        let node = Node::new();
        self.nodes.push(node);
        node
    }

    /// Removes the given node from the scene.
    pub fn despawn(&mut self, node: Node) {
        let index = self.nodes.binary_search_by_key(&node.id, |n| n.id);
        if let Ok(index) = index {
            self.nodes.remove(index);

            for table in &mut self.component_tables {
                table.remove(node);
            }
        }
    }

    /// Adds the component to the node.
    pub fn add<T: Component>(&mut self, node: Node, value: T) {
        let component_index = match self.component_index::<T>() {
            Some(index) => index,
            None => {
                let index = self.component_tables.len();
                self.component_indexes.push(ComponentIndex {
                    type_id: TypeId::of::<T>(),
                    index,
                });
                self.component_indexes.sort_by_key(|c| c.type_id);
                self.component_tables
                    .push(Box::new(ComponentTable::<T>::new()));

                index
            }
        };

        self.component_tables[component_index]
            .as_any_mut()
            .downcast_mut::<ComponentTable<T>>()
            .unwrap()
            .add(node, value);
    }

    /// Returns the component value for the given node.
    pub fn get<T: Component>(&self, node: Node) -> Option<T> {
        if let Some(component_index) = self.component_index::<T>() {
            self.component_tables[component_index]
                .as_any()
                .downcast_ref::<ComponentTable<T>>()
                .unwrap()
                .get(node)
                .cloned()
        } else {
            None
        }
    }

    /// Sets the component value for the given node.
    pub fn set<T: Component>(&mut self, node: Node, value: T) {
        if let Some(component_index) = self.component_index::<T>() {
            self.component_tables[component_index]
                .as_any_mut()
                .downcast_mut::<ComponentTable<T>>()
                .unwrap()
                .set(node, value);
        }
    }

    /// Removes the component from the given node.
    pub fn remove<T: Component>(&mut self, node: Node) {
        if let Some(component_index) = self.component_index::<T>() {
            self.component_tables[component_index]
                .as_any_mut()
                .downcast_mut::<ComponentTable<T>>()
                .unwrap()
                .remove(node);
        }
    }

    fn component_index<T: Component>(&self) -> Option<usize> {
        self.component_indexes
            .binary_search_by_key(&TypeId::of::<T>(), |c| c.type_id)
            .map(|index| self.component_indexes[index].index)
            .ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Component for u32 {}

    #[test]
    fn spawn_contains_returns_true() {
        let mut scene = Scene::new();

        let node = scene.spawn();

        assert!(scene.contains(node));
    }

    #[test]
    fn spawn_get_returns_none() {
        let mut scene = Scene::new();

        let node = scene.spawn();

        assert_eq!(scene.get::<u32>(node), None);
    }

    #[test]
    fn despawn_contains_returns_false() {
        let mut scene = Scene::new();
        let node = scene.spawn();

        scene.despawn(node);

        assert!(!scene.contains(node));
    }

    #[test]
    fn despawn_get_returns_none() {
        let mut scene = Scene::new();
        let node = scene.spawn();
        let value = 17u32;
        scene.add(node, value);

        scene.despawn(node);

        assert_eq!(scene.get::<u32>(node), None);
    }

    #[test]
    fn add_get_returns_value() {
        let mut scene = Scene::new();
        let node = scene.spawn();
        let value = 17u32;

        scene.add(node, value);

        assert_eq!(scene.get::<u32>(node), Some(value));
    }

    #[test]
    fn set_get_returns_new_value() {
        let mut scene = Scene::new();
        let node = scene.spawn();
        let value = 17u32;
        scene.add(node, value);

        let new_value = 192u32;
        scene.set(node, new_value);

        assert_eq!(scene.get::<u32>(node), Some(new_value));
    }

    #[test]
    fn remove_get_returns_none() {
        let mut scene = Scene::new();
        let node = scene.spawn();
        let value = 17u32;
        scene.add(node, value);

        scene.remove::<u32>(node);

        assert_eq!(scene.get::<u32>(node), None);
    }
}
