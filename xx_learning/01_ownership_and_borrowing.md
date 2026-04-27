# 1. Ownership and borrowing

Ownership is the first Rust idea to get comfortable with: each value has one owner, moves happen by default, and borrowing lets code read or modify data without taking ownership.

This project shows all three patterns.

## Owned data inside domain entities

`Container` owns its fields:

**File:** `src/01_domain/container/entity.rs`

```rust
pub struct Container {
    id: ContainerId,
    name: String,
    image: String,
    state: ContainerState,
    status: String,
    created: DateTime<Utc>,
    ports: Vec<PortMapping>,
    networks: Vec<NetworkInfo>,
    mounts: Vec<MountInfo>,
    env_vars: Vec<String>,
}
```

The struct stores `String` and `Vec<T>` values directly. That means a `Container` fully owns its state.

## Borrowing through getters

Instead of cloning everything, getters often return borrowed references:

```rust
pub fn name(&self) -> &str {
    &self.name
}

pub fn ports(&self) -> &[PortMapping] {
    &self.ports
}
```

Why this matters:

- `&self` borrows the container
- `&str` borrows text from the owned `String`
- `&[PortMapping]` borrows a slice from the owned `Vec<PortMapping>`

That keeps reads cheap and avoids unnecessary allocation.

## Moving `self` in builder-style methods

The entity also uses a builder-like style:

```rust
pub fn with_mounts(mut self, mounts: Vec<MountInfo>) -> Self {
    self.mounts = mounts;
    self
}
```

This takes ownership of `self`, updates it, and returns it. That is why chaining works:

```rust
let container = Container::new(...)
    .with_mounts(vec![...])
    .with_env_vars(vec![...]);
```

Each call consumes the previous value and returns a new owned value.

## Borrowing collections in the presentation layer

Presenter code borrows a slice of DTOs rather than taking ownership:

**File:** `src/04_presentation/tui/container/presenter.rs`

```rust
pub fn filter_containers<'a>(
    containers: &'a [ContainerDto],
    filter: &str,
) -> Vec<&'a ContainerDto> {
```

This function:

- borrows the list with `&[ContainerDto]`
- borrows the filter text with `&str`
- returns borrowed references to matching items with `Vec<&ContainerDto>`

It is a good example of Rust saying: "I want to inspect these containers, not own them."

## A simple mental model

Use this rule while reading the code:

- `String`, `Vec<T>`, structs by value: ownership
- `&T`, `&str`, `&[T]`: borrowing
- `self` in a method parameter: the method consumes the value
- `&self`: the method reads from the value without taking it
- `&mut self`: the method mutates the value without taking it

## Where to look next in this repo

- `src/01_domain/container/entity.rs` for owned entities and borrowed getters
- `src/04_presentation/tui/container/presenter.rs` for borrowed filtering
- `src/04_presentation/tui/container/view.rs` for borrowed DTOs passed into rendering

Next: [control flow with `if` and `match`](./02_control_flow_if_and_match.md).
