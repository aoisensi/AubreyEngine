# Scheduling

システム実行順を段階的に制御するための仕組み。

- 段階: `Stage::{PreStartup, Startup, PostStartup, First, PreUpdate, Update, PostUpdate, Last}`
- 優先度: `order: i32` 小さいほど先に実行（同値は登録順）
- ラベル依存: ラベル（`&'static str`）を付け、`before`/`after` で相対順序を指定

## API（App）

- 基本登録
  - `add_systems(stage, system)`
  - `add_systems_ordered(stage, order, system)`

- ラベル付き登録
  - `add_systems_with_label(stage, label, system)`
  - `add_systems_with_deps(stage, label, before, after, order, system)`

`before`/`after` は `&[&'static str]`。同ステージ内で有効。

注意:
- ラベルはステージ内でユニークにすること（重複すると依存が曖昧）。
- 依存が循環した場合は、優先度（order）と登録順のみにフォールバックする。

## 実行規則

1. まず `order` の小さい順に安定ソート。
2. ラベル依存（`before`/`after`）に基づくトポロジカルソートを行い、
   依存が満たされる範囲で 1 の優先度を保ちながら順序を決定。
3. 循環があれば 1 の順序で実行（警告は現状出さない）。

## 例

```rust
use aubrey_core::app::{App, Stage};

let mut app = App::new();

app
  .add_systems_with_label(Stage::Update, "input", |ecs| { /* ... */ })
  .add_systems_with_deps(Stage::Update, "physics", &["render"], &[], 0, |ecs| { /* before render */ })
  .add_systems_with_deps(Stage::Update, "render", &[], &["input"], 10, |ecs| { /* after input */ });
```

