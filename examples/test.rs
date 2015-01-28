extern crate signals;

fn main() {
    let sigs = signals::Signals::new().unwrap();
    sigs.subscribe(signals::Signal::TermStop);
    for s in sigs.receiver().iter() {
        println!("{:?}", s);
    }
}
