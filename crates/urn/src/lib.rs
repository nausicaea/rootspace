#[derive(Debug)]
pub struct Urn<T> {
    max_token: usize,
    free_tokens: Vec<usize>,
    _t: std::marker::PhantomData<T>,
}

impl<T> Urn<T> {
    fn take_id(&mut self) -> usize {
        if let Some(t) = self.free_tokens.pop() {
            t
        } else {
            let tmp = self.max_token;
            self.max_token += 1;
            tmp
        }
    }
}

impl<T: From<usize>> Urn<T> {
    pub fn take(&mut self) -> T {
        let token = self.take_id();
        T::from(token)
    }
}

impl<T> Default for Urn<T> {
    fn default() -> Self {
        Urn {
            max_token: 0,
            free_tokens: Vec::default(),
            _t: std::marker::PhantomData,
        }
    }
}
