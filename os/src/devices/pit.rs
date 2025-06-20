/* ╔═════════════════════════════════════════════════════════════════════════╗
   ║ Module: pit                                                             ║
   ╟─────────────────────────────────────────────────────────────────────────╢
   ║ Descr.: Programmable Interval Timer.                                    ║
   ╟─────────────────────────────────────────────────────────────────────────╢
   ║ Author:  Michael Schoettner, HHU, 15.6.2023                             ║
   ╚═════════════════════════════════════════════════════════════════════════╝
*/
use crate::devices::cga;
use crate::devices::cga::{CGA, Color};
use crate::kernel::cpu;
use crate::kernel::cpu::IoPort;
use crate::kernel::interrupts::intdispatcher::{INT_VECTORS, InterruptVector};
use crate::kernel::interrupts::isr::ISR;
use crate::kernel::interrupts::pic::Irq;
use crate::kernel::interrupts::{intdispatcher, pic};
use crate::kernel::threads::scheduler::get_scheduler;
use alloc::boxed::Box;
use core::arch::asm;
use core::sync::atomic::AtomicUsize;
use spin::Once;

// A5.1: Programmable Interval Timer (PIT)
// Der PIT wird ab sofort verwendet, um eine Systemzeit sowie ein erzwungenes Umschalten zwischen Threads zu realisieren. Die Systemzeit wird in der Variable SYSTEM_TIME (in pit.rs) gespeichert und diese soll bei jedem Interrupt für den PIT inkrementiert werden. Verwenden Sie hierfür im PIT den Zähler 0 und Modus 3 und laden Sie den Zähler mit einem passenden Wert, sodass der PIT jede Millisekunde ein Interrupt ausgelöst. Jeder Interrupt verursacht also eine Inkrementierung und entspricht einem Tick (1ms). Somit zeigt SYSTEM_TIME an, wie viele Ticks seit dem Beginn der Zeiterfassung vergangen sind.
//
// Im Interrupt-Handler des PITs soll die Systemzeit in Form eines rotierenden Zeichens (engl. spinner) an einer festen Stelle dargestellt werden. Verwenden Sie hierfür beispielsweise die rechte obere Ecke und folgende Zeichen: | / - \ (vorgegeben in SPINNER_CHARS), wobei das Zeichen in einem festen Intervall (z.B. alle 250ms) gewechselt werden soll. Hierzu muss in trigger() die CGA Instanz gelockt werden. Sollte das Lock gerade nicht verfügbar sein, würde dies zu einem Deadlock führen, da wir nie aus dem Interrupt Handler zurückkehren würden. Verwenden Sie try_lock() um dies zu vermeiden. Sollte das Lock nicht verfügbar sein, wird das Zeichen einfach nicht ausgegeben. Früher oder später wird das Lock mal frei sein und das Zeichen aktualisiert werden.
//
// Die Funktion plugin() soll den den PIT mit Hilfe von TIMER.call_once(|| { ... }) initialisieren, das Interrupt Intervall setzen und ihn in intdispatcher.rs anmelden. Außerdem sollen die Timer Interrupts im PIC zugelassen werden. Rufen Sie plugin() in startup.rs auf, um den Timer zu starten.
//
// In folgenden Dateien muss Code implementiert werden: devices/pit.rs und startup.rs.

// Ports
const PORT_CTRL: u16 = 0x43;
const PORT_DATA0: u16 = 0x40;

const TIMER_FREQ: usize = 1193182; // Timer frequency in Hz
const NANOSECONDS_PER_TICK: usize = 1_000_000_000 / TIMER_FREQ; // Nanoseconds per timer tick

/// Global timer instance.
/// Not accessible from outside the module.
/// To get the current system time, use `get_system_time()`.
static TIMER: Once<Timer> = Once::new();

/// Global system time in milliseconds.
static SYSTEM_TIME: AtomicUsize = AtomicUsize::new(0);

/// Characters used for the spinner animation.
static SPINNER_CHARS: &[char] = &['|', '/', '-', '\\'];

/// Get the current system time in milliseconds.
pub fn get_system_time() -> usize {
    SYSTEM_TIME.load(core::sync::atomic::Ordering::Relaxed)
}

/// Wait for a specified number of milliseconds using the system time.
pub fn wait(ms: usize) {
    let temp = get_system_time();
    while get_system_time() - temp < ms {
        continue;
    }
}

/* ╔═════════════════════════════════════════════════════════════════════════╗
║ Interrupt service routine implementation.                               ║
╚═════════════════════════════════════════════════════════════════════════╝ */

/// Register the timer interrupt handler.
pub fn plugin() {
    TIMER.call_once(|| {
        let mut timer = Timer::new();
        timer.set_interrupt_interval(1);
        INT_VECTORS.lock().register(
            InterruptVector::Pit,
            Box::new(TimerISR { interval_ms: 250}),
        );
        pic::PIC.lock().allow(Irq::Timer);
        timer
    });

    SYSTEM_TIME.store(0, core::sync::atomic::Ordering::Relaxed);
}

/// The timer interrupt service routine.
struct TimerISR {
    /// The interval between timer interrupts in milliseconds.
    interval_ms: usize,
}

impl ISR for TimerISR {
    fn trigger(&self) {

        let current_time = SYSTEM_TIME.fetch_add(1, core::sync::atomic::Ordering::Relaxed);

        if current_time % self.interval_ms == 0 {
            if let Some(mut cga) = CGA.try_lock() {
                let spinner_index = (current_time / self.interval_ms) % SPINNER_CHARS.len();
                cga.setpos(79, 0);
                cga.print_byte(SPINNER_CHARS[spinner_index] as u8);
            }
        }

        // Switch to the next thread in the scheduler.
        unsafe { INT_VECTORS.force_unlock() }
        get_scheduler().yield_cpu();

    }
}

/* ╔═════════════════════════════════════════════════════════════════════════╗
║ Implementation of the PIT driver itself.                                ║
╚═════════════════════════════════════════════════════════════════════════╝ */

/// Represents the programmable interval timer.
struct Timer {
    control_port: IoPort,
    data_port0: IoPort,
}

impl Timer {
    /// Create a new Timer instance.
    pub const fn new() -> Timer {
        Timer {
            control_port: IoPort::new(PORT_CTRL),
            data_port0: IoPort::new(PORT_DATA0),
        }
    }

    /// Set the timer interrupt interval in milliseconds.
    pub fn set_interrupt_interval(&mut self, interval_ms: usize) {
        let divisor = TIMER_FREQ / 1000 / interval_ms;

        unsafe {
            self.control_port.outb(0x36); // Mode 3, counter 0
            self.data_port0.outb((divisor & 0xFF) as u8);
            self.data_port0.outb(((divisor >> 8) & 0xFF) as u8);
        }
    }
}
