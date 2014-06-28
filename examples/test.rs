extern crate signals;
extern crate debug;

fn main() {
    let sigs = signals::Signals::new().unwrap();
    sigs.subscribe(signals::TermStop);
    for s in sigs.receiver().iter() {
        println!("{:?}", s);
    }
}
