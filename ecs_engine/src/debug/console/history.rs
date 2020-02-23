pub struct History<T> {
    hist: Vec<T>,
    cursor: usize,
    cap: usize,
}

#[derive(Copy, Clone)]
pub enum Direction {
    To_Older,
    To_Newer,
}

impl<T> History<T> {
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            hist: Vec::with_capacity(cap),
            cursor: 0,
            cap,
        }
    }

    pub fn push(&mut self, elem: T) {
        self.cursor = self.cursor.min(self.hist.len());

        if self.hist.len() == self.cap {
            self.hist.remove(0);
        }

        self.hist.push(elem);
        self.cursor = self.hist.len();
    }

    pub fn is_cursor_past_end(&self) -> bool {
        self.cursor == self.hist.len()
    }

    pub fn move_and_read(&mut self, dir: Direction) -> Option<&T> {
        match dir {
            Direction::To_Older => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
                self.hist.get(self.cursor)
            }
            Direction::To_Newer => {
                if !self.hist.is_empty() {
                    if self.cursor < self.hist.len() {
                        self.cursor += 1;
                    } else {
                        return None;
                    }
                }
                self.hist.get(self.cursor)
            }
        }
    }

    pub fn iter(&self) -> History_Iterator<'_, T> {
        self.into_iter()
    }
}

pub struct History_Iterator<'a, T> {
    hist: &'a History<T>,
    cur: usize,
}

impl<'a, T> Iterator for History_Iterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let cur = self.cur;
        self.cur += 1;
        self.hist.hist.get(cur)
    }
}

impl<'a, T> DoubleEndedIterator for History_Iterator<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let cur = self.cur;
        self.cur += 1;
        if cur >= self.hist.hist.len() {
            None
        } else {
            self.hist.hist.get(self.hist.hist.len() - 1 - cur)
        }
    }
}

impl<'a, T> IntoIterator for &'a History<T> {
    type Item = &'a T;
    type IntoIter = History_Iterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter { hist: self, cur: 0 }
    }
}
