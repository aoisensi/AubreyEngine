# AubreyEngine

Rust製ゲームエンジン。ECSを拡張した ECS (Entity Component System) を採用。

- 高速なデータ局所性（文脈単位でチャンク実行）
- 型で文脈を分離（`Component<C>`）し、借用を安全に
- 文脈ローカルなリソース/時間/スケジューリング

## ドキュメント
- `crate/` 本リポジトリのクレートが入っている
- `crate/aubrey_common` 基本型など、単体では役に立たない小粒の機能をまとめるクレート
- `crate/aubrey_core` 本エンジンの核となるシステムが入っている
- `crate/aubrey_widget` GUIシステムを構築するためのシステム
- `crate/aubrey_window` winitでウィンドウを作成・イベント処理を行うシステム
- `crate/aubrey_editor` エディタのエントリポイント（現状: ウィンドウ表示のみ）

## Docs
- ECS design: `docs/ecs.md`
- TickComponent: `docs/components/tick.md`
- Scheduling and system order: `docs/scheduling.md`

## ドキュメント
- ECSの設計: `docs/ecs.md`
- TickComponentの仕様: `docs/components/tick.md`

## ドキュメント
- Entity / Component<C> / Context / Resource / System / Scheduler
- TweenComponent<C> / TickComponent<C> / Time<C>





