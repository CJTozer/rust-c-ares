// Simple example using `Futures`.
extern crate c_ares;

use std::sync::mpsc;

// TODO - commonize with the other example(s).
fn print_a_results(result: Result<c_ares::AResults, c_ares::AresError>) {
    println!("");
    match result {
        Err(e) => {
            let err_string = c_ares::str_error(e);
            println!("A lookup failed with error '{}'", err_string);
        }
        Ok(a_results) => {
            println!("Successful A lookup...");
            println!("Hostname: {}", a_results.hostname());
            for a_result in &a_results {
                println!("{:}", a_result.ipv4_addr());
            }
        }
    }
}

struct Resolver {
    ares_channel: c_ares::Channel,
}
impl Resolver {
    fn new() -> Resolver {
        // Dummy callback.
        let dummy_callback = move |_: i32, _: bool, _: bool| {};

        // Create a c_ares::Channel.
        let mut options = c_ares::Options::new();
        options
            //.set_flags(c_ares::flags::STAYOPEN | c_ares::flags::EDNS)
            .set_timeout(500)
            .set_tries(3);
        let ares_channel = c_ares::Channel::new(dummy_callback, options) // Cheating on the channel
            .ok()
            .expect("Failed to create channel");

        Resolver { ares_channel: ares_channel }
    }

    fn a_query_as_future(&mut self, name: &str) -> Box<Fn() -> Result<c_ares::AResults, c_ares::AresError>> {
        // Make the query.
        let (tx, rx) = mpsc::channel();
        self.ares_channel.query_a(name, move |results| {
            tx.send(results).unwrap();
        });
        
        // Return a closure that waits to receive the result.
        Box::new(move || {
            rx.recv().unwrap()
        })
    }
}

fn main() {
    // Perform a query, getting the result as a future.
    let mut resolver = Resolver::new();
    let get_results = resolver.a_query_as_future("apple.com");

    // Do some other stuff here while we wait
    // ...

    // Wait for and print the results
    print_a_results(get_results());
}
