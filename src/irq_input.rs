pub struct IrqInput(bool);

impl IrqInput {
    pub fn new() -> IrqInput {
        IrqInput(false)
    }

    pub fn is_asserted(&self) -> bool {
        self.0
    }

    pub fn assert(&mut self) {
        self.0 = true;
    }

    pub fn reset(&mut self) {
        self.0 = false;
    }
}
