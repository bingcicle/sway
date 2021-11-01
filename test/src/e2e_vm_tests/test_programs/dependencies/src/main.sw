script;
// This tests importing other files.

dep a_dependency;
dep nested_dependency/bar/bar;

use foo::Foo;
use ::foo::bar::Bar;
use ::foo::bar::double_bar::DoubleBar;

fn main() -> bool {
    let foo = Foo {
        foo: "foo",
    };
    let db = DoubleBar {
        a: 5u32,
    };
    let bar = Bar {
        a: 5u32,
    };
    false
}
