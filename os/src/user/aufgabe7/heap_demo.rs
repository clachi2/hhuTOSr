use crate::devices::{cga, keyboard};
use usrlib::allocator;
use crate::shell_println;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug)]
struct TestStruct {
    x: u32,
    y: u32,
}

pub fn heap_demo(args: &[&str]) {
    shell_println!("Starte Heap-Demo...\n");
    shell_println!("\n========================================\n");
    shell_println!("Demo 1/4: 2 Structs dynamisch allozieren\n");
    shell_println!("Freispeicherliste");
    shell_println!("{:?}\n", allocator::dump_free_list_shell());
    shell_println!("Structs anlegen");
    let s1 = Box::new(TestStruct { x: 1, y: 2 });
    shell_println!("s1.x={}, s1.y={}", s1.x, s1.y);
    let s2 = Box::new(TestStruct { x: 3, y: 4 });
    shell_println!("s2.x={}, s2.y={}", s2.x, s2.y);
    shell_println!("\nFreispeicherliste");
    shell_println!("{:?}\n", allocator::dump_free_list_shell());
    shell_println!("Irgend eine Taste druecken, um fortzufahren...");
    keyboard::get_key_buffer().wait_for_key();

    shell_println!("\n========================================\n");
    shell_println!("Demo 2/4: 2 Structs wieder freigeben\n");
    drop(s1);
    drop(s2);
    shell_println!("Freispeicherliste");
    shell_println!("{:?}\n", allocator::dump_free_list_shell());
    shell_println!("Irgend eine Taste druecken, um fortzufahren...");
    keyboard::get_key_buffer().wait_for_key();

    shell_println!("\n========================================\n");
    shell_println!("Demo 3/4: Vec mit drei Structs anlegen und Inhalt eines Structs ausgeben\n");
    shell_println!("Vec anlegen");
    let mut vec = Vec::new();
    for i in 0..3 {
        let s = Box::new(TestStruct { x: i, y: i + 1 });
        vec.push(s);
    }
    shell_println!("Drei Structs angelegt\n");
    shell_println!("Freispeicherliste");
    shell_println!("{:?}\n", allocator::dump_free_list_shell());
    shell_println!("Irgend eine Taste druecken, um fortzufahren...");
    keyboard::get_key_buffer().wait_for_key();

    shell_println!("\n========================================\n");
    shell_println!("Demo 4/4: Vec mit Structs wieder loeschen\n");
    drop(vec);
    shell_println!("Freispeicherliste");
    shell_println!("{:?}\n", allocator::dump_free_list_shell());
    shell_println!("*** ENDE DER DEMO ***\n");
}
