#![allow(unused)]
#![allow(clippy::all)]

extern crate alloc;

use {
    alloc::{rc::Rc, sync::Arc},
    app_macros::*,
    core::{
        cell::RefCell, error::Error, ffi::*, marker::PhantomData, num::NonZero,
        ptr::NonNull, str::FromStr
    },
    std::{
        collections::{HashMap, HashSet},
        ffi::CString,
        time::Instant
    }
};

#[derive(Debug, Default, ExtendFromIter)]
struct Foo<'a, 'b: 'a, T>
where
    T: AsRef<str> + Default
{
    a: std::string::String,
    b: Option<Box<Option<NonZero<c_uint>>>>,
    c: Arc<bool>,
    d: Option<char>,
    e: Option<Box<Rc<RefCell<f32>>>>,
    f: Option<&'b str>,
    g: Vec<&'a str>,
    h: Option<alloc::boxed::Box<str>>,
    #[parse]
    l: Option<Box<Lang>>,
    m: Option<HashSet<Option<NonZero<i32>>>>,
    n: Option<Rc<RefCell<HashMap<&'a str, Option<&'a str>>>>>,
    o: HashMap<&'a str, Option<String>>,
    p: Option<NonNull<c_void>>,
    r: Box<CStr>,
    s: CString,
    bar: Arc<self::Bar<'b, T>>,
    zar: Zar,
    _phantom: PhantomData<&'b T>
}

#[derive(Debug, Default, ExtendFromIter)]
struct Bar<'b, T>
where
    T: Default
{
    x: &'b str,
    y: RefCell<c_float>,
    z: RefCell<Zar>,
    _phantom: PhantomData<&'b T>
}

#[derive(Debug, Default, ExtendFromIter)]
struct Zar {
    a: Option<i32>,
    b: Option<Box<Vec<i32>>>
}

#[derive(Debug, Default, PartialEq)]
enum Lang {
    #[default]
    Ru,
    En
}

impl FromStr for Lang {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ru" => Ok(Self::Ru),
            "en" => Ok(Self::En),
            _ => Err("Unsupported Lang value: ".to_string() + s)
        }
    }
}

#[test]
fn test_extend() -> Result<(), Box<dyn Error>> {
    let values1 = gen_values1();
    let values2 = gen_values2();
    let mut foo = Foo::<String>::default();
    foo.h = Some("Predefined value".into());
    foo.zar.b = Some(vec![1, 2, 3].into());

    let max_iters = 10000;
    let t = Instant::now();
    for _ in 0..max_iters {
        foo.g = vec!["zzz"];
        foo.extend(values1.clone());
        foo.extend(values2.clone());
    }
    let time = t.elapsed();
    dbg!(&foo);
    dbg!(time, max_iters);

    assert_eq!(foo.a, "Hello".to_owned());
    assert_eq!(foo.b, Some(Box::new(None)));
    assert_eq!(foo.c, true.into());
    assert_eq!(foo.d, Some('X'));
    assert_eq!(foo.e, Box::new(Rc::new(RefCell::new(1.23))).into());
    assert_eq!(foo.f, "World".into());
    assert_eq!(foo.g, vec!["zzz", "a", "b", "c"]);
    assert_eq!(foo.h, Some("Predefined value".into()));
    assert_eq!(foo.l, Some(Lang::En.into()));
    assert_eq!(
        foo.m,
        Some(HashSet::from_iter([
            NonZero::new(111),
            NonZero::new(-1111),
            None
        ]))
    );
    assert_eq!(
        foo.n,
        Some(Rc::new(RefCell::new(HashMap::from_iter([
            ("foo", " Foo ".into()),
            ("bar", " Bar ".into())
        ]))))
    );
    assert_eq!(
        foo.o,
        HashMap::from_iter([
            ("fooooo", Some("Foooooo".into())),
            ("baaaar", Some("Baaaaar".into()))
        ])
    );
    assert_eq!(
        foo.p.map(|p| unsafe {
            CStr::from_ptr(p.as_ptr() as *const i8).to_str().unwrap()
        }),
        "C void".into()
    );
    assert_eq!(foo.r.to_str().unwrap(), "C str");
    assert_eq!(foo.s.to_str().unwrap(), "C string");

    assert_eq!(foo.bar.x, "This is Bar");
    assert_eq!(foo.bar.y, 9.999.into());
    assert_eq!(foo.bar.z.borrow().a, Some(-1111));
    assert_eq!(foo.bar.z.borrow().b, Some(vec![-123, 0, 123].into()));
    assert_eq!(foo.zar.a, Some(-333));
    assert_eq!(foo.zar.b, Some(vec![1, 2, 3].into()));

    Ok(())
}

fn gen_values1() -> Vec<(&'static str, Option<&'static str>)> {
    vec![
        ("a", "Hello".into()),
        ("b", "0".into()),
        ("c", "true".into()),
        ("d", "X".into()),
        ("o.fooooo", "Foooooo".into()),
        ("p", "C void".into()),
        ("r", "C str".into()),
        ("s", "C string".into()),
        ("bar.x", "This is Bar".into()),
        ("bar.z.a", "-1111".into()),
        ("zar.a", "-333".into()),
    ]
}

fn gen_values2() -> Vec<(&'static str, Option<&'static str>)> {
    vec![
        ("e", "1.23".into()),
        ("f", "World".into()),
        ("g", "   a ,   b ,    c   ".into()),
        ("h", None),
        ("l", "en".into()),
        ("m", "  -1111, 0, 111  ".into()),
        ("n.foo", " Foo ".into()),
        ("n.bar", " Bar ".into()),
        ("o.baaaar", "Baaaaar".into()),
        ("bar.y", "9.999".into()),
        ("bar.z.b", "  -123, 0, 123 ".into()),
    ]
}
