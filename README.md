### signals.rs

A library for handling signals in UNIX-like environments.

Using this library it is possible to subscribe and unsubscribe from signals and to
handle them asynchronously. 

[Documentation](https://mahkoh.github.io/signals/doc/signals)

#### Example

```rust
let signals = Signals::new().unwrap();
signals.subscribe(Interrupt);
for s in signals.receiver().iter() {
    println!("{:?}", s);
}
```

At any given time there can only be one signal handler in the program.
`Signals::new()` returns `None` if there is already another signal handler.
