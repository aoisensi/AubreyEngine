#![allow(private_interfaces, non_snake_case)]

use std::any::TypeId;

use crate::ecs::entity::Entity;
use crate::ecs::ecs::{Command, Ecs};

pub trait Bundle: Send + 'static {
    fn insert_immediate(self, ecs: &mut Ecs, entity: Entity);
    fn write_commands(self, entity: Entity, out: &mut Vec<Command>, ecs: &mut Ecs);
}

pub struct Single<T: 'static + Send + Sync>(pub T);

impl<T: 'static + Send + Sync> Bundle for Single<T> {
    fn insert_immediate(self, ecs: &mut Ecs, entity: Entity) {
        ecs.insert::<T>(entity, self.0);
    }
    fn write_commands(self, entity: Entity, out: &mut Vec<Command>, ecs: &mut Ecs) {
        ecs.ensure_store::<T>();
        out.push(Command::Insert { entity, type_id: TypeId::of::<T>(), value: Box::new(self.0) });
    }
}

macro_rules! impl_bundle_tuple {
    ( $( $name:ident ),+ ) => {
        impl<$( $name ),+> Bundle for ( $( $name, )+ )
        where
            $( $name: 'static + Send + Sync ),+
        {
            fn insert_immediate(self, ecs: &mut Ecs, entity: Entity) {
                let ( $( $name, )+ ) = self;
                $( ecs.insert::<$name>(entity, $name); )+
            }
            fn write_commands(self, entity: Entity, out: &mut Vec<Command>, ecs: &mut Ecs) {
                let ( $( $name, )+ ) = self;
                $( {
                    ecs.ensure_store::<$name>();
                    out.push(Command::Insert { entity, type_id: TypeId::of::<$name>(), value: Box::new($name) });
                } )+
            }
        }
    }
}

impl_bundle_tuple!(A, B);
impl_bundle_tuple!(A, B, C);
impl_bundle_tuple!(A, B, C, D);
impl_bundle_tuple!(A, B, C, D, E);
