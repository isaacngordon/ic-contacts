pub struct Counter {
    count: u64,
}

impl Counter {
    pub fn new() -> Counter {
        Counter { count: 0 }
    }

    pub fn increment(&mut self) -> u64 {
        self.count += 1;
        self.count
    }

    pub fn decrement(&mut self) -> u64 {
        self.count -= 1;
        self.count
    }

    pub fn get_count(&self) -> u64 {
        self.count
    }
}

pub struct IncrementCounter {
    counter: Counter,
}

impl IncrementCounter {
    pub fn new() -> IncrementCounter {
        IncrementCounter { counter: Counter::new() }
    }

    pub fn increment(&mut self) -> u64 {
        self.counter.increment()
    }

    pub fn get_count(&self) -> u64 {
        self.counter.get_count()
    }
}

pub struct DecrementCounter {
    counter: Counter,
}

impl DecrementCounter {
    pub fn new() -> DecrementCounter {
        DecrementCounter { counter: Counter::new() }
    }

    pub fn decrement(&mut self) -> u64 {
        self.counter.decrement()
    }

    pub fn get_count(&self) -> u64 {
        self.counter.get_count()
    }
}
