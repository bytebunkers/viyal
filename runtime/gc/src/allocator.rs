#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GcHandle(pub usize);

pub struct GcObject<T> {
    pub value: T,
    pub marked: bool,
}

pub struct GcAllocator<T> {
    objects: Vec<Option<GcObject<T>>>,
    free_slots: Vec<usize>,
}

impl<T> GcAllocator<T> {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            free_slots: Vec::new(),
        }
    }

    pub fn allocate(&mut self, value: T) -> GcHandle {
        let object = GcObject {
            value,
            marked: false,
        };

        if let Some(index) = self.free_slots.pop() {
            self.objects[index] = Some(object);
            GcHandle(index)
        } else {
            let index = self.objects.len();
            self.objects.push(Some(object));
            GcHandle(index)
        }
    }

    pub fn get(&self, handle: GcHandle) -> Option<&T> {
        self.objects.get(handle.0)?.as_ref().map(|obj| &obj.value)
    }

    pub fn get_mut(&mut self, handle: GcHandle) -> Option<&mut T> {
        self.objects.get_mut(handle.0)?.as_mut().map(|obj| &mut obj.value)
    }

    pub fn mark(&mut self, handle: GcHandle) {
        if let Some(Some(obj)) = self.objects.get_mut(handle.0) {
            obj.marked = true;
        }
    }

    pub fn sweep(&mut self) -> usize {
        let mut swept_count = 0;
        for i in 0..self.objects.len() {
            if let Some(obj) = &mut self.objects[i] {
                if !obj.marked {
                    // It's garbage, free it
                    self.objects[i] = None;
                    self.free_slots.push(i);
                    swept_count += 1;
                } else {
                    // Reset mark for next cycle
                    obj.marked = false;
                }
            }
        }
        swept_count
    }
}
