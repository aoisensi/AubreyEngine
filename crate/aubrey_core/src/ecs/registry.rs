use std::collections::HashMap;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct ComponentId(pub u64);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct ResourceId(pub u64);

#[derive(Default)]
pub struct Registry {
    next_component: u64,
    next_resource: u64,
    components_by_name: HashMap<String, ComponentId>,
    resources_by_name: HashMap<String, ResourceId>,
}

impl Registry {
    pub fn new() -> Self { Self::default() }

    pub fn register_component<S: Into<String>>(&mut self, name: S) -> ComponentId {
        let name = name.into();
        if let Some(id) = self.components_by_name.get(&name) { return *id; }
        let id = ComponentId(self.next_component);
        self.next_component += 1;
        self.components_by_name.insert(name, id);
        id
    }

    pub fn get_component<S: AsRef<str>>(&self, name: S) -> Option<ComponentId> {
        self.components_by_name.get(name.as_ref()).copied()
    }

    pub fn register_resource<S: Into<String>>(&mut self, name: S) -> ResourceId {
        let name = name.into();
        if let Some(id) = self.resources_by_name.get(&name) { return *id; }
        let id = ResourceId(self.next_resource);
        self.next_resource += 1;
        self.resources_by_name.insert(name, id);
        id
    }

    pub fn get_resource<S: AsRef<str>>(&self, name: S) -> Option<ResourceId> {
        self.resources_by_name.get(name.as_ref()).copied()
    }
}

