#![allow(unused)]

use {
    app_macros::StructFields,
    core::{
        any::{Any, TypeId, type_name},
        marker::PhantomData,
        num::NonZero,
        ops::Deref
    },
    impls::impls,
    std::fmt::Debug
};

trait TraitA: Debug {}
trait TraitB: Debug {}
trait TraitC: Debug {}

#[derive(Debug)]
struct Foo;

#[derive(Debug)]
struct Bar;

#[derive(Debug, StructFields)]
struct Zar {
    foo: Foo,
    bar: Bar,
    kok: Option<NonZero<u32>>
}

#[derive(Debug)]
struct ZarParent {
    zar: Zar,
    bob: String
}

impl TraitA for Foo {}
impl TraitB for Foo {}
impl TraitA for Bar {}

impl Deref for ZarParent {
    type Target = Zar;
    fn deref(&self) -> &Self::Target {
        &self.zar
    }
}

impl AsRef<ZarParent> for ZarParent {
    fn as_ref(&self) -> &ZarParent {
        self
    }
}

impl AsRef<Zar> for ZarParent {
    fn as_ref(&self) -> &Zar {
        &self.zar
    }
}

impl<'a> From<&'a ZarParent> for Box<[&'a dyn TraitA]> {
    fn from(value: &'a ZarParent) -> Self {
        Box::new([&value.zar.foo])
    }
}

impl AsRef<Zar> for Zar {
    fn as_ref(&self) -> &Zar {
        self
    }
}

impl<'a, T: ?Sized + Deref<Target = Zar>> From<&'a T> for &'a Zar {
    fn from(value: &'a T) -> Self {
        value.deref()
    }
}

impl<'a> From<&'a Zar> for Box<[&'a dyn TraitA]> {
    fn from(value: &'a Zar) -> Self {
        Box::new([&value.foo, &value.bar])
    }
}

impl<'a> From<&'a Zar> for Box<[&'a dyn TraitB]> {
    fn from(value: &'a Zar) -> Self {
        Box::new([&value.foo])
    }
}

impl<'a> From<&'a Zar> for Box<[&'a dyn TraitC]> {
    fn from(value: &'a Zar) -> Self {
        Box::new([])
    }
}

#[test]
fn test_struct_fields() {
    let zar = Zar { foo: Foo, bar: Bar, kok: NonZero::new(0) };
    let zar_parent = ZarParent { zar, bob: "Bob".into() };

    assert_eq!(["foo", "bar", "kok"], zar_parent.field_names());
    assert_eq!(
        [
            ("foo", "Foo"),
            ("bar", "Bar"),
            ("kok", "Option < NonZero < u32 > >")
        ],
        zar_parent.field_types()
    );

    dbg!(
        <Box<[&dyn TraitA]>>::from(&zar_parent as &ZarParent),
        <Box<[&dyn TraitB]>>::from(&zar_parent as &Zar),
        <Box<[&dyn TraitC]>>::from(&zar_parent as &Zar),
    );
}
