use std::time::Instant;

pub struct Timer{
    time: Instant,
    prev: u128,
}

impl Timer{
    pub fn new() -> Self{
        let time = Instant::now();
        let prev = time.elapsed().as_millis();
        Self{ time, prev }
    }

    pub fn elapsed(&self) -> u128{
        self.time.elapsed().as_millis() - self.prev
    }

    pub fn checkpoint(&mut self){
        self.prev = self.time.elapsed().as_millis();
    }
}
