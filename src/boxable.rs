pub trait Boxable {
    fn wrap(self) -> Box<Self> where Self: Sized;
}

impl<T> Boxable for T {
    fn wrap(self) -> Box<Self> {
        Box::new(self)
    }
}