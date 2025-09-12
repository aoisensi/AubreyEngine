# TickComponent<C> 仕様

目的: 文脈 `C` ごとに対象へ定期処理を適用する汎用ドライバ。

## 背景
- ECS: `Component<TCtx>` のジェネリクスで文脈を型分離する。
- 文脈は型レベルで分かれ、同じ `Component` でも `Component<UiCtx>` と `Component<WorldCtx>` は別ストレージになる。
- `TickComponent<C>` は `C` が定義する対象へ一定間隔で処理を行う。

## コア型

```rust
/// 文脈の“種類”。各Cで実装する。
pub trait ContextKind: 'static {}

/// 文脈Cに特化したTickの処理定義。
pub trait TickCtx: ContextKind {
    /// クエリ型。Tickの対象を取得するために必要なコンポーネント束。
    type Query<'w>;
    /// 1ステップの処理。dtは経過時間（秒）。
    fn tick(item: Self::Query<'_>, dt: f32);
}

/// 文脈Cで駆動するTickコンポーネント。
pub struct TickComponent<C> {
    pub enabled: bool,   // 無効ならスキップ
    pub interval: f32,   // 実行間隔（秒）。0以下なら毎フレーム（可変ステップ）
    pub accum: f32,      // 蓄積時間（内部用）
    pub _p: core::marker::PhantomData<C>,
}

/// 文脈ローカル時間資源。
pub struct Time<C> {
    pub delta: f32,  // 前フレーム経過時間
    pub scale: f32,  // 時間スケール（0で停止）
    pub paused: bool,
}
```

## システム挙動（擬似コード）

```rust
fn system_tick<C: TickCtx>(world: &mut World, ctx: CtxHandle<C>) {
    let time = ctx_resource::<Time<C>>(world, ctx);
    let dt = if time.paused { 0.0 } else { time.delta * time.scale };

    for (mut t, target) in query_in_ctx::<(&mut TickComponent<C>, C::Query<'_>), C>(world, ctx) {
        if !t.enabled { continue; }

        if t.interval <= 0.0 {
            // 可変ステップ: 毎フレーム1回呼ぶ
            C::tick(target, dt);
        } else {
            // 固定ステップ: intervalを満たすまで繰り返す
            t.accum += dt;
            while t.accum >= t.interval {
                C::tick(target, t.interval);
                t.accum -= t.interval;
            }
            // 必要なら最大反復数に上限を設ける（スパイク耐性）
        }
    }
}
```

## 不変条件と安全性
- クエリは `(&mut TickComponent<C>, C::Query<'_>)` を同時に取得し、対象への二重可変参照を避ける。
- 同一エンティティに複数の `TickComponent<C>` を付けることは推奨しない（競合の元）。
- 文脈を跨いだ書き込みは行わない。`C` に閉じたデータだけを触る。

## 代表的ユースケース

- NPC思考（低頻度更新）
  ```rust
  struct WorldCtx; impl ContextKind for WorldCtx {}
  struct NpcAiTickCtx; impl ContextKind for NpcAiTickCtx {}
  impl TickCtx for NpcAiTickCtx {
      type Query<'w> = (&'w mut NpcState<WorldCtx>, &'w Perception<WorldCtx>);
      fn tick((s, p): Self::Query<'_>, dt: f32) { s.update(p, dt); }
  }
  // TickComponent<NpcAiTickCtx> { interval: 0.2 }
  ```

- UIブリンク（文脈ローカル時間）
  ```rust
  struct UiCtx; impl ContextKind for UiCtx {}
  struct UiBlinkTickCtx; impl ContextKind for UiBlinkTickCtx {}
  impl TickCtx for UiBlinkTickCtx {
      type Query<'w> = (&'w mut UiColor<UiCtx>,);
      fn tick((c,): Self::Query<'_>, dt: f32) { c.a = blink(c.a, dt); }
  }
  // Time<UiCtx>.scale を変えればUIだけスローモ可能
  ```

- パーティクル積分（固定ステップ）
  ```rust
  struct EffectCtx; impl ContextKind for EffectCtx {}
  struct ParticleTickCtx; impl ContextKind for ParticleTickCtx {}
  impl TickCtx for ParticleTickCtx {
      type Query<'w> = (&'w mut Position<EffectCtx>, &'w Velocity<EffectCtx>);
      fn tick((p, v): Self::Query<'_>, dt: f32) { p.xy += v.xy * dt; }
  }
  // TickComponent<ParticleTickCtx> { interval: 1.0/60.0 }
  ```

## 振る舞い詳細
- `enabled == false` の場合は `accum` を保持したまま処理をスキップ。
- `interval <= 0` は可変ステップモード。毎フレーム1回 `dt` で呼ぶ。
- `paused == true` のとき `dt` は0として扱う（固定ステップのaccumも増えない）。
- フレームスパイク時は `while` の反復が増える。上限設定や`dt`クランプを実装側で検討。
- 文脈ごとにスケジューラはチャンク化して実行し、キャッシュ局所性を高める。

## 相互運用
- `Time<C>` は文脈ローカルの`Resource`として提供する。
- 他の文脈のデータが必要なら、`C::Query`に入れず、別システムで集計/メッセージ経由にする。
- `TweenComponent<C2>` 等と併用する場合、同一ターゲットへの重複書き込みを避ける設計（タグ/優先度）を推奨。

## 実装メモ
- ストレージキーは `(TypeId::of::<TickComponent<C>>(), ContextId)` で分割。
- `query_in_ctx::<T, C>(world, ctx)` のようなAPIで文脈束縛クエリを提供。
- スパイク耐性のため `max_substeps` オプションを`TickComponent`ではなく文脈資源側に置く選択もあり。


