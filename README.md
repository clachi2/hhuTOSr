# Aufgabe 1: Ein-/Ausgabe

Der Quellcode zum Betriebssystem befindet sich im Unterordner [os](os/). Dort finden Sie auch eine README-Datei mit Anleitungen zum Kompilieren und Starten von hhuTOS.

## Lernziele
1. Kennenlernen der Entwicklungsumgebung
2. Einarbeiten in die Programmiersprache Rust
3. Hardwarenahe Programmierung: CGA-Bildschirm und Tastatur


## A1.1: CGA-Bildschirm
Für Testausgaben und zur Erleichterung der Fehlersuche soll das Betriebssystem zunächst Ausgabefunktionen für den Textbildschirm erhalten. Die Funktionsfähigkeit soll mit Hilfe eines aussagefähigen Testprogramms gezeigt werden, siehe Bildschirmfoto unten.

Dazu soll in `startup.rs` in der Einstiegsfunktion `startup` die Makros `print!` und `println!` für verschieden formatierte Ausgaben, wie in Rust üblich, genutzt werden. Damit die Ausgabe-Makros in allen Modulen funktionieren wurde in `cga_print.rs` ein globaler statischer `Writer`
definiert. Dieser wird in den vorgegebenen Makros automatisch benutzt.

In folgenden Dateien müssen Quelltexte einfügt werden: `startup.rs`, `user/text_demo.rs` und
`devices/cga.rs`

*Beachten Sie die Kommentare im Quelltext der Vorgabe, sowie die Datei* `CGA-slides.pdf`

### Beispielausgaben

![CGA](img/cga.png)


## A1.2: Tastatur
Damit eine Interaktion mit dem Betriebssystem möglich wird benötigen wir einen Tastatur-Treiber. In dieser Aufgabe verwenden wir die Tastatur ohne Interrupts. In main soll die Tastatur in einer Endlos-Schleife abgefragt werden und die Eingaben auf dem CGA-Bildschirm zur Kontrolle ausgegeben werden. 

Beginnen Sie mit der Funktion `key_hit`:
- Prüfen Sie zunächst in einer Schleife, ob ein Datenbyte von der Tastatur vorliegt. Hierzu muss im Control-Port geprüft werden, ob das Bit `OUTB` gesetzt ist.
- Lesen Sie anschließend das Datenbyte über den Daten-Port ein und speichern Sie das gelesene Byte in der gegebenen Variable code.
- Verwenden Sie die vorgegeben Funktion `key_decoded` um jeweils ein gelesenes Datenbyte zu übersetzen. Jedoch müssen Sie zuvor prüfen, ob das Datenbyte nicht von einer PS/2 Maus stammt. Dies wird über das Bit `AUXB` im Control-Register angezeigt. Beim Aufruf von `key_decoded` müssen Sie das das Datenbyte nicht übergeben, dies ist bereits in der Variablen `code` gespeichert.
- Wenn `key_decoded` true zurückgibt wurde eine Taste komplett dekodiert und in der Variablen `gather` gespeichert. Geben Sie in diesem Fall `gather` (Typ `Key`) zurück oder ansonsten `invalid`. 

Danach können folgende Funktionen implementiert werden: `set_repeate_rate` und `set_led`. Beide Funktion können, müssen aber nicht implementiert werden.

Namen von benötigten Variablen und Konstanten:
- Control-Port: `KBD_CTRL_PORT`
- Daten-Port: `KBD_DATA_PORT`
- OUTB: `KBD_OUTB`
- AUXB: `KBD_AUXB`

Die Befehle für die Implementierung von `set_led` finden Sie in `keyboard.rs`. Warten und prüfen Sie nach dem Absenden eines Befehls die Antwort auf `KBD_REPLY_ACK`. 
Die Tabellen für die Abbildung von Scan-Codes auf ASCII-Codes unterstützen derzeit keine Umlaute.

In folgenden Dateien müssen Quelltexte einfügt werden: `user/keyboard_demo.rs` und
`devices/keyboard.rs`.

*Achtung:
Die Methoden zur Ansteuerung der LEDs und der Tastaturwiederholrate funktionieren nur richtig auf echter Hardware.*

*Beachten Sie die Kommentare im Quelltext der Vorgabe, sowie die Datei* `KBD-slides.pdf`.

# Aufgabe 2: Speicherverwaltung und PC-Speaker

## Lernziele
1. Verstehen wie eine Speicherverwaltung funktioniert und implementiert wird.
2. Hardwarenahe Programmierung: PC-Speaker / Programmable Interval Timer

Detaillierte Infos zu dieser Aufgabe finden sich [hier](https://os.phil-opp.com/allocator-designs/). Allgemeine Hinweise zu einer Heap-Verwaltung finden sich in `MEM-slides.pdf`.

## A2.1: Bump-Allocator
In dieser Aufgabe soll ein sehr einfacher sogenannter Bump-Allocator implementiert werden, um zunächst die Integration in das System zu verstehen sowie die Anbindung an die Programmiersprache. Dieser Allokator kennt lediglich den Heap-Anfang, das Heap-Ende und merkt sich in der Variablen `next` die aktuelle Adresse im Heap, ab welcher der Speicher frei ist. Bei jeder Allokation wird `next` um die gewünschte Anzahl Bytes weitergesetzt, sofern nicht das Heap-Ende erreicht ist, siehe Abbildung.

![Bump-Allocator](img/bump_allocator.jpg)

Die Heapgröße ist fest auf 1 MB eingestellt, im Speicherbereich 5 – 6 MiB. Bei einer Speicherfreigabe passiert nichts. Bauen Sie die Vorgabe in Ihr System ein und stellen Sie sicher, dass der Heap möglichst bald in der Einstiegsfunktion des Betriebssystems initialisiert wird.

Zur Überprüfung der Implementierung sollen einfache Tests geschrieben werden. Weitere Information hierzu finden sich in den nachfolgenden Hinweisen zur jeweiligen Programmiersprache.

In der Datei `bump.rs` soll die Bump-Speicherverwaltung implementiert werden. Die Integration in die Rust-Runtime erfolgt über das `GloballAlloc` trait. Der Speicherallokator wird in `allocator.rs` in der statischen Variable `ALLOCATOR` angelegt und muss möglichst früh in `startup.rs` initialisiert werden.

Als Tests sollen in `heap_demo.rs` eigene Structs mithilfe von `Box::new` auf dem Heap angelegt werden. Zu beachten ist, dass es in Rust kein klassisches `delete` gibt.

Sofern die Ownership der Structs nicht weitergegeben wird, so werden die Structs beim Rücksprung aus der Funktion, in der sie angelegt wurden, automatisch freigegeben, indem automatisch `deallocate` im Allokator aufgerufen wird. Sie können Objekte jedoch auch mittels `drop()` manuell frühzeitig freigeben.

Im Gegensatz zu C/C++ muss das Längenfeld eines belegten Blocks bei der Allokation nicht manuell
behandelt werden. Dies erledigt die Rust-Runtime automatisch, jedoch ist der Parameter `layout` in `alloc` und `dealloc` zu beachten.

In folgenden Dateien müssen Quelltexte einfügt werden: `kernel/allocator/bump.rs` und
`user/aufgabe2/heap_demo.rs`.

*Achtung: Die Pointer auf einen neu allozierten Speicherblock müssen aligniert werden. Wie die Alignierung aussehen muss steht im Parameter* `layout` *beim Aufruf von* `alloc`*. In* `allocator.rs` *gibt es hierfür die Hilfsfunktion* `align_up`. 

## A2.2: Listenbasierter Allokator
In dieser Aufgabe soll ein verbesserter Allokator implementiert werden, welcher freigegebene Speicherblöcke wiederverwenden kann. Hierzu sollen alle freien Blöcke miteinander verkettet werden.

Zu Beginn gibt es nur einen großen freien Speicherblock, der den gesamten freien Speicher umfasst. Im Rahmen der Heap-Initialisierung in `LinkedListAllocator::init` soll dieser eine freie Block als erster und einziger Eintrag in der verketteten Freispeicherliste gespeichert werden, siehe Abbildung.

![List-Allocator](img/list_allocator-1.jpg)

Die globale Variable `ALLOCATOR` (liegt im generierten OS-Image) speichert den Anfang `hs` und das Ende `he` des Heaps sowie einen Dummy `ListNode` mit der Länge 0. Der Dummy dient nur dazu den Einstieg in die Freispeicherliste zu speichern. Nach der Initialisierung liegt im Heap ein `ListNode`, welcher dessen Länge der Heapgröße entspricht, in unserem Fall 5 MB.  

**Allokation**. Bei der Allokation eines Speicherblocks muss die Freispeicherliste nach einem passenden Block durchsucht werden. Es reicht, wenn immer der erste Block genommen wird, der mindestens die Größe der Allokation erfüllt. Sofern der verbleibende Rest groß genug ist, um die Metadaten eines Listeneintrags zu speichern, so soll dieser abgeschnitten und wieder in die Freispeicherliste eingefügt werden.

**Freigabe**. Der freizugebende Block soll in die Freispeicherliste wieder eingehängt werden. Im Prinzip reicht es, wenn er am Anfang der Liste eingefügt wird. Optional kann geprüft werden, ob benachbarte Speicherbereiche auch frei sind und damit verschmolzen werden kann. Dazu muss in der Liste gesucht werden. 

Damit die Freispeicherverwaltung getestet und geprüft werden kann, ist es sinnvoll eine Ausgabe-Funktion zu implementieren, welche die Freispeicherliste komplett auf dem Bildschirm ausgibt. Zudem soll die Test-Anwendung aus Aufgabe 2.1 ausgebaut werden, um auch die Freigabe von Speicherblöcken zu testen.

Das nachstehende Bild zeigt den Heap mit zwei freien und drei belegten Blöcken.

![List-Allocator](img/list_allocator-2.jpg)

Die folgenden Hinweise sind Ergänzungen zu denen in Aufgabe 2.1!

In der Datei `list.rs` soll die Speicherverwaltung implementiert werden. Der Speicherallokator wird in `allocator.rs` in der statischen Variable `ALLOCATOR` angelegt und muss möglichst früh in `startup.rs` initialisiert werden.

Verwenden/erweitern Sie die Tests aus Aufgabe 2.1. Ein Anregung dazu finden Sie auch in den nachstehenden Abbildungen.

In folgenden Dateien müssen Quelltexte einfügt werden: `kernel/allocator/list.rs` und `user/aufgabe2/heap_demo.rs`.

*Achtung: Die Pointer auf einen neu allozierten Speicherblock müssen aligniert werden. Wie die Alignierung aussehen muss steht im Parameter* `layout` *beim Aufruf von* `alloc`*. In* `allocator.rs` *gibt es hierfür die Hilfsfunktion* `align_up`. 

## A2.3: PC-Lautsprecher
In dieser Aufgabe muss die Funktion `delay` implementiert werden. Diese Funktion ist für das Abspielen von Tönen notwendig, die eine gegebene Zeitdauer (in ms) gespielt werden sollen. Da wir bisher keine Interrupts verarbeiten können und auch keine Systemzeit haben bietet es sich an den Zähler 0 des Programmable Interval Timer (PIT) hierfür zu verwenden. Dieser muss konfiguriert werden, beispielsweise so, dass der er in 1ms auf 0 herunterzählt. Hierfür soll Mode 2 (Rate Generator) für den Zähler 0 verwendet werden. Sobald der Zähler die 0 erreicht hat, wird der konfigurierte Zählwert automatisch wieder neu geladen und wieder heruntergezählt. Um größere Zeiten als 1ms zu warten kann in einer Endlosschleife der Zählerstand ausgelesen werden, um damit zu erkennen, ob die 0 erreicht wurde. Für 100ms Verzögerung würde man entsprechend 100 Mal das Herunterzählen auf 0 erfassen. Beim Auslesen des Zählers wird man selten 0 lesen, da ja ständig runtergezählt wird. Man muss also erkennen, ob der Zähler schon wieder heruntergezählt wird und somit die 0 bereits erreicht wurde. 

Dies ist eine unsaubere Lösung die wir später ersetzen werden.

Hinweis: Gute Informationen zum PIT 8254 finden Sie in der Datei `8254.pdf` sowie auf [OSDev](http://wiki.osdev.org/Programmable_Interval_Timer).

In folgenden Dateien müssen Quelltexte einfügt werden: `devices/pcspk.rs` und `user/aufgabe2/sound_demo.rs`.

## Beispielausgaben zur Speicherverwaltung
Nachstehend sind einige Screenshots zum Testen der Speicherverwaltung. Sie können sich natürlich selbst Testfunktionen und Testausgaben überlegen. Sollten die Ausgaben über mehrere Seiten gehen bietet es sich an auf einen Tastendruck mit `keyboard::key_hit()` zu warten.

![Heap1](img/heap1.jpg)

![Heap2](img/heap2.jpg)

![Heap3](img/heap3.jpg)

![Heap4](img/heap4.jpg)

# Aufgabe 3: Interrupts

## Lernziele

1. Funktionsweise der Interrupt Descriptor Table (IDT) verstehen
2. Funktionsweise des Interrupt-Controllers verstehen
2. Behandlung von Interrupts implementieren, am Beispiel der Tastatur

## A3.1: Interrupt Descriptor Table (IDT)
In dieser Aufgabe soll die Interrupt Descriptor Table (IDT) erstellt und geladen werden.

In der Datei `kernel/interrupts/idt.rs` ist bereits ein Großteil des Codes zu Erstellung einer IDT vorgegeben.
Die IDT hat 256 Einträge, wobei jeder Eintrag auf eine Funktion verweist, die angesprungen werden soll, wenn der entsprechende Interrupt auftritt. In hhuTOS verweist jeder Eintrag auf die Funktion `int_disp()` aus `kernel/interrupts/intdispatcher.rs`, welche die weitere Verarbeitung der Interrupts vornimmt.
Zusätzlich hat jeder Eintag noch einige Flags, die korrekt gesetzt werden müssen (`IdtEntry::options`).

Implementieren Sie zunächst die Funktion `IdtEntry::new()`, welche einen neuen IDT-Eintrag erzeugen soll. Der Parameter `offset` repräsentiert dabei die Adresse der anzuspringenden Funktion und muss innerhalb des Eintrags auf drei einzelne Teile aufgeteilt werden (`IdtEntry::offset_low`, `IdtEntry::offset_mid`, `IdtEntry::offset_high`). Außerdem soll jeder Eintrag immer die Optionen `Present`, `DPL = 0` und `64-Bit Interrupt Gate` gesetzt haben. Details zum Aufbau eines IDT-Eintrags finden Sie im [OSDev Wiki](https://wiki.osdev.org/Interrupt_Descriptor_Table#Structure_on_x86-64).

Laden Sie die IDT in `startup.rs` mit `idt::get_idt().load()`. Nun sollte bei jedem Interrupt die Funktion `int_disp()` aufgerufen werden. Um das zu testen, fügen Sie eine Ausgabe in die (noch leere) Funktion `int_disp()` ein. Hierfür soll `kprintln!()` und nicht `println!()` verwendet werden. Zudem sollte `kprintln!()` nicht in Anwendungscode genutzt werden. Hintergrund ist, dass die `kprintln()!` und `println!()` Makros intern einen Mutex verwenden, welcher eventuell während der Interrupt-Verarbeitung gerade durch die Anwendung gesperrt ist. In diesem Fall würde eine Verklemmung auftreten.

Um manuell einen Interrupt auszulösen können Sie die x86-Instruktion `int` in `startup.rs` verwenden: `unsafe { asm!("int 100"); }` sollte nun `int_disp()` mit dem Parameter `vector = 100` anspringen.

*Hinweis: Unsere Interrupt Handler werden in `idt.rs` also `extern "x86-interrupt"` markiert um dem Rust-Compiler mizuteilen, dass es sich nicht um normale Funktionen handelt und der generierte Maschinencode etwas anders aussehen muss (z.B. `iret` statt `ret` um aus der Funktion zurückzukehren). Dieses Feature ist in Rust noch nicht stabil und muss daher manuell eingeschaltet werden. Dazu muss in der `startup.rs` oben die Zeile `#![feature(abi_x86_interrupt)]` eingefügt werden.*

In folgenden Dateien muss Code implementiert werden: `kernel/interrupts/idt.rs`, `startup.rs`.

## A3.2: Programmable Interrupt Controller (PIC)
In dieser Aufgabe sollen Hardware Interrupts aktiviert und anhand der Tastatur getestet werden.

Zunächst müssen die leeren Funktionen in `pic.rs` implementiert werden. 

Anschliessend soll in `keyboard.rs` die Funktion `plugin` programmiert werden. Hier muss der IRQ der Tastatur am `PIC` mit `allow()` freigeschaltet werden. Die ISR `keyboard::trigger()` kann vorerst leer bleiben. Auch das Registrieren der ISR der Tastatur folgt später.

In `startup.rs` muss die `init()` Funktion des PIC aufgerufen, sowie die ISR der Tastatur mit `keyboard::plugin()` registriert werden. Anschliessend müssen die Interrupts an der CPU mit `cpu::enable_int()` zugelassen werden.

Wenn nun das System startet sollte bei jedem Drücken und Loslassen einer Taste eine Textmeldung von `int_disp()` zu sehen sein. Dies funktioniert allerdings nur einige wenige male (oder sogar nur ein einziges mal). Wenn die Zeichen nicht vom Tastaturcontroller abgeholt werden, läuft der Tastaturpuffer irgendwann voll. Sobald der Puffer voll ist, sendet der Tastaturcontroller keine Interrupts mehr.

In folgenden Dateien muss Code implementiert werden: `kernel/interrupts/pic.rs`,
`devices/keyboard.rs`, `startup.rs` und `kernel/interrupts/int_dispatcher.rs`.

*Allgemeine Hinweise:*
- *Während der Behandlung einer Unterbrechung braucht man sich um unerwünschte Interrupts nicht zu sorgen. Der Prozessor schaltet diese nämlich automatisch aus, wenn er mit der Behandlung beginnt, und lässt sie erst wieder zu, wenn die Unterbrechungsbehandlung beendet wird. Zudem nutzen wir nur einen Prozessor-Kern.*
- *Die Interrupt-Verarbeitung kann nur funktionieren, wenn hhuTOS auch läuft. Sobald hhuTOS die main-Funktion verlässt, ist das Verhalten bei Auftreten eines Interrupts undefiniert. Ein Betriebssystem sollte eben nicht plötzlich enden :-)*


**Beispielausgaben in `int_disp()`**:
```
Welcome to hhuTOS!
Initializing heap allocator
Initializing PIC
Initializing interrupts
int_disp: Interrupt 100!
Initializing keyboard
Enabling interrupts
Boot sequence finished
int_disp: Interrupt 33!
```

## A3.3: Weiterleitung von Interrupts an die Geräte-Treiber
In dieser Aufgabe soll eine Infrastruktur geschaffen werden, um Interrupts, welche in `int_disp()` (siehe Aufgabe A3.2) entgegengenommen werden, an eine zuvor registierte Interrupt-Service-Routine (ISR) in einem Treiber weiterzuleiten.

Ein Treiber muss hierfür eine ISR implementieren und registrieren. Die Schnittstelle der ISR besteht „nur“ aus der `trigger()` Funktion. Zu beachten ist, dass der Interrupt-Dispatcher mit Vektor-Nummern arbeitet und nicht IRQ-Nummern wie der PIC.

Zur Verwaltung der ISR verwendet das Modul `intdispatcher` die dynamische Datenstruktur `Vec`, welche mit 256 Options, die den Wert `None` beinhalten, gefüllt wird. Dies erlaubt es in `register()` eine ISR eines Treibers (Schnittstelle definiert in `isr`) an einem gegebenen Index zu speichern. Leider geht dies in Rust nicht mit einem Array statischer Größe. 

Die Funktion `report()` soll von `int_disp()` aufgerufen werden, um die Funktion `trigger()` einer registrierten ISR-Funktion aufrufen, sofern vorhanden. Falls keine ISR registriert wurde, also `None` eingetragen ist, so soll eine Fehlermeldung ausgegeben und das System gestoppt werden. Entfernen Sie nun unbedingt den manuellen Test-Interrupt in `startup.rs`, da es sonst zu genau diesem Fall kommt.

Um `report()` aufzurufen muss der Mutex um `INT_VECTORS` gelocked werden. Normalerweise ist es keine gute Idee, während eines Interrupts ein Lock zu holen, da es zu einer Verklemmung kommt falls das Lock bereits vergeben ist. In diesem Fall würde der Interrupt Handler nie zurückkehren und das Betriebssystem hängen bleiben (Gleiche Problematik wie bei `println!()`/`kprintln!()`). Um das zu verhindern, sollten alle Interrupt Handler registriert werden, bevor die Interrupts mit `cpu::enable_int()` eingeschaltet werden.

Im Modul `keyboard` muss die Funktion `plugin()` erweitert werden und eine Referenz auf ein Funktionsobjekt `KeyboardISR` mithilfe von `register()` (im Modul `intdispatcher`) registrieren. Die für die Tastatur notwendige Vektor-Nummer ist in `intdispatcher::InterruptVector` definiert. 

Des Weiteren soll eine Text-Ausgabe in die Funktion `trigger()` eingebaut werden, um zu prüfen, ob die Tastaturinterrupts hier ankommen. Auch hier soll für Textausgaben `kprintln!()` verwendet werden.

In folgenden Dateien muss Code implementiert werden: `devices/keyboard.rs`, `kernel/interrupts/intdispatcher.rs` und `startup.rs`.

**Beispielausgaben in `keyboard::trigger()`**:
```
Welcome to hhuTOS!
Initializing heap allocator
Initializing PIC
Initializing interrupts
Initializing keyboard
Enabling interrupts
Boot sequence finished
int_disp: Interrupt 33!
keyboard::trigger called!
```

## A3.4: Tastaturabfrage per Interrupt
Nun soll die Funktion `trigger()` in `keyboard` implementiert werden. Bei jedem Interrupt soll `key_hit_irq()` aufgerufen, ein Byte eingelesen werden und geprüft werden, ob ein Zeichen erfolgreich dekodiert wurde. Wenn dies der Fall ist, so soll der ASCII-Code des Zeichens in die neue globale Variable `KEYBOARD_BUFFER` eingereiht werden. Dabei handelt es sich um eine Queue, auf welche Anwendungen später mit `keyboard::get_key_buffer()` zugreifen und Tasten auslesen können. In `library/input.rs` sind zwei Beispielfunktionen die `keyboard::get_key_buffer()` verwenden. Für die Queue verwenden wir die Crate [nolock](https://lib.rs/crates/nolock), welche Datenstrukturen zur Verfügung stellt, die ohne Locks auskommen und trotzdem konkurriernde Zugriffe unterstützen. Solche Strukturen eignen sich perfekt für die Interruptverarbeitung, da wir ja innerhalb eines Interrupt Handlers normalerweise keine Locks holen dürfen.

In `trigger()` muss die globale Variable `KEYBOARD` gelocked werden, damit `key_hit_irq()` mit einer mutable self Referenz aufgerufen werden kann. Das ist in Ordnung, da die gesamte Tastenverarbeitung in `trigger()` stattfindet und `KEYBOARD` an keiner anderen Stelle mehr gelocked wird.

Bauen Sie nun alle bisherigen Demos so um, dass sie nicht mehr `key_hit()` verwenden um auf einen Tastendruck zu warten, sondern stattdessen Tasten aus dem `KEYBOARD_BUFFER` abholen. Die Funktion `key_hit()` kann nun nicht mehr verwendet werden und sollte gelöscht werden.

*Hinweise:*
- *In `key_hit_irq()` sollte zumindest ein Byte eingelesen werden, da ansonsten keine weitere Interrupts von der Tastatur durchkommen.*
- *Die PS/2-Maus hängt ebenfalls am Keyboard-Controller, verwendet aber IRQ12. Da wir keinen Handler für IRQ12 haben, kann es sein, dass wenn IRQ1 auftritt noch Daten von der Maus abzuholen sind. Dies können Sie anhand des `AUXB`-Bits im Statusregister erkennen.*
- *Ferner tritt unter Qemu manchmal direkt ein IRQ1 nach dem Start auf, ohne eine Tastatureingabe. Das ist auf echter Hardware nicht der Fall. Daher unter Qemu bitte ignorieren.*

# Aufgabe 4: Koroutinen und Threads

## Lernziele
1. Auffrischen der Assemblerkenntnisse
2. Verständnis der Abläufe bei einem Koroutinen-Wechsel
3. Unterschied zwischen Threads und Koroutinen
3. Verstehen wie ein Scheduler funktioniert

FÜr diese Aufgabe sollte zuvor der Assembler-Crashkurs in `ASM-slides.pdf` gelesen werden.

## A4.1: Koroutinen
In dieser Aufgabe soll die Umschaltung zwischen Koroutinen in Assembler programmiert werden. Koroutinen sind eine Vorstufe zu Threads die später (siehe unten) zusätzlich eingeführt werden. 

Sehen Sie sich zunächst die Inhalte der neuen Dateien in der Vorgabe im Ordner `kernel/coroutines` an und implementieren Sie die beiden Assemblerfunktionen `coroutine_start()` und `coroutine_switch()` in `coroutine.rs`. Der Zustand (alle Register) einer Koroutine soll auf dem Stack gesichert werden. Das `rflags`-Register kann nicht direkt per move-Befehl zugegriffen werden, sondern nur mithilfe der Instruktionen `popf` und `pushf`. Wir nutzen `naked` Funktionen um Assembler-Code in unseren Rust-Code zu integrieren. Solche "nackten" Funktionen beinhalten ausschließlich Assmebler-Code. Es ist auch nicht möglich auf Rust-Variablen zuzugreifen. Alle Parameter müssen manuell aus den entsprechenden CPU-Registern ausgelesen werden, Alle Assembler-Befehle müssen als einzelne, durch Kommata separierte, Strings nacheinander in das `naked_asm()!` Makro eingetragen werden.

Der Zeiger auf den letzten Stack-Eintrag soll in der Instanzvariablen `stack_ptr` in der Struct `Coroutine` gespeichert werden.

Ergänzen Sie anschließend die leeren Methoden in `coroutine.rs`. Die Verkettung der Koroutinen erfolgt über `next` in der `struct Coroutine`.

Schreiben Sie für Ihre Koroutinen-Implementierung folgendes Testprogramm. Im Verzeichnis
`user/aufgabe4` der Vorgabe finden Sie hierfür Dateien. Es sollen drei Koroutinen erzeugt und zyklisch miteinander verkettet werden. Jede Koroutine soll einen Zähler hochzählen und an einer festen Position auf dem Bildschirm ausgeben und dann auf die nächste Koroutine umschalten. Durch die Verkettung werden die drei Koroutinen dann reihum abwechselnd ausgeführt, wodurch die Zähler scheinbar nebenläufig vorangetrieben werden, siehe nachstehende Abbildung. Beim Ändern der Cursor-Position müssen Sie darauf achten, dass die CGA-Instanz nur temporär zum Setzen des Cursors gelockt wird, da `println!()` die CGA-Instanz ebenfalls locken wird. Das wird bei präemptiven Multitasking ein Problem, da dann die CPU zwischen `setpos()` und `println!()` entzogen werden kann. Damit werden wir uns jedoch erst auf dem nächsten Übungsblatt beschäftigen.

In folgenden Dateien muss Code implementiert werden: `kernel/corouts/coroutine.rs`, `user/aufgabe4/coroutine_demo.rs` und `startup.rs`.

Hinweis: Schauen Sie sich vor dem Programmieren der Assemblerfunktionen nochmals die Aufrufkonvention für die Parameterübergabe an.

**Beispielausgaben der Koroutinen**

![coroutine_demo](img/coroutine_demo.png)

(In eckigen Klammern wird die Koroutinen-ID angezeigt.)

## A4.2: Warteschlange
Der Scheduler benötigt eine Warteschlange (engl. queue) bei der immer am Anfang einer einfach verketteten Liste ein Element entfernt wird (Thread der als nächstes die CPU erhält) und immer Ende eingefügt wird (zum Beispiel ein neuer Thread oder ein Thread der die CPU abgibt).

In Rust ist die Implementierung einer verketteten Liste anspruchsvoll, weswegen „nur“ die Funktion `remove()` implementiert werden muss.

In folgender Datei muss Code implementiert werden: `mylib/queue.rs`.


## A4.3: Umbau der Koroutinen auf Threads
Im Verzeichnis `kernel/threads` finden Sie eine Vorgabe, die sehr ähnlich zu `coroutine.rs` ist. Sie können Ihren Code aus `coroutine.rs` größtenteils übernehmen und müssen nur bei Funktionsaufrufen die Namen der Funktionen entsprechend anpassen.

Vergleichen Sie die Änderungen in `thread.rs` gegenüber `coroutine.rs`. Insbesondere ist `next` nicht in `struct Thread`, da die Threads nun in der Queue aus Aufgabe A4.2 verwaltet werden sollen und nicht wie die Koroutinen direkt verkettet sind.

*Hinweis: Diese Aufgabe kann nicht separat getestet werden.*


## A4.4 Scheduler
Nun soll ein einfacher Scheduler implementiert werden. Alle Threads werden in einer "Ready Queue" (siehe A4.2) verwaltet und bekommen reihum die CPU (nach freiwilliger Abgabe mittels `yield_cpu()`. Es gibt keine Prioritäten und es ist sinnvoll, dass der aktuell laufende Thread nicht in der Warteschlange gespeichert wird. In der Vorgabe ist die Implementierung für den Idle-Thread gegeben, welcher läuft, falls kein Anwendungsthread in der Ready Queue ist.

Alle Methoden des Schedulers werden mit einer const `&self` Referenz aufgerufen. Das funktioniert, da der Zustand des Schedulers innerhalb des `struct Scheduler` durch einen `Mutex` geschützt wird. Jede Methode des Schedulers muss also zunächst den Mutex locken, bevor sie auf die Instanzvariablen zugreifen kann. Bei `yield_cpu()` und `exit()` ergibt sich dabei folgendes Problem: Normalerweise wird der `Mutex` wieder freigegeben, sobald der Scope (also die Methode) verlassen wird. Da wir in diesen beiden Methoden jedoch zu einem anderen Thread wechseln, wird das Scope nicht direkt verlassen und der `Mutex` bleibt gelockt. Jeder weitere Aufruf einer Scheduler-Methode würde nun zu einem Deadlock führen. Um das zu verhindern, muss die Funktion `unlock_scheduler()` aus Ihrem Assembler-Code in `thread_start()` und `thread_switch()` direkt nach dem Setzen des `rsp` aufgerufen werden. 

Testen Sie den Scheduler zunächst nur mit dem Idle-Thread. Bauen Sie hierzu eine Textausgabe in den Idle-Thread ein.

In der gegebenen Datei `scheduler.rs` sind die gekennzeichneten Funktionn zu implementieren. Bei einem Thread-Wechsel mittels `yield_cpu()` soll der aktive Thread am Ende der `ready_queue` eingereiht werden. Der nächste Thread wird am Kopf der `ready_queue` ausgereiht und in `active` gespeichert. Nun wird auf den neuen aktiven Thread gewechselt.

*Hinweis: Da auf den alten Thread nach der Einreihung in die `ready_queue` nicht mehr zugegriffen werden kann, empfiehlt es sich vorher einen Pointer auf diesen Thread in einer lokalen Variable zu speichern, da dieser noch für `thread_switch()` benötigt wird.*

## A4.5 Eine multi-threaded Testanwendung
Die Vorgabe beinhaltet einen HelloWorld-Thread (`user/aufgabe4/hello_world_thread.rs`), um einen ersten Test durchzuführen. Der Thread gibt einen Spruch aus und terminiert sich dann. Anschließend soll nur noch der Idle-Thread ausgeführt werden. Um dies zu testen soll der Idle-Thread und der HelloWorld-Thread in `main` angelegt und im Scheduler registriert werden. Anschließend soll der Scheduler mit `scheduler::Scheduler::schedule()` gestartet werden.

Als zweiter eigener Test soll nun Anwendungsbeispiel mit den drei Zählern aus Aufgabe 4.1 von Koroutinen auf Threads umgebaut werden. Testen Sie hierbei auch Ihre Implementierung von `kill()`, indem einer der Zähler-Threads nach einer gewissen Zeit die anderen beiden abschießt und `exit()` indem sich der letzte Thread bei einem gewissen Zählerstand selbst beendet, so dass nur noch der Idle-Thread übrig bleibt.

**Beispielausgaben der Threads**

![thread_demo](img/thread_demo.png)


# Aufgabe 5: Preemptives Multithreading

## UPDATE (05.06.2025)
Es kann passieren, dass der Allokator gelockt ist, während der PIT einen Thread-Wechsel einleitet. Das würde zu einem Deadlock führen, da das Aus- und Einreihen von Threads Heap-Speicher freigibt, bzw. alloziert.

Fügen Sie die folgende Funktion in `kernel/allocator.rs` ein, mit der überprüft werden kann, ob der Allokator gerade gelockt ist:
```
pub fn is_locked() -> bool {
    ALLOCATOR.inner.is_locked()
}
```

In `scheduler::yield_cpu()` können Sie nun nach dem Locken der Ready Queue prüfen, ob der Allokator gelockt ist, und in dem Fall einfach mit `return` den Thread-Wechsel abbrechen.

## Lernziele
1. Tieferes Verständnis von präemptiven Multitasking
2. CPU-Entzug mithilfe des PIT
3. Synchronisierung des Schedulers und des Allokators gegenüber dem PIT-Interrupt


## A5.1: Programmable Interval Timer (PIT)
Der PIT wird ab sofort verwendet, um eine Systemzeit sowie ein erzwungenes Umschalten zwischen Threads zu realisieren. Die Systemzeit wird in der Variable 
`SYSTEM_TIME` (in `pit.rs`) gespeichert und diese soll bei jedem Interrupt für den PIT inkrementiert werden.
Verwenden Sie hierfür im PIT den Zähler 0 und Modus 3 und laden Sie den Zähler mit einem passenden Wert, sodass der PIT jede Millisekunde ein Interrupt ausgelöst.
Jeder Interrupt verursacht also eine Inkrementierung und entspricht einem Tick (1ms). Somit zeigt `SYSTEM_TIME` an, wie viele Ticks seit dem Beginn der Zeiterfassung vergangen sind. 

Im Interrupt-Handler des PITs soll die Systemzeit in Form eines rotierenden Zeichens (engl. spinner) an einer festen Stelle dargestellt werden. Verwenden Sie hierfür beispielsweise die rechte obere Ecke und folgende Zeichen: `| / - \` (vorgegeben in `SPINNER_CHARS`), wobei das Zeichen in einem festen Intervall (z.B. alle 250ms) gewechselt werden soll. Hierzu muss in `trigger()` die `CGA` Instanz gelockt werden. Sollte das Lock gerade nicht verfügbar sein, würde dies zu einem Deadlock führen, da wir nie aus dem Interrupt Handler zurückkehren würden. Verwenden Sie `try_lock()` um dies zu vermeiden. Sollte das Lock nicht verfügbar sein, wird das Zeichen einfach nicht ausgegeben. Früher oder später wird das Lock mal frei sein und das Zeichen aktualisiert werden.

Die Funktion `plugin()` soll den den PIT mit Hilfe von `TIMER.call_once(|| { ... })` initialisieren, das Interrupt Intervall setzen und ihn in `intdispatcher.rs` anmelden. Außerdem sollen die Timer Interrupts im PIC zugelassen werden. Rufen Sie `plugin()` in `startup.rs` auf, um den Timer zu starten.

In folgenden Dateien muss Code implementiert werden: `devices/pit.rs` und `startup.rs`.

## A5.2: Umbau des Treibers für den PC-Lautsprecher
Die `delay()` Funktion im Treiber für den PC-Lautsprecher hat bisher den PIT direkt programmiert und dafür den Zähler 0 verwendet. Das geht nun nicht mehr, da Zähler 0 nun anderweitig benötigt wird, siehe A5.1.
Daher sollen alle `delay()` Aufrufe durch `pit::wait()` ersetzt werden.
Die Funktion `pit::wait()` soll in einer Dauerschleife die `SYSTEM_TIME` abfragen, bis eine gegebene Anzahl an Millisekunden vergangen ist.

Testen Sie den Umbau mit einer der Melodien.

In folgenden Dateien muss Code implementiert werden: `devices/pcspk.rs`, `devices/pit.rs`.

## A5.3 Umbau des Interrupt-Dispatchers in Rust
Der Interrupt-Dispatcher in Rust in der Datei `intdispatcher.rs` muss angepasst werden. Der Zugriff auf die globale Variable `INT_VECTORS` ist durch einen Mutex geschützt. Dies wird nun zum Problem, da wir aus der ISR des PITs einen Thread-Wechsel durchführen möchten. Dabei kehren wir vorerst nicht aus der ISR zurück, weswegen der Mutex auf `INT_VECTORS` nicht freigegeben würde. Das wiederum führt dazu, dass beim nächsten Interrupt eine Verklemmung eintritt. Um dieses Problem zu beheben, rufen wir in `pit::trigger()` vor jeder Thread-Umschaltung `intdispatcher::INT_VECTORS.force_unlock()` auf um das Lock mit Gewalt freizugeben.

Damit durch `force_unlock()` zu keiner Race Condition kommen kann, müssen in Ihrer Funktion `intdispatcher::register()` kurz die Interrupts gesperrt werden, wenn eine neue ISR registriert wird. Verwenden Sie hierfür die Funktionen `cpu::disable_int_nested()` und `cpu::enable_int_nested(param)`.

In folgender Datei muss Code implementiert werden: `kernel/interrupts/intdispatcher.rs`.

## A5.4 Threadumschaltung mithilfe des PIT
Nun soll die erzwungene Thread-Umschaltung aus der ISR des PITs realisiert werden. Fügen Sie dem Struct `SchedulerState` in `scheduler.rs` eine boolean Variable `initialized` hinzu. Diese soll anfangs auf `false` gesetzt sein, und in `Scheduler::scheduler()` auf `true` umgesetzt werden, direkt bevor der erste Thread gestartet wird.

In `yield_cpu()` soll nun mit der Variable `initialized` geprüft werden, ob der Scheduler bereits läuft und nur dann eine Thread-Umschaltung eingeleitet werden. Außerdem müssen wir auch hier sichergehen, dass wir kein Deadlock erzeugen, wenn wir `Scheduler::state` locken. Eine Thread-Umschaltung soll nur erfolgen, wenn das Lock mit `try_lock()` geholt werden kann.

In folgender Datei muss Code implementiert werden: `kernel/threads/scheduler.rs`.

## A5.5: Testanwendung mit Multithreading
Testen Sie das präemptive Multitasking mit Ihrer Zähler-Demo aus Aufgabe 4. Entfernen Sie jedoch den `yield_cpu()` Aufruf, da wir jetzt ja den Entzug der CPU überprüfen wollen. Zusätzlich soll noch ein weiterer Thread erzeugt werden der eine Melodie abspielt. Neben diesen beiden Threads soll zusätzlich der Fortschritt der Systemzeit im Interrupt ausgegeben werden, siehe nachstehende Abbildung (rechts oben).

Vermutlich werden Sie nun ein paar unerwartete Dinge feststellen:
 1. Hauptsächlich gibt nur der erste Thread seinen Zähler aus. Die anderen beiden kommen, wenn überhaupt, nur selten zum Zug.
 2. Sollte doch mal einer der anderen beiden Threads rechnen, kann es passieren, dass die Ausgabe mit `println!()` an der falschen Position auf dem Bildschirm geschieht.

Das erste Problem kommt daher, dass der erste Thread die meiste Zeit über das CGA-Lock hält. Er holt es einmal zum Setzen der Cursor-Position und einmal in `println!()`. Es gibt nur ein sehr kleines Zeitfenster, in dem das CGA-Lock frei ist. Nur wenn der PIT zufällig in diesem Zeitfenster eine Thread-Umschaltung veranlasst, kann einer der anderen Threads seinen Zähler ausgeben. Das passiert jedoch nur selten, so dass die anderen beiden Threads "verhungern". Bauen Sie testweise ein `pit::wait(100)` nach der Ausgabe ein, und es müssten nun alle drei Threads ihre Zähler ausgeben, da nun die Wahrscheinlichkeit, dass ein Thread-Wechsel stattfindet, während das CGA-Lock frei ist deutlich gesteigert wurde. Das ist jedoch eine sehr unschöne Lösung, da viel Rechenzeit verschwendet wird. Sinnvoller wäre es stattdessen, wenn ein Thread nach einer gewissen Anzahl an Schleifendurchläufen (z.B. 10) auch mal freiwillig die CPU abgibt.

Das zweite Problem entsteht, weil wir das CGA-Lock einmal holen um die Cursor-Position zu setzen. Anschließend wird es freigegeben und dann wieder von `println!()` geholt. Sollte eine Thread-Umschaltung genau dazwischen passieren, kommt es zu Inkonsistenzen bei der Cursor-Position: Ein Thread setzt den Cursor z.B. auf (10, 10), dann wird ihm die CPU entzogen und ein anderer Thread setzt den Cursor auf (10, 20) um dort seine Ausgabe zu machen. Wenn nun wieder der ursprüngliche Thread dran kommt, geht er davon aus, dass der Cursor weiterhin bei Position (10, 10) steht, was aber nicht mehr stimmt und die Ausgabe erfolgt an der falschen Position auf dem Bildschirm.
Um dieses Problem zu lösen wurden in `cga_print.rs` die zusätzlichen Makros `cga_print!()` und `cga_println!()` eingeführt. Diese funktionieren analog zu `print!()` und `println!()`, bekommen aber als ersten Parameter zusätzlich eine CGA-Referenz. Auf diese Weise können wir erst das CGA-Lock holen (`let mut cga = CGA.lock()`), dann die Cursor-Position setzen und schließlich mit der gelockten Referenz und `cga_println!()` unsere Ausgabe machen (z.B. `cga_println!(&cga, "Hello!")`);
Übernehmen Sie hierzu die `cga_print.rs` aus der Vorgabe und bauen die Änderungen in `cga.rs` in ihr System ein.


**Beispielausgabe des Testprogramms**

![MTHR](img/threads.png)


# Aufgabe 6: Synchronisierung

## Lernziele
1. Verstehen wie ein Spinlock sowie ein Guard funktioniert
2. Im Scheduler das Blockieren von Threads realisieren
3. Einen eigenen Mutex mit Warteschlange schreiben 


## A6.1: Umbau auf eigene Lock-Implementierung
In der Vorgabe finden sie die zwei Dateien `library/spinlock.rs` und `library/mutex.rs`, die sie in Ihr Projekt einbauen sollen. Sie sollen damit den bisher verwendeten Mutex aus der `spin`-Crate ersetzen. Dabei soll nachvollzogen werden können, wie Locks generell und speziell Guards in Rust funktionieren. Merh Details dazu finden Sie in den folgenden Aufgaben.

Kopieren Sie zunächst die beiden Dateien aus der Vorgabe in den Ordner `os/src/library` Ihres Projekts und ergänzen Sie die entsprechenden Zeilen in `os/src/library/mod.rs`. Ersetzen Sie nun in `cga.rs` den Mutex aus der `spin`-Crate durch den aus der Vorgabe. Die Mutex-API bleibt dabei gleich, es reich also oben in `devices/cga.rs` die Zeile `use spin::Mutex;` durch `use crate::library::mutex::Mutex;` auszutauschen.

Die Mutex-Implementierung aus der Vorgabe macht aktuell noch nichts und tut einfach bei jedem Aufruf von `lock()` und `try_lock()` so, als hätte man das Lock erfolgreich bekommen. Das heißt, dass Zugriffe auf den CGA-Bildschirm nun effektiv nicht mehr synchronisiert sind und mehrere mutable Referenzen auf `CGA` gleichzeitig existieren können. Wenn Sie nun einmal die Demo aus Aufgabe 5 starten, wird sich die Ausgabe der Zähler nicht mehr korrekt verhalten. Wo bisher jeder Thread seinen Zähler in einer eigenen Zeile ausgegeben hat, springt der Cursor nun wild zwischen den drei Zeilen herum und die Ausgabe geschieht kreuz und quer (ähnlich wie auf der unten stehenden Abbildung).

![aufgabe1.png](img/aufgabe1.png)

## A6.2: Implementierung eines einfachen Spinlocks
Implementieren Sie nun die leeren Methoden in `spinlock.rs`. Sie benötigen dafür die folgenden atomaren Methoden, die bereits in dem `Atomic` Struct aus der Rust `core` Bibliothek implementiert sind:
 - `store()`: Überschreibt den aktuellen Wert der atomaren Variable
 - `load()`: Liest den aktuellen Wert der atomaren Variable
 - `swap()`: Überschreibt den Wert der atomaren Variable und gibt den alten Wert zurück.
 
All diese Methoden kapseln atomare Operationen, die nicht unterbrochen werden können. Es ist daher nicht möglich, dass der Scheduler dazwischen "grätscht", während man eine der atomaren Operationen durchführt.

Das Prinzip eines Spinlocks ist simpel: In der `lock()`-Methode wird `true` in die Lock-Variable geschrieben. War der Wert vorher bereits `true`, so hält gerade ein anderer Thread das Lock. In diesem Fall wird in in einer Schleife erneut versucht das Lock zu bekommen, bis es klappt. War der Wert vorher `false`, wurde das Lock erfolgreich geholt und die Schleife kann verlassen werden.

Das dauerhafte Warten auf einen bestimmten Wert in einer Schleife nennt sich *Busy Waiting* (oder *Busy Polling*). Häufig möchte man dies vermeiden, da die CPU hier wertvolle Rechenzeit und auch Energie verschwendet. Wie wir das Busy Waiting vermeiden können, schauen wir uns in der nächsten Aufgabe mit der `Mutex`-Implementierung an. Um solche Busy Waiting Schleifen etwas zu optimieren, bietet x86 die Instruktion `pause` an. Diese soll am Ende eines jeden Schleifendurchlaufs ausgeführt werden um der CPU mitzuteilen, dass sie sich gerade in einer Busy Waiting Schleife befindet. Sie können diese Instruktion einfach mit einem einzeiligen `asm!()` Block in Ihren Rust-Code einbauen.

Testen Sie Ihren Spinlock indem Sie ihn zur Synchronisierung des CGA-Bildschirms nutzen. Dazu müssen sie einfach nur `use crate::library::mutex::Mutex;` durch `use crate::library::spinlock::Spinlock as Mutex;` ersetzen. Wenn alles klappt, sollten die drei Zähler-Threads nun wieder korrekt in ihre eigenen Zeilen schreiben.

Schauen Sie sich außerdem die Implementierung des Structs `SpinlockGuard` einmal genauer an. Dieses kapselt eine Referenz auf die synchronisierte Datenstruktur und ermöglicht durch Implementierung der `Deref` und `DerefMut` Traits einen transparent Zugriff auf diese. Die `drop()` Methode sorgt außerdem dafür, dass das Spinlock automatisch freigegeben wird, sobald das Scope in welchem das Lock geholt (und somit die Guard-Instanz angelegt) wurde, verlassen wird.

In den folgenden Dateien muss Code implementiert werden: `library/spinlock.rs` und `devices/cga.rs`. 

![aufgabe2.png](img/aufgabe2.png)

## A6.3: Mutex mit Warteschlange
Nun soll in `mutex.rs` ein Mutex mit einer Warteschlange implementiert werden. Falls ein Thread `lock()` aufruft und die Sperre nicht frei ist, soll der Thread blockiert werden. In diesem Fall soll der blockierte Thread in die Warteschlange des Mutex eingefügt und auf einen anderen Thread umgeschaltet werden. 

Wenn ein Thread die Sperre freigibt, also durch das Freigeben des Guards `unlock()` aufgerufen wird, soll geprüft werden, ob die Warteschlange nicht leer ist. Falls ein Thread dort vorhanden ist, soll dieser entfernt und in die Ready-Queue des Schedulers eingefügt werden. Dass heißt, dass nicht direkt auf den Thread umgeschaltet wird, der deblockiert wird.

In der Vorgabe finden Sie zwei neue Methoden in `scheduler.rs`, die Sie für den Mutex zuerst implementieren müssen. Die Methode `prepare_block()` schaltet die Interrupts ab und gibt anschließend den aktiven Thread, sowie den Rückgabewert von `cpu::disable_ints_nested()` zurück. Die Interrupts müssen abgeschaltet werden, da bei einem Thread-Wechsel zum jetzigen Zeitpunkt der aktuelle Thread verloren wäre. Er wurde aus dem Scheduler ausgehakt und es würde somit nie wieder zu ihm zurück gekehrt werden.

Die Methode `switch_from_blocked_thread()` nimmt einen Pointer auf den blockierten Thread entgegen, sowie den Rückgabewert von `cpu::disable_int_nested()` aus dem vorhergehenden Aufruf von `prepare_block()` und wechselt zum nächsten Thread. Die Interrupts werden durch den Thread-Wechsel automatisch wieder aktiviert (bzw. ist das abhängig vom Flags-Register des nächsten Threads). Nach `Thread::switch()` müssen wir sie jedoch mit Hilfe von `cpu::enable_int_nested()` wieder einschalten. Zu diesem Zeitpunkt wurde der blockierte Thread wieder fortgesetzt.

Implementieren Sie nun die leeren Funktionen in `mutex.rs`. Die Methode `lock()` soll hierbei den aktuellen Thread blockieren und in die Warteschlage des Mutex einhängen. In `unlock()` soll ein Thread aus der Warteschlange des Mutex ausgereiht und mit `scheduler::ready()` wieder in den Scheduler eingehangen werden. In `lock()` müssen Sie folgende Dinge beachten:
 - Wenn der Scheduler noch nicht initialisiert wurde, kann natürlich kein Thread blockiert werden. In dem Fall soll sich der Mutex einfach wie ein Spinlock verhalten.
 - Zum Einreihen des blockierten Threads in die Warteschlange des Mutex muss diese gelockt werden (sie ist ja bereits in der Vorgabe durch ein Spinlock geschützt). Da wir jedoch aus `scheduler::switch_from_blocked_thread()` erstmal nicht zurückkehren, bleibt die Warteschlage gelockt und beim nächsten Aufruf von `mutex::lock()` würde es zu einer Verklemmung kommen. Um das zu vermeiden sollte das Manipulieren der Warteschlange in einem eigenen Scope geschehen, so dass beim Verlassen dieses Scopes die Warteschlagen wieder freigegeben wird.

Bevor wir den Mutex in `cga.rs` verwenden können, müssen wir noch eine Sache beachten: Beim Aufruf von `lock()` oder `unlock()` werden jeweils drei verschiedene andere Locks geholt: Das Lock der Mutex-Warteschlange, das Lock des Schedulers (zum Blockieren/Deblockieren des Threads) und das Lock des Allokators (zum Einreihen/Ausreihen aus der Mutex-Warteschlange). Sollte eines dieser Locks gerade gehalten werden, während in `pit::trigger()` das Spinner-Symboler ausgegeben wird, führt dies zu einer Verklemmung im Interrupt Handler. Um dieses Problem zu Lösen wurde im Scheduler die neue Methode `is_locked()` eingeführt. Prüfen Sie in `pit::trigger()` ob der Scheduler, der Allokator und die Warteschlange des CGA-Mutex gerade frei sind, bevor Sie `CGA.try_lock()` aufrufen und das Spinner-Symbol aktualisieren. Das wirkt nach sehr viel Aufwand nur für die Aktualisierung eines Symbols, verdeutlicht jedoch, wie behutsam man Code in Interupt-Handlern implementieren muss.

In den folgenden Dateien muss Code implementiert werden: `library/mutex.rs`, `kernel/threads/scheduler.rs`, `devices/pit.rs`. 

## A6.4: Vergleich aller Lösungen

Lassen Sie nun in Ihrer Demo alle drei Threads bis zu einem bestimmten Wert zählen (z.B. 100000) und messen Sie dabei in jedem Thread die benötigte Zeit. Vergleichen Sie die neue Mutex-Implementierung mit dem Spinlock und dem Mutex aus der `spin`-Crate. Wie viel schneller rechnen die Threads mit unserer neuen Mutex-Implementierung?

Sie können nun *FAST* überall wo Ihr Betriebssytem den Mutex aus der `spin`-Crate verwendet stattdessen unseren neuen Mutex benutzen. Jedoch sollte `INT_VECTORS` in `kernel/interrupts/intdispatcher.rs` besser mit einem Spinlock synchronisiert werden, da hier eine ähnliche Problematik wie bei `pit:trigger()` besteht. Außerdem sollte der `SchedulerState` in `kernel/threads/scheduler.rs` ebenfalls durch ein Spinlock gesichert sein, da der Mutex selbst ja auch auf den Scheduler zugreift und es so zu rekursiven Aufrufen und Verklemmungen kommt.

Das System sollte weiterhin fehlerfrei funktionieren.


# Aufgabe 7: Eine eigene BS-Erweiterung / Anwendung

## Lernziele
1. Eine Anwendung schreiben
2. Alternativ eine Betriebssystem-Komponente entwickeln

## Mögliche Themenrichtungen
- Grafikdemo (multithreaded)
- Retro-Spiel (z.B. Snake, Pacman, ...)
- einfache Shell (Beispiele für Befehle: clear, time, meminfo, ...) 
- Scheduler mit Prioriäten (mit einer Demo)

## Vorgabe
Die Vorgabe umfasst einige Dateien, um einen Grafikmodus nutzen zu können. Außerdem enthält sie Code, mit dem der PCI-Bus nach Geräten gescannt wird.

### Grafikfunktionen 
Vorhanden sind nur sehr grundlegende Grafik-Funktionen, inkl. einer Text-Ausgabe mit einer Schriftart. Weitere Funktionen sollen je nach Anwendung ergänzt werden. 

Ob das System im Grafik- oder Textmodus startet wird in `boot/boot.asm`durch die die Konstante `TEXT_MODE` festgelegt. Wenn diese Konstante aukommentiert wird, so schaltet `grub` direkt in den Grafikmodus (800x600 mit 32 Bit pro Pixel). Eine alternative Grafikauflösung kann durch die Konstanten `MULTIBOOT_GRAPHICS_*` in  `boot/boot.asm` eingestellt werden. Mögliche Auflösungen sollten sich an dem VESA-Standard orientieren, siehe hier: [VESA](https://en.wikipedia.org/wiki/VESA_BIOS_Extensions). Es sollte immer ein Modus mit 4 Byte pro Pixel als Farbtiefe verwendet werden.

Da jeder Pixel von der CPU einzeln im Grafikspeicher gesetzt werden muss, kann das Zeichnen komplexer Szenen schnell langsam werden. Eine deutliche Performance-Steigerung erhält man mit einem Release-Build, bei dem Compiler-Optimierungen aktiviert sind. Allerdings kann dann nicht mehr debuggt werden. Einen solchen Build erstellt man mit dem folgenden Befehl:
```bash
cargo make --profile production qemu
```

Die Textausgabe über CGA funktioniert nicht im Grafikmodus! Ein Beispiel für die Textausgabe befindet sich in der Vorgabe in `user/aufgabe7/graphic_demo.rs`, siehe auch nachstehendes Bild.

Folgende Dateien sind für die Grafik-Unterstützung in der Vorgabe:
- `startup.rs`: Bekommt jetzt eine Mutltiboot-Referenz als Parameter
- `devices/lfb.rs`: Zeichenfunktionen im Grafikmodus
- `devices/font_8x8.rs`: Bitmap-Font für die Textausgabe im Grafikmodus
- `kernel/cpu.rs`: Erweiterte IO-Port Befehle zum Schreiben von 16-/32-Bit Werten.
- `kernel/multiboot.rs`: Struct-Definition für Bootloader-Daten, die gemäß des [Multiboot](https://www.gnu.org/software/grub/manual/multiboot/multiboot.html)-Protokolls an den Kernel übergen werden.
- `user/aufgabe7/graphic_demo.rs`: Kleine Grafikdemo (siehe Bild unten)
- `user/aufgabe7/bmp_hhu.rs`: HHU-Logo als Bitmap
- `cbmp2rs.c`: Kleines C-Programm zum Konvertieren von Bildern, gespeichert von GIMP als C-Source, siehe auch `Graphics-Rust.pdf`.

**Beispielausgabe der Grafikdemo**

![lfb](img/lfb.png)

### PCI
Mit dem Quellcode in `devices/pci.rs` kann der PCI-Bus nach vorhandenen Geräten abgesucht werden. In `startup.rs` ist ein kleines Beispiel, welches nach einer Realtek RTL8139 Netzwerkkarte sucht und deren MAC-Adresse ausliest. Dies ist nur als kleine Demo gedacht, für diejenigen, die sich für Treiber-Programmierung interessieren und auch mal mit komplexeren Geräten als z.B. dem PIT arbeiten möchten. So etwas wäre auch als Abgabe denkbar. Dann muss allerdings je nach ausgesuchter Hardware kein voll funktionsfähiger Treiber implementiert werden, da dies den Aufwand der Aufgabe übersteigen würde. Gut dokumentierte PCI-Geräte sind z.B. die Realtek RTL8139 Netzwerkkarte oder ein IDE Festplatten Controller. Dazu findet man auch Artikel im OSDev-Wiki.

Wenn die Vorgabe richtig eingebaut wurde, sollten beim Booten folgenden Meldungen über die serielle Schnittstelle ausgegeben werden:

```
Scanning PCI bus
Found PCI device 8086:1237
Found PCI device 8086:7000
Found PCI device 8086:7010
Found PCI device 8086:7113
Found PCI device 1234:1111
Found PCI device 10ec:8139
Found Realtek RTL8139 network controller
RTL8139 I/O base address: 0xc000
MAC address: [52, 54, 0, 12, 34, 56]
```

Folgende Dateien sind für die PCI-Unterstützung in der Vorgabe:
- `Makefile.toml`: Enthält kleine Änderungen um QEMU mit einer emulierten einer Realtek RTL8139 Netzwerkkarte zu starten
- `startup.rs`: Sucht den PCI-Bus nach einer Realtek RTL8139 Netzwerkkarte ab
- `devices/pci.rs`: Treiber für den PCI-Bus


# Aufgabe 8: User Mode Threads (Isolation und Schutz - Aufgabe 1)

Dies ist die erste Aufgabe des zweiten Masterkurses zum Thema "Betriebssystementwicklung". Falls Sie den ersten Kurs bereits gehört haben, können Sie Ihr vorhandenes Betriebssystem weiter entwickeln. Sollten Sie in diesem Kurs neu dazu gekommen sein, oder generell lieber mit einem "frischen" OS starten, finden Sie im Branch [vorgabe-8](https://github.com/hhu-bsinfo/hhuTOSr/tree/vorgabe-8) eine etwas abgespeckte Version (ohne PC Speaker, Mutex und Demos) unserer Lösung für den ersten Kurs. Diese können Sie als Basis für den Kurs "Isolation und Schutz in Betriebssystemen" verwenden.

*Achtung: Der Branch wird 14 Tage nach Beginn der Vorlesungszeit offline genommen.*

## Lernziele
1. Verstehen wie bei der x86-Architektur Anwendungscode im User Mode (Ring 3) ausgeführt wird und wie man damit verhindert, dass ein User-Thread privilegierte Instruktionen ausführt oder auf Ports zugreift.

## A8.1: Global Descriptor Table (GDT)
Zunächst benötigen wir in der vorhandenen GDT (liegt bei der Marke `_gdt`) zwei weitere Einträge: Einen für Ring 3 Code und einen für Ring 3 Daten. Das Limit der GDT muss ebenfalls angepasst werden, da wir die Anzahl der Einträge verändern. Das Limit liegt bei der Marke `_gdt_descriptor`.

Alle Arbeiten in dieser Aufgabe sind in Assembler in der Datei `boot.asm` durchzuführen.

Die Änderungen lassen sich zunächst nicht wirklich testen. Jedoch sollte das OS weiterhin ordungsgemäß booten, nachdem Sie die GDT angepasst.

*Wichtige Information für diese Aufgabe finden Sie in Intel Software Developer’s Manual Volume 3 in Kapitel 3.4.5 Segment Descriptors.*

## A8.2: Task State Segment (TSS)
Als nächstes benötigen wir ein TSS, damit der Prozessor den Kernel-Stack findet, wenn beispielweise ein Thread im Ring 3 durch einen Interrupt unterbrochen wird. Hierfür nutz der Prozessor im TSS den Eintrag `rsp0` (Stack-Zeiger für den Ring 0). In der Vorgabe befindet sich bereits ein TSS (ohne IO-Bitmap; lassen wir absichtlich weg, dadurch sind jegliche Portzugriffe im User-Mode unterbunden) an der Marke `_tss`. Da wir nur einen CPU-Kern nutzen, reicht ein einziges TSS für unser Betriebssystem. 

Damit das TSS genutzt werden kann muss ein TSS-Deskriptor (TSSD) in der GDT hinzugefügt werden. Bis auf die Basis-Adresse können alle Informationen im TSSD direkt statisch eingetragen werden. Zusätzlich muss nochmals das Limit in `_gdt_descriptor` angepasst werden (wie bei Aufgabe 1).

*Achtung: Der TSSD-Eintrag hat die doppelte Größe!*

Die Basisadresse des TSS muss zur Laufzeit mithilfe der Funktion `_tss_set_base_address()` in den TSSD eingetragen werden. Diese Funktion wird in der Vorgabe bereits an der richtigen Stelle aufgerufen (nur einmal beim Bootvorgang). Um einfach an die Basisadresse des TSSD-Eintrags zu gelangen, definieren Sie sich in der GDT vor dem TSSD-Eintrag in Assembler eine Marke (engl. Label).

Nun benötigen wir noch eine Funktion zum Setzen des Kernel-Stack-Zeigers (`rsp0`) in unserem TSS. Hierfür gibt es in der Vorgabe die Marke `_tss_set_rsp0()`. Diese Funktion wird bereits an der richtigen Stelle in der Vorgabe aufgerufen und hierbei wird als Parameter der Zeiger auf den Stack übergeben, welcher aktuell verwendet wird. Später nutzen wir diese Funktion in Rust, um bei jedem Thread-Wechsel den rsp0 (Kernel-Stack-Pointer) im TSS zu sichern.

Zuletzt muss noch das TSS-Register (TSSR) mit dem Befehl `ltr` geladen werden. Die richtige Stelle ist in `boot.asm` durch einen Kommentar markiert.

Der Speicherplatz des initialen Stacks ist an der Marke `_init_stack` definiert. Er wird aber, sobald der Scheduler läuft, nicht mehr verwendet. Alle Stacks (User- und Kernel-Mode) aller Threads liegen dann im Heap und werden mit unserem Allokator alloziert.

Alle Arbeiten in dieser Aufgabe sind in Assembler in der Datei `boot.asm` durchzuführen.

*Wichtige Information für diese Aufgabe finden Sie in Intel Software Developer’s Manual Volume 3 in Kapitel 9.2 Task Management Data Structures*

## A8.3: Threads in Ring 3 starten
User-Level-Threads (laufen im Ring 3) benötigen immer zwei Stacks, einen für den User- und einen für den Kernel-Mode. Der Einfachheit halber allozieren wir immer zwei Stacks, auch für reine Kernel-Threads. Hierfür muss die `struct Thread` in `thread.rs` angepasst werden. Außerdem soll die Funktion `new_user_thread()` einen Thread zurückgeben, bei dem die Boolean-Flag `is_kernel_thread` auf `false` gesetzt ist. Die Funktion `new()` wird in `new_kernel_thread()` umbenannt, die Funktion `kickoff()` in `kickoff_kernel_thread()`  und `prepare_stack()` in `prepare_kernel_stack()`.

Die Thread-Umschaltung ist in `thread_switch()` fertig vorgegeben. Neben dem Sichern und Wieder­her­stellen der Register muss auch der Kernel-Stack im TSS umgeschaltet werden. Dafür wird der Parameter `next_stack_end` verwendet.  Der User-Level-Stack-Zeiger muss nicht extra verwaltet werden, da er automatisch durch die Hardware bei einem Thread-Wechsel gesichert wird. Der Thread-Wechsel aus einem User-Thread erfolgt nur aus dem Timer-Interrupt heraus und dafür wechselt der Prozessor automatisch von Ring 3 nach Ring 0 und sichert den Stack-Zeiger des unterbrochenen User-Level Threads auf dem Kernel-Stack.

*Achtung: Das heißt, User-Threads dürfen aktuell nicht `Scheduler::yield_cpu()` und `Scheduler::exit()` aufrufen. Daher kann auch `pit::wait()` noch nicht im User Mode verwendet werden. Außerdem klappt auch unser Mutex aus Aufgabe 6 nicht im User Mode. Sollten Sie mit Ihrem Betriebssystem aus dem ersten Kurs weiterarbeiten, müssen Sie für die CGA-Instanz wieder ein klassisches Spinlock verwenden, statt unseres Mutex mit Warteschlange.*

Der vorhandene Thread-Start mithilfe von `prepare_kernel_stack()` und `start()` bleibt un­ver­ändert. Damit startet jeder Thread zunächst immer im Ring 0. In `kickoff_kernel_thread()` muss der Kernel-Stack im TSS gesetzt werden, mithilfe der Funktion _tss_set_rsp0.

Als nächstes muss die Funktion `switch_to_usermode()` in thread.rs implementiert werden. Sie soll die Assembler-Funktion `thread_user_start()` verwenden, um mit `iretq` einen Ring-Wechsel durchzuführen (anders geht das nicht). In `switch_to_usermode()` muss ein Stackframe gebaut werden, wie er bei einem Interrupt mit Privilegienwechsel vorliegt. Für SS und CS sind die ent­sprechenden User-Mode-Einträge in der GDT zu verwenden (siehe Aufgabe 1).

*Achtung: für die Stack­einträge CS und SS muss RPL = 3 gesetzt werden! Und der Stackframe darf keinen Error-Code beinhalten.*

Hierdurch sollten wir dann in `kickoff_user_thread()` „landen“ und der Prozessor sollte dann in Ring 3 sein.

Am besten sollte das Umschalten in den Ring 3 vorerst nur mit einem Thread getestet werden, beispielsweise dem Idle-Thread, also versuchen den Idle-Thread im Ring 3 laufen zu lassen. Später soll der Idle-Thread natürlich im Ring 0 ausgeführt werden.

Hinweis: wie kann man erkennen, ob man im User-Mode ist?
 - Einen Port-Befehl ausführen, dieser sollte eine General Protection Fault (GPF) auslösen.
 - Oder mit dem Debugger einen Breakpoint auf `kickoff_user_thread()` setzen und prüfen, ob RPL = 3 im CS-Register steht.

Alle Arbeiten in dieser Aufgabe sind in Rust in der Datei `thread.rs` durchzuführen.

*Wichtige Information für diese Aufgabe finden Sie in Intel Software Developer’s Manual Volume 3 in Kapitel 7.12 Exception and Interrupt Handling*

### Testszenario
In der Datei `user/aufgabe8/user_threads.rs` finden Sie bereits ein fertiges Testprogramm, welches einen Kernel- und einen User-Thread startet. Jeder Thread füllt eine eigene Zeile auf dem Bildschirm nach und nach mit dem Buchstaben `K` oder `U`. Für ein erweitertes Testszenario können Sie mehrer User-Threads starten. Prüfen Sie außerdem mit dem Debugger ob der Kernel-Thread auch nach mehreren Schleifendurchläufen noch in Ring 0 und der User-Thread in Ring 3 läuft (ablesbar am CS-Register).

| ![User-Thread Test 1](img/user-threads1.png) |
|:--:|
| *Ein Kernel- und ein User-Thread* |

| ![User-Thread Test 2](img/user-threads2.png) |
|:--:|
| *Zwei Kernel- und drei User-Threads* |

# Aufgabe 9: Systemaufrufe (Isolation und Schutz - Aufgabe 2)

## Lernziele
1. Verstehen wie Systemaufrufe funktionieren, um kontrolliert von Ring 3 in Ring 0 zu schalten und somit in kontrollierter Weise Funktionen des Kerns aufzurufen, welche wiederum privilegierte Instruktionen verwenden dürfen.

## A9.1: Interrupt Descriptor Table (IDT)
Ein sehr grundlegender Ansatz Systemaufrufe durchzuführen besteht darin, einen Software-Interrupt auszulösen (wie beispielsweise der Vektor 0x80 in Linux), um in den Ring 0 zu wechseln. Hierzu müssen wir einen Eintrag in der IDT verändern.

Aktuell ist unsere IDT komplett (256 Einträge) mit Interrupt Gates befüllt, die alle DPL=0 gesetzt haben und auf die Funktion `int_disp()` verweisen. Damit wir einen Systemaufruf mithilfe eines Software-Interrupts realisieren können benötigen wir in der IDT ein Trap-Gate mit DPL=3 (damit dieses aus dem User-Mode heraus zugegriffen werden darf). Wir überschreiben hierzu einfach den existierenden Eintrag an der Index-Position 0x80 (=Vektornummer)

Zum Erzeugen eines Trap-Gate Eintrags soll in `idt.rs` ein neue Funktion `new_trap_gate()` für das Struct `IdtEntry` implementiert werden.

*Achtung: Die bereits existierende Funktion `IdtEntry::new()` wurde in der Vorgabe zu `new_interrupt_gate()` umbenannt.*

Anschließend muss noch ein Trap Gate in der IDT installiert werden. Setzen Sie hierfür mit Hilfe der Funktion `IdtEntry::syscall_gate()` den Vektor 0x80 der IDT so, dass er auf die Funktion `syscall_dispatcher::syscall_disp()` verweist.

*Wichtige Information für diese Aufgabe finden Sie in Intel Software Developer’s Manual Volume 3 in Kapitel 7.11 IDT Descriptors.*

## A9.2: Syscall-Handler
Als nächstes muss der Syscall-Handler `syscall_disp()` in `syscall_dispatcher.rs` programmiert werden. Ergänzen Sie hierzu den fehlenden Code in `syscall_disp()`. Ein Systemaufruf soll folgendermaßen ablaufen:
1. Eine Anwendung schreibt die Nummer des gewünschten Systemaufrufs in das Register `rax` und löst dann mit `int 0x80` einen Software-Interrupt aus.
2. Durch unseren neuen IDT-Eintrag wird direkt die Funktion `syscall_disp()` angesprungen (dabei wurde von Ring 3 in Ring 0 gewechselt). Diese muss nun alle Register (außer `rax`) sichern, anschließend den Wert in `rax` nutzen um den richtigen Eintrag der Tabelle der Systemaufrufe zu finden und schließlich die entsprechende Funktion aus der Tabelle aufrufen.
3. Nun müssen alle Register (außer `rax`) wiederhergestellt werden und anschließend wird mit `iretq` in Ring 3 zur Anwendung zurückgekehrt. Der Rückgabewert des Systemaufrufs bleibt dabei in `rax` liegen und kann von der Anwendung verarbeitet werden.

Die User Mode API für **alle** Systemaufrufe befindet sich in der Datei `user_api.rs`. Hier ist bereits der Systemaufruf `usr_hello_world()` vorgegeben, welcher "Hello, world!" auf der seriellen Schnittstelle ausgibt.

Das kernelseitige Gegenstück zu `usr_hello_world()` heißt `sys_hello_world()` und befindet sich im Ordner `syscalls/functions/`. Hier sollen auch alle anderen Systemaufrufe implemnetiert werden Es bietet sich an, thematisch zusammenpassende Funktionen in Dateien zu gruppieren. Das bleibt jedoch Ihnen überlassen.

Zum Testen der Systemaufrufe finden Sie in der Vorgabe die Datei `user/aufgabe-9/syscall_demo.rs`, welche einen neuen User-Thread erzeugt, der die Funktion `syscall_test_thread()` ausführt. Rufen Sie hier `usr_hello_world()` auf um Ihre Implementierung zu testen. Wenn alles funktioniert sollte die Textausgabe der Funktion `sys_hello_world()`  auf der seriellen Schnittstelle zu sehen sein. *Wichtig: Hierfür werden Portbefehle verwendet, die im User-Mode nicht erlaubt sind. Durch den Systemaufruf wechseln wir jedoch in den Ring 0 können dann den Code ausführen.*

## A9.3: Weitere Systemaufrufe (mit Parametern)
Nachdem der erste Systemaufruf erfolgreich getestet wurde, sollen nun die nachfolgenden implementiert werden:
- `usr_thread_yield()` -> `sys_thread_yield()`: Gibt freiwillig die CPU ab und wechselt zum nächsten Thread.
- `usr_thread_exit()` -> `sys_thread_exit()`: Beendet den aktuell laufenden Thread -> In `threads.rs` kann anschließend die Funktion `kickoff_user_thread()` so geändert werden, dass sie `usr_thread_exit()` aufruft, statt am Ende eines User Threads in einer Dauerschleife hängen zu bleiben.
- `usr_thread_get_id() -> usize` -> `sys_thread_get_id() -> u64`: Gibt die ID des aktuell laufenden Threads zurück.
- `usr_get_system_time() -> usize` -> `sys_get_system_time() -> u64`: Gibt die Systemzeit in Millisekunden zurück.
- `usr_print(msg: &str)` -> `sys_print(buffer: *const u8, len: usize)`: Gibt eine Nachricht an der aktuellen Position auf dem CGA-Bildschirm aus.
- `usr_get_char() -> char` -> `sys_get_char() -> u64`: Gibt den nächsten Buchstaben aus dem Buffer des Tastatur zurück.
  Für jede Funktion muss außerdem ein Eintrag in `user_api::SyscallFunction` angelegt werden.

Um Systemaufrufe mit Parametern zu unterstützen, müssen zusätzlich zu `syscall0(syscall: SyscallFunction)` auch noch Funktionen wie `syscall1(syscall: SyscallFunction, arg1: u64)`, `syscall2(syscall: SyscallFunction, arg1: u64, arg2: u64)`, etc. angelegt werden um bis zu fünf Parameter zu unterstützen. Die Parameter sollen für die Systemaufrufe in den Registern entsprechend der `System V AMD64 ABI` übergeben werden.

*Wichtige Information für die Parameterübergabe in Registern finden Sie in System V Application Binary Interface AMD64 Architecture Processor Supplement in Kapitel 3.2.3 Parameter Passing.*

Testen Sie Ihre Systemaufrufe in `syscall_demo::syscall_test_thread()` folgendermaßen: Zu Beginn soll der aktuelle Thread-ID auf dem CGA-Bildschirm ausgegeben werden. Anschließend sollen in einer Schleife Zeichen von der Tastatur eingelesen werden. Immer wenn Enter gedrückt wird, soll die eingegebene Zeile, sowie die aktuelle Systemzeit auf dem CGA-Bildschirm ausgegeben werden.

![System Call Test](img/syscalls.png)

## A9.4: Schnelle Systemaufrufe (optional)
Systemaufrufe über ein Trap-Gate sind nicht sehr schnell. Daher wurden schnellere Varianten eingeführt: `sysenter`/`sysexit` (Intel) und `syscall`/`sysret` (AMD). Aus Gründen der Kompatibilität verwenden x86_64-Systeme letztere Variante. Um diese schnellere Variante zu nutzen muss die GDT dem geforderten Aufbau entsprechen.

Argumente können in den Registern übergeben werden, wie beim Interrupt-basierten Systemaufruf, aber es ist zu beachten, dass das Register RCX (4. Parameter) auch von der `syscall` Instruktion genutzt wird. Als Work-Around kann der 4. Parameter in einem anderen Register übergeben werden.

Als Präfix für die schnellen Systemaufrufe bietet sich beispielsweise `fast_` an.


# Aufgabe 10: Speicherverwaltung für physikalischen Speicher (Isolation und Schutz - Aufgabe 3)

## Lernziele
1. Eine Speicherverwaltung für physikalische Kacheln (Page Frames) implementieren.

## A10.1: Verfügbaren physikalischen Speicher ermitteln
Bevor das Paging implementiert wird, muss zunächst ermittelt werden, wie viel und wo nutzbarer physikalischer Speicher vorhanden ist. Die Lösung ist in `multiboot.rs` in der Funktion `init_phys_memory_allocator()` bereits vorhanden, soll aber gelesen und verstanden werden.

Die notwendigen Informationen bekommen wir von Multiboot in Form von `mmap`-Einträgen. Diese Einträge beschreiben jeweils einen zusammenhängenden Block physikalischen Speichers, der entweder reserviert oder verfügbar sein kann. Die als verfügbar markierten Speicherbereiche werden in unsere Speicherverwaltung für physikalischen Speicher via `PfListAllocator::free_block()` eingefügt. Wir müssen jedoch beachten, dass der Kernel-Code vom Bootloader nicht als reserviert markiert wird. Unser Kernel-Image wird an die Adresse 1 MiB geladen. Der Linker erzeugt die Symbole `___KERNEL_DATA_START__` und `___KERNEL_DATA_END__` um diesen Speicherbereich zu markieren. Außerdem wird der Bereich von 0 bis 1 MiB teilweise vom BIOS verwendet (Der Grafikspeicher liegt auch hier), weshalb wir ihn als nicht nutzbar betrachten. Der für uns nutzbare physikalische Speicher beginnt also direkt hinter dem Kernel Image.

Fügen Sie den folgenden Code zu Beginn ihrer `startup()`-Funktion in `startup.rs` ein, um die physikalische Speicherverwaltung mit den freien Speicherblöcken aus der Multiboot Memory Map zu initialisieren:
```
// Copy multiboot into on stack, because it lies in physical memory that might get reused after initializing the physical memory allocator
let multiboot_info = *multiboot_info;

kprintln!("Initializing physical memory allocator");
multiboot_info.init_phys_memory_allocator();
```

*Information zu Multiboot, insbesondere den mmap-Einträgen finden Sie hier:* https://www.gnu.org/software/grub/manual/multiboot/multiboot.html

## A10.2: Ein Allokator für Page Frames
Der verfügbare physikalische Speicher muss in 4 KiB Kacheln (Page Frames) verwaltet werden. Hierfür wird nun ein Page-Frame-Allokator benötigt. Als Basis empfiehlt sich der vorhandene Heap-Allokator (in list.rs), der Code muss natürlich angepasst werden. Wir verketten also die freien Page-Frames, wobei ein Block aus mehreren aufeinanderfolgenden Page-Frames bestehen kann. Die Metadaten für freie Blöcke schreiben wir direkt in die freien Page-Frames. Für die belegten Page-Frames be­nötigen wir keine Metadaten, da es nur 4 KiB Page-Frames gibt und jeder Page-Frame von einem Seiten­tabellen-Eintrag referenziert wird, sobald wir Paging implementiert haben.

Beim Freigeben eines Speicherblocks soll dieser **sortiert** in die Liste eingefügt werden. Falls ein freigegebener Block an seinen Vorgänger oder Nachfolger angrenzt, soll er mit diesem (oder falls möglich beiden) verschmolzen werden. So beugen wir einer Fragmentierung des Speichers in viele kleine Blöcke vor.

Schreiben sie eine Testfunktion, die die Korrektheit ihres Allokators überprüft. Hierfür ist es nützlich eine `dump()` Funktion zu implementieren, welche die gesamte Freispeicherliste auf dem Bildschirm oder seriellen Port ausgibt.

*Wichtige Hinweise:*
- *Der Allokator darf keinesfalls einen reservierten Page-Frame erneut vergeben! Bevor Sie fortfahren, sollten Sie die Korrektheit Ihres Page-Frame-Allokators testen! `assert!()` ist hier sehr nützlich.*
- *Ferner empfiehlt es sich Page-Frames beim Allozieren mit 0 zu initialisieren. So können später Pointer-Fehler einfacher erkannt werden, da dann ein Null-Pointer-Zugriff mithilfe des Pagings erkannt werden kann.*

## A10.3: Kernel Heap
Nun soll noch die Initialisierung des bestehenden Allokators für den Heap in `startup.rs` angepasst werden. Statt an einer festen Adresse einfach freien Speicher zu vermuten, soll nun freier Speicher vom Page-Frame-Allokator für den Heap angefordert werden. Nun sollte alles weiterhin funktionieren. Die Stacks werden vorerst noch im Kernel-Heap alloziert. Dies ändert sich mit dem nächsten Aufgabenblatt, wenn wir Paging einführen.


# Aufgabe 11: Paging (Isolation und Schutz - Aufgabe 4)

## Lernziele
1. Eine grundlegende Paging-Funktionalität implementieren.
2. Threads in eigenen Adressräumen mit isolierten User Stacks starten.

## A11.1: Seitentabellen für den Kernel
Wir verwenden ein vier-stufiges Paging mit ausschließlich 4 KiB Seiten (keine 2 MiB Seiten). Der gesamte physikalische Adressraum soll im Kernel 1:1 gemappt werden. Dadurch kann der Kernel immer alle physikalischen Adressen zugreifen. Dafür müssen wir zunächst die höchste physikalische Adresse ermitteln. In der Vorgabe wird der `PfListAllocator` in `frames.rs` um eine neue Variable `max_addr` erweitert. Passen Sie Ihre `free_block()`-Methode so an, dass sie bei jedem eingefügten Block prüft, ob seine Endaddresse größer ist als `max_addr` und `max_addr` entsprechend aktualisiert.

Nun soll in `pages.rs` die Funktion `PageTable::map(&mut self, virt_addr: u64, num_pages: usize, kernel: bool)` implementiert werden, die physikalischen Speicher in einen Addressraum einblendet (es empfiehlt sich eine rekursive Lösung). Ist der Parameter `kernel` auf `true` gesetzt, soll ein 1:1-Mapping erstellt werden. Die gegebene Addresse `virt_addr` ist also gleich der physikalischen Adresse, die auf der untersten Ebene in die Page Tables eingetragen wird. Für die Seitentabellen müssen dabei Page Frames alloziert werden, aber auf der untersten Ebene nicht, da hier der bestehende physikalische Speicher nur „gemappt“ wird. Der Fall, dass `kernel` auf `false` gesetzt ist, wird erst in Aufgabe 11.2 relevant.

Bezüglich der Seitentabelleneinträge lassen wir vorerst alle Einträge im Ring 3 zugreifbar, löschen also nicht das User-Bit. Das ist noch notwendig, damit wir den Code im Ring 3 ausführen können, wird aber in einem späteren Übungsblatt abgeschafft. Zudem setzen wir alle Seiten auf schreibbar und sofern mit Page-Frames unterlegt auf „Präsent“. Um andere mögliche Bits in den Seitentabellen­einträgen, wie Caching, No-Execute, Protection Keys, etc. kümmern wir uns nicht.

Die erste Seite (Adresse 0) sollte auf nicht-Präsent gesetzt werden, um Nullpointer-Zugriffe abfangen und er­kennen zu können.

Die Funktion `init_kernel_tables()` nutzt `PageTable::map()` um ein Kernel Mapping in einem Adressraum einzurichten. Testen Sie Ihre Implementierung zunächst ohne User Threads und Interrupts, indem Sie `init_kernel_tables()` in `startup.rs` aufrufen und den Rückgabewert in das CR3-Register schreiben. Dadurch wird der neu erstellte Adressraum geladen. Wenn etwas beim Anlegen der Page Tables nicht passt, wird Ihr System hier sehr wahrscheinlich abstürzen. Es empfiehlt sich dann, das System mit einem Debugger vor dem Setzen des CR3-Registers anzuhalten, und sich die Page Tables im Speicher anzuschauen.

Wenn das funktioniert sollte der Nullpointer-Zugriff geprüft werden. Es sollte ein Page-Fault auf­treten (Interrupt 14) und die Adresse der Instruktion die den Page-Fault ausgelöst hat steht in CR2. Schreiben Sie einen Page Fault Handler, der die Adresse ausliest und sie in einer Panic ausgibt. Sie müssen Ihren Handler in `idt.rs` an der richtigen Stelle in die IDT eintragen.

Sofern der Kernel weiterhin funktioniert, können wieder die Interrupts aktiviert werden sowie unsere bestehenden User Threads ge­testet werden.

*Wichtige Information für diese Aufgabe finden Sie in Intel Software Developer’s Manual Volume 3 in Kapitel 5.5 4-Level Paging and 5-Level Paging.*

## A11.2: Seitentabellen für User Threads
Nun soll eine erste Isolation der Threads im User-Mode erfolgen, zunächst nur für die Stacks. Der Stack jedes User-Mode Threads soll an einem gegebenen virtuellen Adressbereich liegen (ab 64 TiB) (siehe `consts.rs`). Der User-Stack soll also nicht mehr über den globalen Heap-Allokator alloziert werden. Damit dieser Adressbereich genutzt werden kann muss ein Mapping in den Seitentabellen eingerichtet werden. Hierfür muss in `pages.rs` die Funktion `map_user_stack()` implementiert werden, welche in `threads.rs` für das Allozieren des User-Stacks verwendet werden soll. Die Funktion `PageTable::map()` muss dafür auch mit dem Parameter `kernel = false` umgehen können. Dabei soll auf der untersten Ebene nicht mehr ein 1:1 Mapping wie beim Kernel eingerichtet werden, sondern jeweils ein Page Frame mit Hilfe des Frame Allokators alloziert und eingeblendet werden.

Der User-Stack kann im `Thread`-Struct weiterhin als `Vec` angelegt werden, obwohl er nicht mehr dynamisch auf dem Heap alloziert wird, sonder die (virtuelle) Adresse jetzt fest ist. Hierzu kann die Rust-Funktion `Vec::from_raw_parts()` verwendet werden, welche einen neuen `Vec` mit einer gegebenen Adresse, Kapazität und Größe initialisiert. *ACHTUNG:* Diese Funktion sollte generell nur mit Vorsicht eingesetzt werden, da der `Vec` seinen Speicher auf dem Kernel-Heap freigeben möchte, wenn er gedropped wird.

Der Kernel-Stack eines jeden Threads soll weiterhin über den Heap-Allokator angelegt werden. Jeder Thread bekommt so einen eigenen Addressraum, welchen wir bei der Erstellung eines Threads mit `pages::init_kernel_tables()` anlegen können. Der Kernel ist dadurch in jeden Addressraum eingeblendet und bleibt durch die bisher verwendeten Page Table Flags auch vom User Mode aus ausführbar. Das ändert sich in den kommenden Ausbaustufen des Systems.

Die Adresse der Page Map Level 4 (PML4), welche von `pages::init_kernel_tables()` zurückgegeben wird, muss außerdem im `Thread`-Struct gespeichert werden. In `Thread::start()` und `thread_switch()` muss nun bei jedem Thread-Wechsel auch in den Adressraum des nächsten Threads gewechselt werden. Hierzu bekommt `thread_switch()` einen weiteren Parameter `next_pml4`.

Testen Sie diese Aufgabe mit zwei User-Threads, beispielweise Threads die einen Zähler ausgeben. Prüfen Sie auch mit dem Debugger, dass die Stacks wirklich alle an der gleichen virtuellen Adresse beginnen und die Isolation funktioniert!

![Mapping eines virtuellen Adressraums](img/address_space.png)

*Abschließende Bemerkungen*
- *Die Isolation des Codes folgt später. Wir haben zwar bereits eine Systemaufrufsschnittstelle, können aber daran vorbei noch beliebige Funktionen des Kernels aufrufen.*
- *Auch die Isolation des Heaps folgt später. Wir verwenden also vorerst unseren globalen Allokator für jeden User-Level Thread, welcher noch Speicher im Kernel-Space verwendet.*
- *Beides funktioniert noch, da wir die die Seiten für den Kernel noch nicht geschützt haben.*
