# Migration guide

This document helps with migration to newer versions of `tskit-rust`.
This document will not be exhaustive.
Rather, it will focus on highlights or "must know" items.

Regarding breaking changes:

* This document will discuss breaking changes requiring change in client code.
* Other changes listed in the change log as breaking are considered "technically breaking
  but unlikely to affect anyone in practice". Please file an issue if we are wrong!

Acronyms used:

* UB = undefined behavior.

## v0.11.0

### Bug fixes

* Issue [363](https://github.com/tskit-dev/tskit-rust/issues/363), which was introduced in PR [300](https://github.com/tskit-dev/tskit-rust/pull/300).
  The bug resulted in UB when advancing trees for release builds.

### Breaking changes

#### New error variant.

The error enum now includes `LibraryError`.
Any code previously matching on errors will now break because the match is no longer exhaustive.

#### Row getters

This change occurred over several PR.

All "getter" functions for table rows now return [`Option`](https://doc.rust-lang.org/std/option/enum.Option.html) instead of [`Result`](https://doc.rust-lang.org/std/result/enum.Result.html) when a row index is out of bounds.
These functions now return `None` for the out of bounds case.
This behavior change brings the API in line with other rust APIs.

To update:

* If `None` should represent an error for your use case, use [`Option::ok_or_else`](https://doc.rust-lang.org/std/option/enum.Option.html#method.ok_or_else).
  In the case of a previous `Result<Option<_>, _>` that is now an `Option<Result<_, _>>`, use [`transpose`](https://doc.rust-lang.org/std/option/enum.Option.html#method.transpose).

#### Lifetime annotations of table types

PR [#373](https://github.com/tskit-dev/tskit-rust/pull/373).

Previously, a non-owning view of an edge table had a type `EdgeTable<'a>`.
Those lifetimes are no longer necessary, which is a big design win.

Two traits, `TableAccess` and `NodeListRowGenerator`, have been removed.
The `TableCollection` and `TreeSequence` types now contain that functionality as
[`Deref`](https://doc.rust-lang.org/std/ops/trait.Deref.html) targets.

Removing these traits implies:

* Their removal from the crate prelude.
* They can no longer be used as trait bounds.

Some views of a node table that required mutability broke due to these changes.
A new function in the `Deref` target is `nodes_mut(&mut self)` addresses this breakage.
