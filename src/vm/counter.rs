#[derive(Clone, PartialEq, Debug)]
pub struct IndexedCounter {
    pub(crate) index: i32,
    step: i32,
    end: i32
}

impl IndexedCounter {

    pub fn new(index: i32, step: i32, end: i32) -> Self {
        IndexedCounter { index, step, end }
    }

    pub fn increment(&mut self) {
        self.index += self.step;
    }

    pub fn has_next(&self) -> bool {
        self.index < self.end
    }

    pub fn is_done(&self) -> bool {
        self.index > self.end
    }
}