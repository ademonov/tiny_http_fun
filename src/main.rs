extern crate num_cpus;
extern crate tiny_http;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use std::thread;


fn main() {
    eprintln!("test");
    let num_cpus = num_cpus::get();
    println!("{} CPUs detected", num_cpus);

    let alive = Arc::new(AtomicBool::new(true));
    let server = Arc::new(tiny_http::Server::http("0.0.0.0:3000").unwrap());
    println!("Now listening on port 9975");

    let duration = Duration::from_secs(1);
    let mut handles = Vec::new();
    for i in 0 .. num_cpus {
        let server = server.clone();
        let alive = alive.clone();

        handles.push(thread::spawn(move || {
            println!("starting worker #{}...", i);
            
            while alive.load(Ordering::Relaxed) {
                match server.recv_timeout(duration) {
                    Ok(Some(request)) => { 
                        let response = handle_request(&request, &alive);
                        if let Err(e) = request.respond(response) { 
                            eprintln!("Respond error: {}", e);
                        }
                    },

                    Ok(None) => { /* No requests come */ }
                    Err(err) => eprintln!("Error: {}", err),
                }
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }
}

use tiny_http::{Request, Response, Method, StatusCode};
fn handle_request(request: &Request, alive: &Arc<AtomicBool>) -> Response<std::io::Cursor<std::vec::Vec<u8>>> {
    match (request.method(), request.url()) {
        (&Method::Get, url) => { 
            match url {
                "/shutdown" => {
                    alive.store(false, Ordering::Relaxed);
                    Response::from_string("good bye!")
                }
                _ => Response::from_string(url).with_status_code(StatusCode::from(404)),
            }         
        }
        _ => {
            Response::from_string("").with_status_code(StatusCode::from(404))
        }
    }    
}
