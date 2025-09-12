use std::marker::PhantomData;

use crate::ecs::entity::Entity;
use crate::ecs::ecs::{ComponentStore, Ecs};

pub struct Query<'w, T: 'static + Send + Sync> {
    ecs: &'w Ecs,
    _m: PhantomData<T>,
}

impl<'w, T: 'static + Send + Sync> Query<'w, T> {
    pub fn new(ecs: &'w Ecs) -> Self { Self { ecs, _m: PhantomData } }

    pub fn iter(&self) -> impl Iterator<Item = (Entity, &T)> {
        let opt: Option<&ComponentStore<T>> = self.ecs.get_store::<T>();
        let alive = &self.ecs;
        let items: Vec<(Entity, &T)> = match opt {
            Some(store) => store
                .map
                .iter()
                .filter_map(|(e, c)| if alive.is_alive(*e) { Some((*e, c)) } else { None })
                .collect(),
            None => Vec::new(),
        };
        items.into_iter()
    }

    pub fn iter_with<F: Filter>(&self, filter: F) -> impl Iterator<Item = (Entity, &T)> {
        let opt: Option<&ComponentStore<T>> = self.ecs.get_store::<T>();
        let ecs = self.ecs;
        let items: Vec<(Entity, &T)> = match opt {
            Some(store) => store
                .map
                .iter()
                .filter_map(|(e, c)| if ecs.is_alive(*e) && filter.matches(ecs, *e) { Some((*e, c)) } else { None })
                .collect(),
            None => Vec::new(),
        };
        items.into_iter()
    }
}

pub struct Query2<'w, A: 'static + Send + Sync, B: 'static + Send + Sync> {
    ecs: &'w Ecs,
    _m: PhantomData<(A, B)>,
}

impl<'w, A: 'static + Send + Sync, B: 'static + Send + Sync> Query2<'w, A, B> {
    pub fn new(ecs: &'w Ecs) -> Self { Self { ecs, _m: PhantomData } }

    pub fn iter(&self) -> impl Iterator<Item = (Entity, &A, &B)> {
        let sa: Option<&ComponentStore<A>> = self.ecs.get_store::<A>();
        let sb: Option<&ComponentStore<B>> = self.ecs.get_store::<B>();
        let items: Vec<(Entity, &A, &B)> = match (sa, sb) {
            (Some(sa), Some(sb)) => {
                // iterate on smaller store for efficiency
                if sa.map.len() <= sb.map.len() {
                    sa.map.iter().filter_map(|(e, a)| {
                        if self.ecs.is_alive(*e) { sb.map.get(e).map(|b| (*e, a, b)) } else { None }
                    }).collect()
                } else {
                    sb.map.iter().filter_map(|(e, b)| {
                        if self.ecs.is_alive(*e) { sa.map.get(e).map(|a| (*e, a, b)) } else { None }
                    }).collect()
                }
            }
            _ => Vec::new(),
        };
        items.into_iter()
    }

    pub fn iter_with<F: Filter>(&self, filter: F) -> impl Iterator<Item = (Entity, &A, &B)> {
        let sa: Option<&ComponentStore<A>> = self.ecs.get_store::<A>();
        let sb: Option<&ComponentStore<B>> = self.ecs.get_store::<B>();
        let items: Vec<(Entity, &A, &B)> = match (sa, sb) {
            (Some(sa), Some(sb)) => {
                if sa.map.len() <= sb.map.len() {
                    sa.map.iter().filter_map(|(e, a)| {
                        if self.ecs.is_alive(*e) && filter.matches(self.ecs, *e) { sb.map.get(e).map(|b| (*e, a, b)) } else { None }
                    }).collect()
                } else {
                    sb.map.iter().filter_map(|(e, b)| {
                        if self.ecs.is_alive(*e) && filter.matches(self.ecs, *e) { sa.map.get(e).map(|a| (*e, a, b)) } else { None }
                    }).collect()
                }
            }
            _ => Vec::new(),
        };
        items.into_iter()
    }
}

// --------- Filters ---------
pub trait Filter {
    fn matches(&self, ecs: &Ecs, e: Entity) -> bool;
}

pub struct With<T: 'static + Send + Sync>(PhantomData<T>);
pub struct Without<T: 'static + Send + Sync>(PhantomData<T>);

impl<T: 'static + Send + Sync> Default for With<T> { fn default() -> Self { Self(PhantomData) } }
impl<T: 'static + Send + Sync> Default for Without<T> { fn default() -> Self { Self(PhantomData) } }

impl<T: 'static + Send + Sync> Filter for With<T> {
    fn matches(&self, ecs: &Ecs, e: Entity) -> bool { ecs.has::<T>(e) }
}
impl<T: 'static + Send + Sync> Filter for Without<T> {
    fn matches(&self, ecs: &Ecs, e: Entity) -> bool { !ecs.has::<T>(e) }
}

// Convenience helpers on Ecs
impl Ecs {
    pub fn query<'w, T: 'static + Send + Sync>(&'w self) -> Query<'w, T> { Query::new(self) }
    pub fn query2<'w, A: 'static + Send + Sync, B: 'static + Send + Sync>(&'w self) -> Query2<'w, A, B> { Query2::new(self) }
}
