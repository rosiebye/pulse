use std::any::Any;
use std::any::TypeId;
use std::cell::Ref;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use nohash::IntMap;
use nohash::IntSet;

static ALLOCATOR: AtomicUsize = AtomicUsize::new(1);

/// # Component
pub trait Component: 'static + Clone + PartialEq {}

/// # Component Event
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ComponentEvent {
    /// Component was added to the node.
    Added(Node),
    /// Component was modified for node.
    Modified(Node),
    /// Component was removed from node.
    Removed(Node),
}

/// # Node
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
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

impl nohash::IsEnabled for Node {}

trait DynamicComponentTable {
    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn remove(&mut self, node: Node);

    fn clear_events(&mut self);
}

struct ComponentTable<T> {
    node_indexes: IntMap<Node, usize>,
    items: Vec<T>,
    events: Vec<ComponentEvent>,
}

impl<T: Component> ComponentTable<T> {
    fn new() -> Self {
        Self {
            node_indexes: IntMap::default(),
            items: Vec::new(),
            events: Vec::new(),
        }
    }

    fn add(&mut self, node: Node, value: T) {
        if !self.node_indexes.contains_key(&node) {
            let index = self.items.len();
            self.node_indexes.insert(node, index);
            self.items.push(value);
            self.events.push(ComponentEvent::Added(node));
        }
    }

    fn get(&self, node: Node) -> Option<&T> {
        self.node_indexes
            .get(&node)
            .map(|index| &self.items[*index])
    }

    fn set(&mut self, node: Node, value: T) {
        if let Some(index) = self.node_indexes.get(&node) {
            if self.items[*index] != value {
                self.items[*index] = value;
                self.events.push(ComponentEvent::Modified(node));
            }
        }
    }

    fn remove(&mut self, node: Node) {
        if let Some(index) = self.node_indexes.remove(&node) {
            self.events.push(ComponentEvent::Removed(node));
            self.items.swap_remove(index);

            let moved_index = self.items.len();
            if moved_index != index {
                for node_index in &mut self.node_indexes.values_mut() {
                    if *node_index == moved_index {
                        *node_index = index;
                        break;
                    }
                }
            }
        }
    }

    fn events(&self) -> &[ComponentEvent] {
        &self.events
    }

    fn clear_events(&mut self) {
        self.events.clear();
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

    fn clear_events(&mut self) {
        self.clear_events();
    }
}

/// # Scene
pub struct Scene {
    nodes: IntSet<Node>,
    parents: IntMap<Node, Node>,
    children: IntMap<Node, Vec<Node>>,
    component_indexes: RefCell<BTreeMap<TypeId, usize>>,
    component_tables: RefCell<Vec<Box<dyn DynamicComponentTable>>>,
}

impl Scene {
    /// Returns an empty scene.
    pub fn new() -> Self {
        Self {
            nodes: IntSet::default(),
            parents: IntMap::default(),
            children: IntMap::default(),
            component_indexes: RefCell::new(BTreeMap::new()),
            component_tables: RefCell::new(Vec::new()),
        }
    }

    /// Returns true if the scene contains the given node.
    pub fn contains(&self, node: Node) -> bool {
        self.nodes.contains(&node)
    }

    /// Creates a new node and adds it to the scene.
    pub fn spawn(&mut self) -> Node {
        let node = Node::new();
        self.nodes.insert(node);
        node
    }

    /// Removes the given node from the scene.
    pub fn despawn(&mut self, node: Node) {
        if self.contains(node) {
            Self::despawn_internal(
                &mut self.nodes,
                &mut self.parents,
                &mut self.children,
                &mut self.component_tables.borrow_mut(),
                node,
            );
            self.remove_parent(node);
        }
    }

    fn despawn_internal(
        nodes: &mut IntSet<Node>,
        parents: &mut IntMap<Node, Node>,
        children: &mut IntMap<Node, Vec<Node>>,
        component_tables: &mut Vec<Box<dyn DynamicComponentTable>>,
        node: Node,
    ) {
        if nodes.remove(&node) {
            for child in children.remove(&node).into_iter().flatten() {
                Self::despawn_internal(nodes, parents, children, component_tables, child);
            }

            for table in component_tables {
                table.remove(node);
            }

            parents.remove(&node);
        }
    }

    /// Returns the parent node for the given node.
    pub fn get_parent(&self, node: Node) -> Option<Node> {
        self.parents.get(&node).copied()
    }

    /// Sets the parent node for the given node. Keeps the existing parent if the given parent
    /// doesn't exist in the scene or if the given parent would create a node cycle.
    pub fn set_parent(&mut self, node: Node, parent: Node) {
        if !self.contains(node) || !self.contains(parent) {
            return;
        }

        let mut root = Some(parent);
        while root.is_some() {
            if root.unwrap() == node {
                return;
            }

            root = self.get_parent(root.unwrap());
        }

        self.remove_parent(node);
        self.parents.insert(node, parent);

        if !self.children.contains_key(&parent) {
            self.children.insert(parent, Vec::new());
        }

        self.children.get_mut(&parent).unwrap().push(node);
    }

    /// Removes the parent node for the given node.
    pub fn remove_parent(&mut self, node: Node) {
        if let Some(parent) = self.parents.remove(&node) {
            if let Some(children) = self.children.get_mut(&parent) {
                let mut i = 0;
                while i < children.len() {
                    if children[i] == node {
                        children.remove(i);
                        break;
                    }
                    i += 1;
                }
            }
        }
    }

    /// Returns the root nodes i.e. nodes that don't have a parent.
    pub fn get_root_nodes<'a>(&'a self) -> impl 'a + Iterator<Item = Node> {
        self.nodes
            .iter()
            .copied()
            .filter(|node| self.get_parent(*node).is_none())
    }

    /// Returns the children for the given node.
    pub fn get_children(&self, node: Node) -> Option<&[Node]> {
        self.children.get(&node).map(Vec::as_slice)
    }

    /// Adds the component to the node.
    pub fn add<T: Component>(&self, node: Node, value: T) {
        let component_index = match self.component_index::<T>() {
            Some(index) => index,
            None => {
                let index = self.component_tables.borrow().len();
                self.component_indexes
                    .borrow_mut()
                    .insert(TypeId::of::<T>(), index);
                self.component_tables
                    .borrow_mut()
                    .push(Box::new(ComponentTable::<T>::new()));

                index
            }
        };

        self.component_tables.borrow_mut()[component_index]
            .as_any_mut()
            .downcast_mut::<ComponentTable<T>>()
            .unwrap()
            .add(node, value);
    }

    /// Returns the component value for the given node.
    pub fn get<T: Component>(&self, node: Node) -> Option<T> {
        if let Some(component_index) = self.component_index::<T>() {
            self.component_tables.borrow()[component_index]
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
    pub fn set<T: Component>(&self, node: Node, value: T) {
        if let Some(component_index) = self.component_index::<T>() {
            self.component_tables.borrow_mut()[component_index]
                .as_any_mut()
                .downcast_mut::<ComponentTable<T>>()
                .unwrap()
                .set(node, value);
        }
    }

    /// Sets the component value for the given node or adds the component.
    pub fn set_or_add<T: Component>(&self, node: Node, value: T) {
        self.add(node, value.clone());
        self.set(node, value);
    }

    /// Removes the component from the given node.
    pub fn remove<T: Component>(&self, node: Node) {
        if let Some(component_index) = self.component_index::<T>() {
            self.component_tables.borrow_mut()[component_index]
                .as_any_mut()
                .downcast_mut::<ComponentTable<T>>()
                .unwrap()
                .remove(node);
        }
    }

    /// Returns the component events for the given component.
    pub fn events<T: Component>(&self) -> Ref<[ComponentEvent]> {
        if let Some(component_index) = self.component_index::<T>() {
            Ref::map(self.component_tables.borrow(), |table| {
                table[component_index]
                    .as_any()
                    .downcast_ref::<ComponentTable<T>>()
                    .unwrap()
                    .events()
            })
        } else {
            Ref::map(self.component_tables.borrow(), |_| &[])
        }
    }

    /// Clears the component events for all the components.
    pub fn clear_events(&self) {
        for table in self.component_tables.borrow_mut().iter_mut() {
            table.clear_events();
        }
    }

    fn component_index<T: Component>(&self) -> Option<usize> {
        self.component_indexes
            .borrow()
            .get(&TypeId::of::<T>())
            .copied()
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Deref;

    use super::*;

    impl Component for u32 {}

    #[test]
    fn spawn_contains_returns_true() {
        let mut scene = Scene::new();

        let node = scene.spawn();

        assert!(scene.contains(node));
    }

    #[test]
    fn spawn_get_parent_returns_none() {
        let mut scene = Scene::new();

        let node = scene.spawn();

        assert_eq!(scene.get_parent(node), None);
    }

    #[test]
    fn spawn_get_children_returns_none() {
        let mut scene = Scene::new();

        let node = scene.spawn();

        assert_eq!(scene.get_children(node), None);
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
    fn despawn_get_parent_returns_none() {
        let mut scene = Scene::new();
        let parent = scene.spawn();
        let node = scene.spawn();
        scene.set_parent(node, parent);

        scene.despawn(node);

        assert_eq!(scene.get_parent(node), None);
    }

    #[test]
    fn despawn_get_children_returns_none() {
        let mut scene = Scene::new();
        let parent = scene.spawn();
        let node = scene.spawn();
        scene.set_parent(node, parent);

        scene.despawn(node);

        assert_eq!(scene.get_children(node), None);
    }

    #[test]
    fn despawn_parent_contains_returns_false() {
        let mut scene = Scene::new();
        let parent = scene.spawn();
        let node = scene.spawn();
        scene.set_parent(node, parent);

        scene.despawn(parent);

        assert!(!scene.contains(node));
    }

    #[test]
    fn despawn_parent_get_parent_returns_none() {
        let mut scene = Scene::new();
        let parent = scene.spawn();
        let node = scene.spawn();
        scene.set_parent(node, parent);

        scene.despawn(parent);

        assert_eq!(scene.get_parent(node), None);
    }

    #[test]
    fn despawn_parent_get_children_returns_none() {
        let mut scene = Scene::new();
        let parent = scene.spawn();
        let node = scene.spawn();
        scene.set_parent(node, parent);

        scene.despawn(parent);

        assert_eq!(scene.get_children(node), None);
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
    fn set_parent_get_parent_returns_parent() {
        let mut scene = Scene::new();
        let parent = scene.spawn();
        let node = scene.spawn();

        scene.set_parent(node, parent);

        assert_eq!(scene.get_parent(node), Some(parent));
    }

    #[test]
    fn set_parent_get_children_returns_node() {
        let mut scene = Scene::new();
        let parent = scene.spawn();
        let node = scene.spawn();

        scene.set_parent(node, parent);

        assert_eq!(scene.get_children(parent), Some([node].as_slice()));
    }

    #[test]
    fn set_parent_removed_node_get_parent_returns_previous_parent() {
        let mut scene = Scene::new();
        let parent = scene.spawn();
        let node = scene.spawn();
        let despawned = scene.spawn();
        scene.set_parent(node, parent);

        scene.despawn(despawned);
        scene.set_parent(node, despawned);

        assert_eq!(scene.get_parent(node), Some(parent));
    }

    #[test]
    fn set_parent_self_get_parent_returns_previous_parent() {
        let mut scene = Scene::new();
        let parent = scene.spawn();
        let node = scene.spawn();
        scene.set_parent(node, parent);

        scene.set_parent(node, node);

        assert_eq!(scene.get_parent(node), Some(parent));
    }

    #[test]
    fn set_parent_child_get_parent_returns_none() {
        let mut scene = Scene::new();
        let parent = scene.spawn();
        let node = scene.spawn();
        scene.set_parent(node, parent);

        scene.set_parent(parent, node);

        assert_eq!(scene.get_parent(parent), None);
    }

    #[test]
    fn remove_parent_get_parent_returns_none() {
        let mut scene = Scene::new();
        let parent = scene.spawn();
        let node = scene.spawn();
        scene.set_parent(node, parent);

        scene.remove_parent(node);

        assert_eq!(scene.get_parent(node), None);
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
    fn add_events_returns_added_event() {
        let mut scene = Scene::new();
        let node = scene.spawn();
        let value = 17u32;

        scene.add(node, value);

        assert_eq!(
            scene.events::<u32>().deref(),
            &[ComponentEvent::Added(node)]
        );
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
    fn set_events_returns_modified_event() {
        let mut scene = Scene::new();
        let node = scene.spawn();
        let value = 17u32;
        scene.add(node, value);

        let new_value = 192u32;
        scene.set(node, new_value);

        assert_eq!(
            scene.events::<u32>().deref(),
            &[ComponentEvent::Added(node), ComponentEvent::Modified(node)]
        );
    }

    #[test]
    fn set_existing_value_events_does_not_return_modified_event() {
        let mut scene = Scene::new();
        let node = scene.spawn();
        let value = 17u32;
        scene.add(node, value);

        scene.set(node, value);

        assert_eq!(
            scene.events::<u32>().deref(),
            &[ComponentEvent::Added(node)]
        );
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

    #[test]
    fn remove_events_returns_removed_event() {
        let mut scene = Scene::new();
        let node = scene.spawn();
        let value = 17u32;
        scene.add(node, value);

        scene.remove::<u32>(node);

        assert_eq!(
            scene.events::<u32>().deref(),
            &[ComponentEvent::Added(node), ComponentEvent::Removed(node)]
        );
    }

    #[test]
    fn clear_events_events_returns_empty() {
        let mut scene = Scene::new();
        let node = scene.spawn();
        let value = 17u32;
        scene.add(node, value);

        scene.clear_events();

        assert_eq!(scene.events::<u32>().deref(), &[]);
    }
}
