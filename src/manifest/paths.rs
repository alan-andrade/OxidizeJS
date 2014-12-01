static CORES: uint = 4;

pub struct Paths<'a, T: 'static> {
    collection: &'a [T],
    index: uint,
    per_core: uint
}

impl<'a, T> Paths<'a, T> {
    pub fn new(col: &'a [T]) -> Paths<'a, T> {
        Paths {
            per_core: (col.len() as f32 / CORES as f32).ceil() as uint,
            collection: col,
            index: 0
        }
    }
}

impl<'a, T> Iterator<&'a [T]> for Paths<'a, T> {
    fn next(&mut self) -> Option<&'a [T]> {
        if self.index < self.collection.len() && self.per_core > 0 {
            let current = self.index;
            self.index  = self.index + self.per_core;
            Some(self.collection.slice(current, self.index))
        } else {
            None
        }
    }
}

