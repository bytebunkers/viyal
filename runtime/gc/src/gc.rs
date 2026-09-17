

pub trait Trace {
    fn trace(&self, gc: &mut Gc);
}

pub struct Gc {
    // Basic root tracking for MVP
}
