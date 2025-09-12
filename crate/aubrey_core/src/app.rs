
pub use crate::ecs::Stage;
pub use crate::ecs::entity::Entity;
pub use crate::ecs::Commands;
pub use crate::ecs::{Bundle, One as OneComponent};
use crate::ecs::Ecs;
use crate::ecs::schedule::Schedules;

// Appを終了させるためのリソース。存在すればrunループを抜ける。
pub struct AppExit;

/// Bevy風の最小ECSをまとめたエントリポイント。
/// 使い方例:
/// ```ignore
/// use aubrey_core::app::{App, Stage};
/// 
/// let mut app = App::new();
/// app.insert_resource(0usize)
///     .add_systems(Stage::Startup, |ecs| {
///         let e = ecs.spawn_empty();
///         ecs.insert(e, 123i32);
///     })
///     .add_systems(Stage::Update, |ecs| {
///         // リソースやコンポーネントへアクセス
///         if let Some(counter) = ecs.get_resource_mut::<usize>() {
///             *counter += 1;
///         }
///     });
/// 
/// app.run();
/// ```
pub struct App {
    ecs: Ecs,
    schedules: Schedules,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self { ecs: Ecs::new(), schedules: Schedules::new() }
    }

    // Bevy-like API surface
    pub fn add_systems<F>(&mut self, stage: Stage, system: F) -> &mut Self
    where
        F: crate::ecs::system::System + 'static,
    {
        self.schedules.add_system(stage, Box::new(system));
        self
    }

    pub fn add_systems_ordered<F>(&mut self, stage: Stage, order: i32, system: F) -> &mut Self
    where
        F: crate::ecs::system::System + 'static,
    {
        self.schedules.add_system_with_order(stage, order, Box::new(system));
        self
    }

    pub fn add_systems_with_label<F>(&mut self, stage: Stage, label: &'static str, system: F) -> &mut Self
    where
        F: crate::ecs::system::System + 'static,
    {
        self.schedules.add_system_with_label(stage, label, 0, Box::new(system));
        self
    }

    pub fn add_systems_with_deps<F>(&mut self, stage: Stage, label: &'static str, before: &[&'static str], after: &[&'static str], order: i32, system: F) -> &mut Self
    where
        F: crate::ecs::system::System + 'static,
    {
        self.schedules.add_system_with_deps(stage, Some(label), before, after, order, Box::new(system));
        self
    }

    pub fn run(&mut self) {
        // 1フレーム分を実行（Startup系を未実行なら含む）
        loop {
            self.schedules.run_frame(&mut self.ecs);
            if self.ecs.get_resource::<AppExit>().is_some() { break; }
        }
    }

    pub fn update(&mut self) {
        // run() と同じく1フレーム分を進める
        self.schedules.run_frame(&mut self.ecs);
    }

    // --- Resource APIs ---
    pub fn insert_resource<T: 'static + Send + Sync>(&mut self, value: T) -> &mut Self {
        self.ecs.insert_resource::<T>(value);
        self
    }

    pub fn resource<T: 'static + Send + Sync>(&self) -> Option<&T> {
        self.ecs.get_resource::<T>()
    }

    pub fn resource_mut<T: 'static + Send + Sync>(&mut self) -> Option<&mut T> {
        self.ecs.get_resource_mut::<T>()
    }

    // --- Entity/Component APIs ---
    pub fn spawn_empty(&mut self) -> Entity {
        self.ecs.spawn_empty()
    }

    pub fn despawn(&mut self, entity: Entity) {
        self.ecs.despawn(entity)
    }

    pub fn insert_component<T: 'static + Send + Sync>(&mut self, entity: Entity, component: T) {
        self.ecs.insert::<T>(entity, component)
    }

    pub fn get_component<T: 'static + Send + Sync>(&self, entity: Entity) -> Option<&T> {
        self.ecs.get::<T>(entity)
    }

    pub fn get_component_mut<T: 'static + Send + Sync>(&mut self, entity: Entity) -> Option<&mut T> {
        self.ecs.get_mut::<T>(entity)
    }

    pub fn spawn<T: Bundle>(&mut self, bundle: T) -> Entity { self.ecs.spawn(bundle) }
    pub fn spawn_one<T: 'static + Send + Sync>(&mut self, component: T) -> Entity { self.ecs.spawn_one(component) }

    pub fn commands(&mut self) -> &mut Commands { self.ecs.commands() }

    // 手動でアプリ終了を要求（エディタやテスト用）
    pub fn request_exit(&mut self) {
        self.ecs.insert_resource(AppExit);
    }
}

// System内から使える終了ヘルパ
pub fn request_app_exit(ecs: &mut crate::ecs::ecs::Ecs) {
    ecs.insert_resource(AppExit);
}



