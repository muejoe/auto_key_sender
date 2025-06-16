
use enigo::{Enigo, KeyboardControllable};
use rdev::{listen, Event, EventType, Key};
use std::{env, sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex}, thread, time::Duration};

struct Arguments {
    pub interval_ms: u64,
    pub key: char
}

struct Modifiers {
    pub shift: bool,
    pub ctrl: bool
}

fn main() {
    let arguments = check_arguments();
    if arguments.is_none() { return; }
    let arguments = arguments.unwrap();

    // shared variables on heap, access through thread safe pointer
    // Arc for thread safe sharing, AtomicBool for thread safe modification
    let shall_run = Arc::new(AtomicBool::new(true));
    let shall_click = Arc::new(AtomicBool::new(false));
    let modifiers = Arc::new( Mutex::new(Modifiers{shift: false, ctrl: false}) );
    
    // make copys of the arcs and move them through the closure to the thread
    let shall_run2 = shall_run.clone();
    let shall_click2 = shall_click.clone();
    thread::spawn( move || clicker_thread(shall_click2, shall_run2, arguments.interval_ms, arguments.key));

    // move arcs to listen closure, called function gets references to it, when events occur  
    if let Err(error) = listen(move |event: Event| 
        listen_to_hotkeys(event, &modifiers, &shall_click, &shall_run)
    ) {
        println!("Error: {:?}", error);
    }
}

fn check_arguments() -> Option<Arguments> {
let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Usage: {} <interval_in_milliseconds> <key>", args[0]);
        println!("Start/stop auto key events with Shift+Ctrl+F11.");
        println!("Stop application with Shift+Ctrl+Q.");
        return None;
    }
    if args[2].chars().count() != 1 {
        eprint!("Key must be 1 character long.");
        return None;
    }

    let interval_ms: u64 = args[1].parse().expect("Interval must be an integer number.");
    let key = args[2].chars().next().unwrap();
    Some( Arguments{interval_ms, key} )
}

fn clicker_thread(shall_click: Arc<AtomicBool>, shall_run: Arc<AtomicBool>, interval: u64, key: char) {
    let mut enigo = Enigo::new();
    while shall_run.load( Ordering::SeqCst) {
        if shall_click.load(Ordering::SeqCst) {
            enigo.key_click(enigo::Key::Layout(key)); 
            thread::sleep(Duration::from_millis(interval)); 
        } else {
            thread::sleep(Duration::from_millis(100));
        }
    }
    std::process::exit(0);
}

fn listen_to_hotkeys(event: Event, modifiers: &Arc<Mutex<Modifiers>>, shall_click: &Arc<AtomicBool>, shall_run: &Arc<AtomicBool>) {
    //panicks when already held by this thread, so unwrap seems okay
    let mut mods = modifiers.lock().unwrap(); 

    match event.event_type {
        EventType::KeyPress(key) => {
            match key {
                Key::ShiftLeft | Key::ShiftRight => mods.shift = true,
                Key::ControlLeft | Key::ControlRight => mods.ctrl = true,
                Key::F11 => {
                    if mods.shift && mods.ctrl { // == F11 + Strg + Shift
                        let currently_running = shall_click.load(Ordering::SeqCst);
                        shall_click.store(!currently_running, Ordering::SeqCst);
                    }
                },
                Key::KeyQ => {
                    if mods.shift && mods.ctrl {
                        println!("Shift+Ctrl+Q detected, ending application ...");
                        shall_run.store(false, Ordering::SeqCst);
                    }
                }
                _ => ()
            }
        }
        EventType::KeyRelease(key) => {
            match key {
                Key::ShiftLeft | Key::ShiftRight => mods.shift = false,
                Key::ControlLeft | Key::ControlRight => mods.ctrl = false,
                _ => ()
            }
        }
        _ => ()
    };
}