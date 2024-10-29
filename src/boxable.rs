pub trait Boxable {
    fn into_box(self) -> Box<Self> where Self: Sized;
}

impl<T> Boxable for T {
    fn into_box(self) -> Box<Self> {
        Box::new(self)
    }
}