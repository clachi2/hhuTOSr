use crate::devices::{cga, keyboard};
use crate::kernel::allocator;
use alloc::boxed::Box;
use alloc::vec::Vec;

#[derive(Debug)]
struct TestStruct {
    x: u32,
    y: u32
}

pub fn run() {
    println!("Starte Heap-Demo...");

    // let box1 = Box::new(TestStruct {
    //     x: 42,
    //     y: 123,
    // });
    // let box2 = Box::new(TestStruct {
    //     x: 100,
    //     y: 200,
    // });
    // let mut vec = Vec::new();
    // for i in 0..5 {
    //     vec.push(i);
    // }
    //
    // println!("Box2 angelegt: {:?}", box2);
    // println!("Box1 angelegt: {:?}", box1);
    // println!("Vector angelegt: {:?}", vec);
    //
    // // heap überfüllen -> Error erwartet
    // let mut heap_fill = Vec::new();
    // for i in 0..(1024 * 1024 / core::mem::size_of::<u8>()) {
    //     heap_fill.push(i as u8);
    // }

    let mut keyboard = keyboard::KEYBOARD.lock();

    cga::CGA.lock().clear();
    println!("Demo 1/4: 2 Structs dynamisch allozieren");
    println!(" ");
    println!("========================================");
    println!(" ");
    println!("Freispeicherliste");
    allocator::dump_free_list();
    println!(" ");
    println!("Structs anlegen");
    let s1 = Box::new(TestStruct {
        x: 1,
        y: 2
    });
    println!("s1.x={}, s1.y={}", s1.x, s1.y);
    let s2 = Box::new(TestStruct {
        x: 3,
        y: 4
    });
    println!("s2.x={}, s2.y={}", s2.x, s2.y);
    println!(" ");
    println!("Freispeicherliste");
    allocator::dump_free_list();
    println!(" ");
    println!("Weiter mit <ENTER>");
    let mut key = keyboard.key_hit();
    while !key.valid() || key.get_scancode() != 28 {
        key = keyboard.key_hit();
    }


    cga::CGA.lock().clear();
    println!("Demo 2/4: 2 Structs wieder freigeben");
    println!(" ");
    println!("========================================");
    println!(" ");
    drop(s1);
    drop(s2);
    println!("Freispeicherliste");
    allocator::dump_free_list();
    println!("Weiter mit <ENTER>");
    let mut key = keyboard.key_hit();
    while !key.valid() || key.get_scancode() != 28 {
        key = keyboard.key_hit();
    }


    cga::CGA.lock().clear();
    println!("Demo 3/4: Vec mit drei Structs anlegen und Inhalt eines Structs ausgeben");
    println!(" ");
    println!("========================================");
    println!(" ");
    println!("Vec anlagen");
    let mut vec = Vec::new();
    for i in 0..3 {
        let s = Box::new(TestStruct {
            x: i,
            y: i + 1
        });
        vec.push(s);
    }
    println!("Drei Structs angelegt");
    println!(" ");
    println!("Freispeicherliste");
    allocator::dump_free_list();
    println!(" ");
    println!("Weiter mit <ENTER>");
    let mut key = keyboard.key_hit();
    while !key.valid() || key.get_scancode() != 28 {
        key = keyboard.key_hit();
    }


    cga::CGA.lock().clear();
    println!("Demo 4/4: Vec mit Structs wieder loeschen");
    println!(" ");
    println!("========================================");
    println!(" ");
    drop(vec);
    println!("Freispeicherliste");
    allocator::dump_free_list();
    println!(" ");
    println!("*** ENDE DER DEMO ***");
}
