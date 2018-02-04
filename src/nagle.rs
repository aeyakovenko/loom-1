struct Node {
    prev: *Signal,
    next: *Signal,
}

struct Dispatcher {
}

struct Signal {
    bn: Node,
    qn: Node,
    q: *Dispatcher,
    b: *Bus,
    set: bool,
    pub sig: u64,
}

struct Ctl {
    q: *Dispatcher,
}


impl Drop for Signal {
    fn drop(&mut self) {
        unsafe {
        }
    }
}

impl Dispatcher {
    pub fn new() -> Dispatch {
    }
    pub fn signal(h: u64) -> (Box<Signal>, Box<Ctl>) {
        return &Signal{prev:0, next:0, set:false, handler:h};
    }
    pub fn pop() -> &Signal {
    }
}

struct Bus {
    prev: *Signal,
    next: *Signal,
}

impl Bus {
    pub fn new() -> Bus {
    }
    pub fn add(&mut self, sig: &Signal) {
    }
    pub fn set(&mut self) {
    }
}
