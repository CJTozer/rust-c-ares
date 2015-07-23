// Simple example using futures.
extern crate c_ares;

use std::sync::{mpsc, Arc, Future};

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

fn print_aaaa_results(result: Result<c_ares::AAAAResults, c_ares::AresError>) {
    println!("");
    match result {
        Err(e) => {
            let err_string = c_ares::str_error(e);
            println!("AAAA lookup failed with error '{}'", err_string);
        }
        Ok(aaaa_results) => {
            println!("Successful AAAA lookup...");
            println!("Hostname: {}", aaaa_results.hostname());
            for aaaa_result in &aaaa_results {
                println!("{:}", aaaa_result.ipv6_addr());
            }
        }
    }
}

fn print_srv_results(result: Result<c_ares::SRVResults, c_ares::AresError>) {
    println!("");
    match result {
        Err(e) => {
            let err_string = c_ares::str_error(e);
            println!("SRV lookup failed with error '{:}'", err_string);
        }
        Ok(srv_results) => {
            println!("Successful SRV lookup...");
            for srv_result in &srv_results {
                println!("host: {} (port: {}), priority: {} weight: {}",
                         srv_result.host(),
                         srv_result.port(),
                         srv_result.weight(),
                         srv_result.priority());
            }
        }
    }
}

fn print_cname_result(result: Result<c_ares::CNameResult, c_ares::AresError>) {
    println!("");
    match result {
        Err(e) => {
            let err_string = c_ares::str_error(e);
            println!("CNAME lookup failed with error '{}'", err_string);
        }
        Ok(cname_result) => {
            println!("Successful CNAME lookup...");
            println!("{}", cname_result.cname());
        }
    }
}

fn print_mx_results(result: Result<c_ares::MXResults, c_ares::AresError>) {
    println!("");
    match result {
        Err(e) => {
            let err_string = c_ares::str_error(e);
            println!("MX lookup failed with error '{}'", err_string);
        }
        Ok(mx_results) => {
            println!("Successful MX lookup...");
            for mx_result in &mx_results {
                println!(
                    "host {}, priority {}",
                    mx_result.host(),
                    mx_result.priority());
            }
        }
    }
}

fn print_ns_results(result: Result<c_ares::NSResults, c_ares::AresError>) {
    println!("");
    match result {
        Err(e) => {
            let err_string = c_ares::str_error(e);
            println!("NS lookup failed with error '{}'", err_string);
        }
        Ok(ns_results) => {
            println!("Successful NS lookup...");
            for ns_result in &ns_results {
                println!("{:}", ns_result.name_server());
            }
        }
    }
}

fn print_ptr_results(result: Result<c_ares::PTRResults, c_ares::AresError>) {
    println!("");
    match result {
        Err(e) => {
            let err_string = c_ares::str_error(e);
            println!("PTR lookup failed with error '{}'", err_string);
        }
        Ok(ptr_results) => {
            println!("Successful PTR lookup...");
            for ptr_result in &ptr_results {
                println!("{:}", ptr_result.cname());
            }
        }
    }
}

fn print_txt_results(result: Result<c_ares::TXTResults, c_ares::AresError>) {
    println!("");
    match result {
        Err(e) => {
            let err_string = c_ares::str_error(e);
            println!("TXT lookup failed with error '{}'", err_string);
        }
        Ok(txt_results) => {
            println!("Successful TXT lookup...");
            for txt_result in &txt_results {
                println!("{:}", txt_result.text());
            }
        }
    }
}

fn print_soa_result(result: Result<c_ares::SOAResult, c_ares::AresError>) {
    println!("");
    match result {
        Err(e) => {
            let err_string = c_ares::str_error(e);
            println!("SOA lookup failed with error '{}'", err_string);
        }
        Ok(soa_result) => {
            println!("Successful SOA lookup...");
            println!("Name server: {}", soa_result.name_server());
            println!("Hostmaster: {}", soa_result.hostmaster());
            println!("Serial: {}", soa_result.serial());
            println!("Retry: {}", soa_result.retry());
            println!("Expire: {}", soa_result.expire());
            println!("Min TTL: {}", soa_result.min_ttl());
        }
    }
}

fn main() {
    // Dummy callback.
    let dummy_callback = move |_: i32, _: bool, _: bool| {};

    // Create a c_ares::Channel.
    let mut options = c_ares::Options::new();
    options
        .set_timeout(500)
        .set_tries(3);
    let mut ares_channel = c_ares::Channel::new(dummy_callback, options) // Cheating on the channel
        .ok()
        .expect("Failed to create channel");

    // Perform a query, getting the result as a future.
    let mut results_arc: Arc<Result<c_ares::AResults, c_ares::AresError>> = Arc::new(Err(c_ares::AresError::ENODATA));
    let mut results_arc_clone = results_arc.clone();
    let results_future = Future::spawn(move || {
        ares_channel.query_a("apple.com", move |results| {
            *results_arc_clone = results;
        });
    });

    // Wait for the results
    results_future.into_inner();
    print_a_results(*results_arc);
}
