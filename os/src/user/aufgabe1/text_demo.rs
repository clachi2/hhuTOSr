use crate::devices::cga; // shortcut for cga
use crate::devices::cga_print; // used to import code needed by println!

pub fn run() {
    println!("Test der Formatierungsfunktionen mit Zahlen:");
    println!(" ");
    println!("| Dezimal | Hexadezimal | Binaer    |");
    println!("-------------------------------------");
    for i in 0..16 {
        println!("| {:>7} | {:#11x} | {:<9b} |", i, i, i);
    }
}
