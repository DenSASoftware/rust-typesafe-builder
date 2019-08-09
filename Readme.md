# Builder verified at compile-time in rust

This is a little challenge I gave myself, I wanted to see if there's a way to create a builder-object in rust that checks at compile-time if all fields have been set. Most other builders in rust check this at runtime, that didn't feel very rust-y given that rust offers zero-cost-abstraction that get resolved at compile-time. If you want to use something like what I described check out the [typed-builder-crate](https://crates.io/crates/typed-builder). It creates a builder based on the object you pass it.

## The plan

The idea is simple: use the rust-generics to store which fields have been set. My first approach stored the types of the fields, using `()` as the type for a field that has not been set. A builder for an object with 2 fields would start out as `Builder<(), ()>` and after setting a field would become `Builder<String, ()>`, assuming the field is a string. And constructing the final object would only be possible for an `Builder<String, AnotherType>`. While this approach does work fine, the builder does look like this:  
```rust
struct Builder<A, B> {
	field_a: A,
	field_b: B,
}
```
This means the different builder-types will contain different types and therefore have different memory-layouts. Rusts compiler will surely figure out where to put stuff to avoid moving objects more than necessary, I am not 100% certain. That's what lead me to my second approach.

**What if we allocated the memory beforehand and used the generics to store which fields have been initialized?**

That lead to my second approach on the builder:  
```rust
struct Builder<A, B> {
	field_a: std::mem::MaybeUninitialized<String>,
	field_b: std::mem::MaybeUninitialized<Vec<i32>>,
	_a: std::marker::PhantomData<A>,
	_b: std::marker::PhantomData<B>,
}
```
Here `field_a` contains the data for the first field that might not be initialized yet, while `A` contains information about whether `field_a` is set yet. For this we use empty enums (see the [void-crate](https://crates.io/crates/void)), `Unset` and `Set` denote the respective state. The builder starts out at `Builder<Unset, Unset>` and setting a value returns a new builder with one of the generic-types set to `Set`. Similar to the first approach, the final object can only be constructed from a `Builder<Set, Set>`. Most of my time spent with this approach was dedicated to tearing down rusts memory-safety guarantees since the builder has to (not) drop different things based on its generic-type-information.

In the end I'm happy with the result, as I think it's similar on how the typed-builder-crate operates. Maybe this code gives someone insight on how a builder that's checked during compile-time can be done. You can also look at the aforementioned crate, but I beliebe reading through code dealing with proc-macro-stuff can be not easy to understand.

