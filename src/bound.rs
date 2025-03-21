pub struct Bound<T> {
    pub width: T,
    pub height: T,
}

impl<T> Bound<T> {
    pub fn new(width: T, height: T) -> Self {
        Self { width, height }
    }
}
