#[derive(Debug)]
pub struct Windowed<I> {
    window_size: usize,
    source: I,
}

impl<I> Windowed<I> {
    pub fn new(window_size: usize, source: I) -> Self {
        Windowed { window_size, source }
    }
}

impl<T, I> Iterator for Windowed<I>
where
    I: Iterator<Item = T>,
{
    type Item = Vec<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let buffer: Vec<T> = self.source.by_ref().take(self.window_size).collect();

        if buffer.is_empty() { None } else { Some(buffer) }
    }
}
