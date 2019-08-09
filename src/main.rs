use std::any::TypeId;
use std::marker::PhantomData;
use std::mem::{ManuallyDrop, transmute};

/// An unconstructible-type, use it as type-information for the builder indicating a value has not
/// been set. This could be replaced with the type from the void-crate.
enum Unset {}

/// A helper-function to check if the type is `Unset`
fn is_unset<A: 'static>() -> bool {
    TypeId::of::<A>() == TypeId::of::<Unset>()
}

/// The item we construct in the end. We use types that free memory on drop to show the builder
/// does handle memory correctly.
#[derive(Debug)]
struct Item {
    a: String,
    b: Vec<i32>,
}

/// The builder, containing the fields that will be passed to the item and the types that are used
/// at compile-time to check if the fields are set. A generic type either is `Unset` or the actual
/// type of the field (`String` or `Vec<i32>` in this case). The fields are stored as
/// `ManuallyDrop` to bypass rusts destructor because they might be uninitialized.
struct ItemBuilder<A: 'static, B: 'static> {
    a: ManuallyDrop<String>,
    b: ManuallyDrop<Vec<i32>>,
    _a: PhantomData<A>,
    _b: PhantomData<B>,
}

impl ItemBuilder<Unset, Unset> {
    /// Construct a new builder, set fields to uninitialized and set types to `Unset`
    fn new() -> Self {
        unsafe {
            Self {
                a: std::mem::uninitialized(),
                b: std::mem::uninitialized(),
                _a: PhantomData,
                _b: PhantomData,
            }
        }
    }
}


impl<A, B> ItemBuilder<A, B> {
    /// Set a new value into the field and return the builder. That will also change the
    /// corresponding type-parameter to the fields type to indicate a value has been set. Since we
    /// can't construct a new object with a new type because of our custom destructor we simply
    /// cast it. The builder always has the same size and memory-layout regardless of
    /// type-parameters, so this will never be an issue (i guess).
    fn a(mut self, a: String) -> ItemBuilder<String, B> {
        // if we already set a value before, drop it
        if !is_unset::<A>() {
            unsafe { ManuallyDrop::drop(&mut self.a); }
        }

        self.a = ManuallyDrop::new(a);
        unsafe { transmute(self) }
    }

    /// Same as `b`
    fn b(mut self, b: Vec<i32>) -> ItemBuilder<A, Vec<i32>> {
        if !is_unset::<B>() {
            unsafe { ManuallyDrop::drop(&mut self.b); }
        }

        self.b = ManuallyDrop::new(b);
        unsafe { transmute(self) }
    }
}

/// Implementation for constructing an `Item`. This only can be done when no field is `Unset`,
/// meaning all fields have been set.
impl ItemBuilder<String, Vec<i32>> {
    /// Consume this builder and construct an item with the values set in the builder. Do some
    /// memory-magic to avoid problems.
    fn construct(self) -> Item {
        let (a, b) = unsafe {
            // get pointers to fields
            let s = &self.a as *const ManuallyDrop<String>;
            let v = &self.b as *const ManuallyDrop<Vec<i32>>;

            // forget the builder, otherwise this would destroy the fields as soon as the builder
            // gets dropped
            std::mem::forget(self);
            // read the pointers to reclaim ownership of values we "forgot"
            (std::ptr::read(s), std::ptr::read(v))
        };

        Item {
            // remove the `ManuallyDrop` as we can be sure that the memory-locations are
            // initialized thanks to the type-information
            a: ManuallyDrop::into_inner(a),
            b: ManuallyDrop::into_inner(b),
        }
    }
}

/// Since we can't let rust handle destruction because fields might not be initialized yet we have
/// to provide our own destructor. We simply use the type-information of the generics to check
/// which field is initialized. Again, this is generated at compile-time and will result in an
/// destructor rust couldn't do better.
impl<A, B> Drop for ItemBuilder<A, B> {
    fn drop(&mut self) {
        if !is_unset::<A>() {
            unsafe { ManuallyDrop::drop(&mut self.a); }
        }
        if !is_unset::<B>() {
            unsafe { ManuallyDrop::drop(&mut self.b); }
        }
    }
}

fn main() {
    let builder = ItemBuilder::new();

    let with_field = builder.a("incomplete".into());
    let complete = with_field.b(vec![]);

    println!("{:?}", complete.construct());

    // Try uncommenting this code and see it won't work. The builder will have the type
    // `ItemBuilder<Unset, Unset>` indicating both fields have not been set yet.
    // println!("{:?}", ItemBuilder::new().construct());

    // behold, no memory-errors. although memory-leaks are not checked for, you have to believe me
    // on this one or test for yourself.
    drop(ItemBuilder::new());
    drop(ItemBuilder::new().a("str".into()));
    drop(ItemBuilder::new().b(vec![1, 2, 3, 4]));

    drop(ItemBuilder::new().a("str".into()).a("str2".into()));
    drop(ItemBuilder::new().b(vec![1, 2, 3, 4]).b(vec![5, 6, 7, 8, 9, 10]));
    drop(ItemBuilder::new().a("str".into()).b(vec![5, 6, 7, 8, 9, 10]).construct());
}

