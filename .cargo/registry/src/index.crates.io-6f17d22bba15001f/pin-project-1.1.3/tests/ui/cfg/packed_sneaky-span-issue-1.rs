use auxiliary_macro::hidden_repr;
use pin_project::pin_project;

#[pin_project]
#[hidden_repr(packed)] //~ ERROR may not be used on #[repr(packed)] types
struct S {
    #[cfg(not(any()))]
    #[pin]
    f: u32,
    #[cfg(any())]
    #[pin]
    f: u8,
}

fn main() {}
