pub struct IdCounter {
    value: u32
}

impl IdCounter {
    pub fn new(initial: u32) -> IdCounter {
        IdCounter {
            value: initial
        }
    }

    pub fn get_id(&mut self) -> u32 {
        let ret = self.value;

        if self.value == std::u32::MAX {
            self.value = 0;
        } else {
            self.value = self.value + 1;
        }

        ret
    }
}