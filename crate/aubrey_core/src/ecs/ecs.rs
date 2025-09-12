use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};

use crate::ecs::entity::Entity;
use crate::resources::Resources;
use crate::ecs::bundle::Bundle;
use crate::ecs::registry::{Registry, ComponentId, ResourceId};

pub struct Ecs {
    next_id: u64,
    alive: HashSet<Entity>,
    // Per-type component storage boxed behind Any
    components: HashMap<TypeId, Box<dyn ErasedStore>>, 
    pub(crate) resources: Resources,
    // --- Dynamic (id-based) storage for script-friendly access ---
    dyn_components: HashMap<ComponentId, DynStore>,
    dyn_resources: HashMap<ResourceId, Box<dyn Any + Send + Sync>>,
    registry: Registry,
}

impl Ecs {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            alive: HashSet::new(),
            components: HashMap::new(),
            resources: Resources::new(),
            dyn_components: HashMap::new(),
            dyn_resources: HashMap::new(),
            registry: Registry::new(),
        }
    }

    pub fn spawn_empty(&mut self) -> Entity {
        let id = self.next_id;
        self.next_id += 1;
        let e = Entity::new(id);
        self.alive.insert(e);
        e
    }

    pub fn reserve_entity(&mut self) -> Entity {
        let id = self.next_id;
        self.next_id += 1;
        Entity::new(id)
    }

    pub fn despawn(&mut self, entity: Entity) {
        if self.alive.remove(&entity) {
            // remove from all component stores
            for store in self.components.values_mut() {
                store.remove(entity);
            }
        }
    }

    pub fn insert<T: 'static + Send + Sync>(&mut self, entity: Entity, component: T) {
        let type_id = TypeId::of::<T>();
        if !self.components.contains_key(&type_id) {
            self.components
                .insert(type_id, Box::new(ComponentStore::<T>::default()));
        }

        let store = self.get_store_mut::<T>().expect("Component store type mismatch");
        store.map.insert(entity, component);
    }

    pub fn get<T: 'static + Send + Sync>(&self, entity: Entity) -> Option<&T> {
        self.get_store::<T>()
            .and_then(|store| store.map.get(&entity))
    }

    pub fn get_mut<T: 'static + Send + Sync>(&mut self, entity: Entity) -> Option<&mut T> {
        self.get_store_mut::<T>()
            .and_then(|store| store.map.get_mut(&entity))
    }

    pub fn for_each<T: 'static + Send + Sync, F: FnMut(Entity, &T)>(&self, mut f: F) {
        if let Some(store) = self.get_store::<T>() {
            for (e, c) in &store.map {
                if self.alive.contains(e) {
                    f(*e, c);
                }
            }
        }
    }

    pub fn for_each_mut<T: 'static + Send + Sync, F: FnMut(Entity, &mut T)>(&mut self, mut f: F) {
        // two-pass to satisfy borrow checker
        let mut ents: Vec<Entity> = Vec::new();
        if let Some(store) = self.get_store::<T>() {
            for e in store.map.keys() {
                if self.alive.contains(e) { ents.push(*e); }
            }
        }
        if let Some(store) = self.get_store_mut::<T>() {
            for e in ents {
                if let Some(c) = store.map.get_mut(&e) {
                    f(e, c);
                }
            }
        }
    }

    pub fn spawn<T: Bundle>(&mut self, bundle: T) -> Entity {
        let e = self.spawn_empty();
        bundle.insert_immediate(self, e);
        e
    }

    pub fn spawn_one<T: 'static + Send + Sync>(&mut self, component: T) -> Entity {
        let e = self.spawn_empty();
        self.insert::<T>(e, component);
        e
    }

    // Resources proxies
    pub fn insert_resource<T: 'static + Send + Sync>(&mut self, value: T) {
        self.resources.insert::<T>(value);
    }

    pub fn get_resource<T: 'static + Send + Sync>(&self) -> Option<&T> {
        self.resources.get::<T>()
    }

    pub fn get_resource_mut<T: 'static + Send + Sync>(&mut self) -> Option<&mut T> {
        self.resources.get_mut::<T>()
    }

    pub fn remove_resource<T: 'static + Send + Sync>(&mut self) -> Option<T> {
        self.resources.remove::<T>()
    }

    pub fn is_alive(&self, e: Entity) -> bool { self.alive.contains(&e) }

    pub fn has<T: 'static + Send + Sync>(&self, entity: Entity) -> bool {
        self.get::<T>(entity).is_some()
    }

    // --------- Dynamic (id-based) API ---------
    pub fn registry(&mut self) -> &mut Registry { &mut self.registry }

    pub fn insert_dyn(&mut self, entity: Entity, comp: ComponentId, value: Box<dyn Any + Send + Sync>) {
        if !self.alive.contains(&entity) { return; }
        self.ensure_dyn_store(comp);
        self.dyn_components.get_mut(&comp).unwrap().insert_boxed(entity, value);
    }

    pub fn get_dyn(&self, entity: Entity, comp: ComponentId) -> Option<&dyn Any> {
        self.dyn_components
            .get(&comp)
            .and_then(|s| s.get(entity).map(|b| &**b as &dyn Any))
    }

    pub fn get_dyn_mut(&mut self, entity: Entity, comp: ComponentId) -> Option<&mut dyn Any> {
        self.dyn_components
            .get_mut(&comp)
            .and_then(|s| s.get_mut(entity).map(|b| &mut **b as &mut dyn Any))
    }

    pub fn has_dyn(&self, entity: Entity, comp: ComponentId) -> bool {
        self.dyn_components.get(&comp).map_or(false, |s| s.contains(entity))
    }

    pub fn remove_dyn(&mut self, entity: Entity, comp: ComponentId) {
        if let Some(s) = self.dyn_components.get_mut(&comp) { s.remove(entity); }
    }

    pub fn query_dyn(&self, comps: &[ComponentId]) -> Vec<Entity> {
        if comps.is_empty() { return self.alive.iter().copied().collect(); }
        // choose smallest set as base
        let mut ids = comps.to_vec();
        ids.sort_by_key(|c| self.dyn_components.get(c).map(|s| s.len()).unwrap_or(0));
        let mut out = Vec::new();
        if let Some(base) = self.dyn_components.get(&ids[0]) {
            for e in base.keys() {
                if self.is_alive(*e) && ids.iter().skip(1).all(|c| self.dyn_components.get(c).map_or(false, |s| s.contains(*e))) {
                    out.push(*e);
                }
            }
        }
        out
    }

    pub fn query_rows_dyn(&self, comps: &[ComponentId]) -> Vec<(Entity, Vec<&dyn Any>)> {
        let mut rows = Vec::new();
        for e in self.query_dyn(comps) {
            let mut cols: Vec<&dyn Any> = Vec::with_capacity(comps.len());
            for c in comps {
                if let Some(v) = self.get_dyn(e, *c) { cols.push(v); }
            }
            if cols.len() == comps.len() { rows.push((e, cols)); }
        }
        rows
    }

    pub fn insert_resource_dyn(&mut self, id: ResourceId, value: Box<dyn Any + Send + Sync>) {
        self.dyn_resources.insert(id, value);
    }

    pub fn get_resource_dyn(&self, id: ResourceId) -> Option<&dyn Any> {
        self.dyn_resources.get(&id).map(|b| &**b as &dyn Any)
    }

    pub fn get_resource_dyn_mut(&mut self, id: ResourceId) -> Option<&mut dyn Any> {
        self.dyn_resources.get_mut(&id).map(|b| &mut **b as &mut dyn Any)
    }

    pub fn remove_resource_dyn(&mut self, id: ResourceId) -> Option<Box<dyn Any + Send + Sync>> {
        self.dyn_resources.remove(&id)
    }

    pub fn commands(&mut self) -> &mut Commands {
        if !self.resources.contains::<Commands>() {
            self.insert_resource(Commands::default());
        }
        self.resources.get_mut::<Commands>().expect("Commands should be present")
    }

    // Low-level component store accessors for Query/erased ops
    pub(crate) fn get_store<T: 'static + Send + Sync>(&self) -> Option<&ComponentStore<T>> {
        self.components
            .get(&TypeId::of::<T>())
            .and_then(|s| s.as_any().downcast_ref::<ComponentStore<T>>())
    }

    pub(crate) fn get_store_mut<T: 'static + Send + Sync>(&mut self) -> Option<&mut ComponentStore<T>> {
        self.components
            .get_mut(&TypeId::of::<T>())
            .and_then(|s| s.as_any_mut().downcast_mut::<ComponentStore<T>>())
    }

    pub(crate) fn ensure_store<T: 'static + Send + Sync>(&mut self) {
        let type_id = TypeId::of::<T>();
        if !self.components.contains_key(&type_id) {
            self.components.insert(type_id, Box::new(ComponentStore::<T>::default()));
        }
    }

    fn ensure_dyn_store(&mut self, id: ComponentId) {
        if !self.dyn_components.contains_key(&id) {
            self.dyn_components.insert(id, DynStore::default());
        }
    }
}

pub(crate) struct ComponentStore<T: 'static + Send + Sync> {
    pub(crate) map: HashMap<Entity, T>,
}

impl<T: 'static + Send + Sync> Default for ComponentStore<T> {
    fn default() -> Self { Self { map: HashMap::new() } }
}

// removed obsolete remove_from_store logic; handled by ErasedStore::remove

// ----- Erased component store for Commands -----
pub(crate) trait ErasedStore: Send + Sync + 'static {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn insert_boxed(&mut self, entity: Entity, value: Box<dyn Any + Send + Sync>);
    fn remove(&mut self, entity: Entity);
}

impl<T: 'static + Send + Sync> ErasedStore for ComponentStore<T> {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn insert_boxed(&mut self, entity: Entity, value: Box<dyn Any + Send + Sync>) {
        let v = value.downcast::<T>().expect("Component type mismatch");
        self.map.insert(entity, *v);
    }
    fn remove(&mut self, entity: Entity) { self.map.remove(&entity); }
}

// ----- Commands (deferred ops per-stage) -----
#[derive(Default)]
pub struct Commands {
    queue: Vec<Command>,
}

pub(crate) enum Command {
    Spawn(Entity),
    Despawn(Entity),
    Insert { entity: Entity, type_id: TypeId, value: Box<dyn Any + Send + Sync> },
    InsertDyn { entity: Entity, comp_id: ComponentId, value: Box<dyn Any + Send + Sync> },
}

impl Commands {
    pub fn spawn_empty(&mut self, ecs: &mut Ecs) -> Entity {
        let e = ecs.reserve_entity();
        self.queue.push(Command::Spawn(e));
        e
    }

    pub fn despawn(&mut self, entity: Entity) {
        self.queue.push(Command::Despawn(entity));
    }

    pub fn insert<T: 'static + Send + Sync>(&mut self, ecs: &mut Ecs, entity: Entity, component: T) {
        ecs.ensure_store::<T>();
        self.queue.push(Command::Insert { entity, type_id: TypeId::of::<T>(), value: Box::new(component) });
    }

    pub fn insert_dyn(&mut self, _ecs: &mut Ecs, entity: Entity, comp_id: ComponentId, value: Box<dyn Any + Send + Sync>) {
        self.queue.push(Command::InsertDyn { entity, comp_id, value });
    }

    pub fn spawn<T: Bundle>(&mut self, ecs: &mut Ecs, bundle: T) -> Entity {
        let e = self.spawn_empty(ecs);
        bundle.write_commands(e, &mut self.queue, ecs);
        e
    }
    pub fn spawn_one<T: 'static + Send + Sync>(&mut self, ecs: &mut Ecs, component: T) -> Entity {
        let e = self.spawn_empty(ecs);
        ecs.ensure_store::<T>();
        self.queue.push(Command::Insert { entity: e, type_id: TypeId::of::<T>(), value: Box::new(component) });
        e
    }

    pub fn apply(&mut self, ecs: &mut Ecs) {
        for cmd in self.queue.drain(..) {
            match cmd {
                Command::Spawn(e) => {
                    ecs.alive.insert(e);
                }
                Command::Despawn(e) => {
                    ecs.alive.remove(&e);
                    // best-effort clean up per store
                    for store in ecs.components.values_mut() {
                        store.remove(e);
                    }
                    // clean up dynamic stores as well
                    for store in ecs.dyn_components.values_mut() {
                        store.remove(e);
                    }
                }
                Command::Insert { entity, type_id, value } => {
                    insert_erased(ecs, entity, type_id, value);
                }
                Command::InsertDyn { entity, comp_id, value } => {
                    ecs.insert_dyn(entity, comp_id, value);
                }
            }
        }
    }
}

fn insert_erased(ecs: &mut Ecs, entity: Entity, type_id: TypeId, value: Box<dyn Any + Send + Sync>) {
    // Store should already exist when using Commands::insert/Commands::spawn, but keep a defensive path
    if let Some(store) = ecs.components.get_mut(&type_id) {
        store.insert_boxed(entity, value);
        return;
    }
    // If no store, try a few common types then give up
    try_create_and_insert::<i32>(ecs, entity, type_id, value).unwrap_or_else(|value|
        try_create_and_insert::<u32>(ecs, entity, type_id, value).unwrap_or_else(|value|
        try_create_and_insert::<f32>(ecs, entity, type_id, value).unwrap_or_else(|value|
        try_create_and_insert::<usize>(ecs, entity, type_id, value).unwrap_or_else(|_value| {
            panic!("Unsupported component type for Commands::insert; use Ecs::insert<T> directly once to create store");
        }))))
}

fn try_create_and_insert<T: 'static + Send + Sync>(ecs: &mut Ecs, entity: Entity, type_id: TypeId, value: Box<dyn Any + Send + Sync>) -> Result<(), Box<dyn Any + Send + Sync>> {
    if type_id == TypeId::of::<T>() {
        ecs.components.insert(type_id, Box::new(ComponentStore::<T>::default()));
        let store = ecs.components.get_mut(&type_id).unwrap();
        store.insert_boxed(entity, value);
        Ok(())
    } else {
        Err(value)
    }
}

// ----- Dynamic store -----
#[derive(Default)]
struct DynStore {
    map: HashMap<Entity, Box<dyn Any + Send + Sync>>,
}

impl DynStore {
    fn insert_boxed(&mut self, entity: Entity, value: Box<dyn Any + Send + Sync>) {
        self.map.insert(entity, value);
    }
    fn get(&self, entity: Entity) -> Option<&Box<dyn Any + Send + Sync>> { self.map.get(&entity) }
    fn get_mut(&mut self, entity: Entity) -> Option<&mut Box<dyn Any + Send + Sync>> { self.map.get_mut(&entity) }
    fn remove(&mut self, entity: Entity) { self.map.remove(&entity); }
    fn contains(&self, entity: Entity) -> bool { self.map.contains_key(&entity) }
    fn len(&self) -> usize { self.map.len() }
    fn keys(&self) -> impl Iterator<Item = &Entity> { self.map.keys() }
}












