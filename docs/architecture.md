# Architecture and Design Policies

## Layered Architecture

This application is implemented as a [Cargo Workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html)
that brings together each of the following layers as a crate.

![](./figs/layered_architecture.png)

* **Controller**: A layer that serves as an interface to the outside world
    * Routing
    * Validation of the request
    * Calling the `usecase` layer
* **UseCase**: A layer that represents the procedural flow of the application.
    * Manipulate the Domain layer 
    * Define and create the `Request` / `Response` struct
    * Transaction management
    * In this layer, we should focus on expressing the "flow" as much as possible, and the "function" should be expressed in `Domain` and `Infrastructure`.
* **Domain**: A layer that consolidates the concepts and logic that appear in the business
* **Infrastructure**: A layer that represents the logic for interfacing with middleware and external APIs
    * Calling middleware/external APIs
    * Transforms responses into domain models


## Data Flow
![](./figs/data_flow.png)

## `crate` design
### component list
| crate            | rust component         | description |
| ---------------- | ---------------------- | --- |
| `controller`     | `fn controller`        | Functions to represent each endpoint of the RESTful API |
| `usecase`        | `trait UseCase`        | Represents application procedures <br> Transaction management <br> Perform conversion between `Request`/`Response` and `DomainModel` |
|                  | `struct Request`       | Data object to be passed from `controller` to `UseCase` <br> Deserializable from json <br> Request validation should be done here. |
|                  | `struct Response`      | Data object to be passed from `UseCase` to `controller` <br> Serializable to json |
| `domain`         | `struct DomainModel`   | Describes behavior as a Model (i.e., operations that are completed with its own data) |
|                  | `struct DomainService` | Represents of the interaction between different `DomainModel` |
|                  | `trait Repository`     | Interfaces for data access logic in `infrastructure` <br> Converts the retrieved data to `DomainModel` and returns it |
|                  | `trait Api`            | Interfaces for access logic to external APIs, etc. <br> Converts the retrieved data to `DomainModel` and returns it |
| `infrastructure` | `struct Repository`    | Data access logic to DB, etc. <br> `impl domain::Repository` |
|                  | `struct Api`           | Access logic to external APIs, etc. <br> `impl domain::Api` |

### Access Rule
| ↓Caller \ Callee→    | `controller` | `UseCase` | `Request`/`Response` | `DomainModel` | `DomainService` | `trait Repository` | `trait Api` |
| -------------------  | --- | --- | --- | --- | --- | --- | --- |
| `controller`         | × | ⭕️ | ⭕️ | × | × | × | × |
| `UseCase`            | × | × | ⭕️ | ⭕️ | ⭕️ | ⭕️ | ⭕️ |
| `Request`/`Response` | × | × | × | ⭕️ | × | × | × |
| `DomainModel`        | × | × | × | ⭕️ （Owned models only） | × | × | × |
| `DomainService`      | × | × | × | ⭕️ | × | × | × |
| `struct Repository`  | × | × | × | ⭕️ | × | ⭕️ | × |
| `struct Api`         | × | × | × | ⭕️ | × | × | ⭕️ |
