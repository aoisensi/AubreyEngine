# ECS (Entity Component System)

このエンジンのECSは、シンプルな型安全ストレージとリソース、スケジューリングで構成される。

- Entity: 数値ID。
- Component<T>: 任意の型Tをエンティティに付与（`'static + Send + Sync`）。
- Resources: グローバルな1個ずつのデータ格納（型で一意）。
- Systems: `fn(&mut Ecs)` を満たすクロージャ/関数。ステージに登録して実行。
- Scheduler: ステージ順・優先度・ラベル依存で実行順を制御。
- Commands: ステージ内での遅延操作（spawn/insert/despawn等）を蓄積し、ステージ末尾で適用。

## 主要API

```rust
use aubrey_core::app::{App, Stage};

let mut app = App::new();

// リソース
app.insert_resource(0usize);

// システム登録（基本）
app.add_systems(Stage::Startup, |ecs| {
    let e = ecs.spawn_empty();
    ecs.insert(e, 123i32);
});

// システム順序（order/label/before/after）
app
  .add_systems_with_label(Stage::Update, "input", |ecs| {/* ... */})
  .add_systems_with_deps(Stage::Update, "render", &[], &["input"], 10, |ecs| {/* ... */});

// 実行
app.run();
```

## クエリ

`Ecs::query<T>()` と `Ecs::query2<A, B>()` を提供。フィルタ `With<T>`/`Without<T>` で条件付け。

```rust
use aubrey_core::ecs::query::{With, Without};

for (e, pos) in ecs.query::<Position>().iter_with(With::<Velocity>::default()) {
    // ...
}
``;

## Commands（遅延操作）

`ecs.commands()` から取得して `spawn/insert/despawn` を発行。ステージ末のコミットで適用。

## スケジューリング

詳細は `docs/scheduling.md` を参照。ステージ、order、label依存で柔軟に制御できる。

