use crate::devices::cga;
use crate::devices::cga_print::print;
use crate::kernel::coroutines::coroutine::Coroutine;

fn coroutine_loop(coroutine: &mut Coroutine) {

    let id = coroutine.get_id();
    let mut count = 0;

    loop {
        cga::CGA.lock().setpos(10, 10 + id);
        print!("Coroutine [{}]: {}\n", id, count);

        count += 1;

        coroutine.switch();
    }

}

pub fn run() {

    let mut coroutine1 = Coroutine::new(coroutine_loop);
    let mut coroutine2 = Coroutine::new(coroutine_loop);
    let mut coroutine3 = Coroutine::new(coroutine_loop);

    coroutine1.set_next(&mut coroutine2);
    coroutine2.set_next(&mut coroutine3);
    coroutine3.set_next(&mut coroutine1);

    cga::CGA.lock().clear();

    print!("Coroutine Demo:");
    coroutine1.start();



}