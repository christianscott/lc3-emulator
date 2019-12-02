#[derive(Debug)]
pub(crate) struct Reader<T> {
    items: Vec<T>,
    is_newline: fn(T) -> bool,
    pub(crate) offset: usize,
    pub(crate) item_in_line: usize,
    pub(crate) line: usize,
}

impl<T: Clone> Reader<T> {
    pub(crate) fn from(items: Vec<T>, is_newline: fn(T) -> bool) -> Self {
        Reader {
            items,
            is_newline,
            offset: 0,
            item_in_line: 0,
            line: 0,
        }
    }

    pub(crate) fn reset(&mut self) {
        self.offset = 0;
        self.item_in_line = 0;
        self.line = 0;
    }

    pub(crate) fn get(&self, index: usize) -> Option<T> {
        self.items.get(index).map(ToOwned::to_owned)
    }

    pub(crate) fn peek(&self) -> Option<T> {
        self.get(self.offset)
    }

    pub(crate) fn next(&mut self) -> Option<T> {
        let c = self.peek();

        if let Some(c) = c {
            if (self.is_newline)(c.clone()) {
                self.line += 1;
                self.item_in_line = 0;
            } else {
                self.item_in_line += 1;
            }

            self.offset += 1;
            Some(c)
        } else {
            None
        }
    }

    pub(crate) fn skip_while<F>(&mut self, predicate: F)
    where
        F: Fn(T) -> bool + Copy,
    {
        while self.peek().map_or(false, predicate) {
            self.next();
        }
    }

    pub(crate) fn take_while<F>(&mut self, predicate: F) -> Vec<T>
    where
        F: Fn(T) -> bool + Copy,
    {
        let mut chars = Vec::new();
        while self.peek().map_or(false, predicate) {
            match self.next() {
                Some(c) => chars.push(c),
                None => break,
            }
        }
        chars.iter().map(ToOwned::to_owned).collect()
    }
}
