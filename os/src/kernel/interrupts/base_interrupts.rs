use crate::devices::mouse::MouseISR;
use crate::kernel::interrupts::intdispatcher::{INT_VECTORS, InterruptVector};
use crate::kernel::interrupts::isr::ISR;
use alloc::boxed::Box;
use alloc::format;

pub fn init_base_interrupts() {
    let mut int_vector_lock = INT_VECTORS.lock();
    int_vector_lock.register(
        InterruptVector::DivisionByZero,
        Box::new(DivisionByZeroISR {}),
    );
    int_vector_lock.register(
        InterruptVector::GeneralProtectionFault,
        Box::new(GeneralProtectionFaultISR {}),
    );
    int_vector_lock.register(
        InterruptVector::PageFault,
        Box::new(PageFaultISR {}),
    );
}

pub struct DivisionByZeroISR {}

impl ISR for DivisionByZeroISR {
    fn trigger(&self) {
        panic!("{:?}", InterruptVector::DivisionByZero);
    }
}

pub struct GeneralProtectionFaultISR {}

impl ISR for GeneralProtectionFaultISR {
    fn trigger(&self) {
        panic!("{:?}", InterruptVector::GeneralProtectionFault);
    }
}
pub struct PageFaultISR {}

impl ISR for PageFaultISR {
    fn trigger(&self) {
        panic!("{:?}", InterruptVector::PageFault);
    }
}
