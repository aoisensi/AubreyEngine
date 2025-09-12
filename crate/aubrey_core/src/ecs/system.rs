use crate::ecs::ecs::Ecs;

pub trait System: Send {
    fn run(&mut self, ecs: &mut Ecs);
}

impl<F> System for F
where
    F: FnMut(&mut Ecs) + Send,
{
    fn run(&mut self, ecs: &mut Ecs) {
        (self)(ecs);
    }
}
