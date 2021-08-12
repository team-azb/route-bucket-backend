# 設計方針

## レイヤードアーキテクチャ

本アプリケーションは、以下に示すそれぞれの層をcrateとし、
それらを取りまとめる[Cargo Workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html)として実装されている。

![](./figs/layered_architecture.png)

* **Controller**: 外部へのインターフェスとなる層
  * ルーティング
  * リクエストのバリデーション
  * UseCaseレイヤーの呼び出し
* **UseCase**: アプリケーションの手続きを表現する層
  * Domainレイヤの操作
  * `Request` / `Response` 構造体の定義、生成
  * トランザクション管理
* **Domain**: 業務上登場する概念やロジックをまとめた層
* **Infrastructure**: ミドルウェアや外部APIとの繋ぎこみ
  * ミドルウェア/外部APIの呼び出し
  * 呼び出し結果のドメインモデルへの変換


## データの流れ
![](./figs/data_flow.png)


## `crate`, `struct`設計
### `struct`一覧
| crate            | struct (or trait)   | 説明 |
| ---------------- | ------------------- | --- |
| `controller`     | `Controller`        | RESTful APIのcontroller |
| `usecase`        | `UseCase`           | アプリケーションの手続きを表現 <br> トランザクション管理 <br> `Request`, `Response`と`DomainModel`との変換を行う |
|                  | `Request`           | `Controller`から`UseCase`へ渡すデータオブジェクト <br> jsonから`Deserialize`される <br> validationはここで行うべき |
|                  | `Response`          | `UseCase`から`Controller`へ渡すデータオブジェクト <br> jsonへ`Serialize`される |
| `domain`         | `DomainModel`       | Modelとしての振る舞い（＝自身のデータで完結する操作）を記述 |
|                  | `DomainService`     | 異なる`DomainModel`同士の相互作用を表現 |
|                  | `trait Repository`  | `infrastructure`でのデータアクセスロジックのインターフェス <br> 取得したデータを`DomainModel`へと変換して返す |
|                  | `trait Api`         | 外部APIなどへのアクセスロジックのインターフェス <br> 取得したデータを`DomainModel`へと変換して返す |
| `infrastructure` | `Repository`        | DBなどへのデータアクセスロジック <br> `domain::Repository`を実装 |
|                  | `Api`               | 外部APIなどへのアクセスロジック <br> `domain::Api`を実装 |

### 呼び出し規則
| ↓呼出元 \ 呼出先→      | `Controller` | `UseCase` | `Request`/`Response` | `DomainModel` | `DomainService` | `trait Repository` | `trait Api` |
| -------------------  | --- | --- | --- | --- | --- | --- | --- |
| `Controller`         | × | ⭕️ | ⭕️ | × | × | × | × |
| `UseCase`            | × | × | ⭕️ | ⭕️ | ⭕️ | ⭕️ | ⭕️ |
| `Request`/`Response` | × | × | × | ⭕️ | × | × | × |
| `DomainModel`        | × | × | × | ⭕️ （所有するモデルのみ） | × | × | × |
| `DomainService`      | × | × | × | ⭕️ | × | × | × |
| `Repository`         | × | × | × | ⭕️ | × | ⭕️ | × |
| `Api`                | × | × | × | ⭕️ | × | × | ⭕️ |
