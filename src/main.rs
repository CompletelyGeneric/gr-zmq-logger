extern crate cero;
extern crate clap;
extern crate zmq;
extern crate serde;
extern crate serde_json;
use std::path::PathBuf;
use std::io::prelude::*;
use std::fs::File;
use std::fs::canonicalize;
use std::io::BufWriter;
use cero::deserialize;
use clap::{App, SubCommand};



const DEFAULT_FILE_SIZE: usize = 1000000; // 1MB


fn main() {
    let matches = App::new("gr-zmq-log")
                        .version("0.1.0")
                        .author("Hugh M. <contact@hugh.moe>")
                        .about("Stand-alone logger for GNURadio's ZeroMQ blocks")
                        .args_from_usage(
                            // TODO: add ability to set ZeroMQ subscription(s)  
                            "<IP> -i --ip <ipaddr>             'Set the ZeroMQ IP address to connect to, of the form \'protocol://host:port\''
                             [OUTPUT] -o --output [output]     'Write output files to <DIR> (default: \'./\')'
                             [FORMAT] -f --format <format>     'Set output file format. (default: PRETTY) <CSV | JSON | PRETTY>'
                             [MAXSIZE] -s --size <maxsize>     'Set maximum file size in MB's, that when reached, will continue logging to new file of name <OUTPUT>-N. (default:  5(MB))'
                             [NUMMSGS] -n --nummsgs <nummsgs>  'Number of ZeroMQ messages to log. (default: infinity)'")
                        .get_matches();



    let file_size = match matches.value_of("MAXSIZE") {
        Some(s) => s.parse::<usize>().expect("Invalid size parameter. Hint: try \'-s 5\'") * 1000000,
        None => DEFAULT_FILE_SIZE,
    };
    

    let outdir = match matches.value_of("OUTPUT") {
        Some(d) => canonicalize(d).expect("invalid directory"),
        None => canonicalize(".").unwrap(),
    }; 

    let ipaddr = matches.value_of("IP").unwrap();

    let format = match matches.value_of("FORMAT") {
        Some(f) => f,
        None => "pretty"
    };


    // We start parsing from this match because we have two different functions to handle parsing behavoir
    // based on if we want to parse a discrete number of messages or not
    // This is quick and dirty, and will be re-written
    match matches.value_of("NUMMSGS") {
        Some(s) => write_log_ntimes(format, outdir, ipaddr, file_size, s.parse::<usize>().expect("Invalid nummsgs parameter. Hint: try \'-n 1000\'")),
        None => write_log_forever(format, outdir, ipaddr, file_size),
    };

    
    
}

fn write_log_forever(format :&str, outdir: PathBuf, ipaddr: &str, max_file_size: usize) {
    let ctx = zmq::Context::new();
    let socket = ctx.socket(zmq::SUB).expect("Unable to set socket type");
    socket.connect(ipaddr).expect("Unable to connect to socket");
    let subs = [];
    socket.set_subscribe(&subs).expect("Unable to set ZeroMQ subscriptions");
    
    let mut num_files = 0;

    // Ugly hack
    let ext = match format {
        "pretty" => "json",
        _ => format,
    };



    loop {
        // This is awful, but PathBuf was fighting me.  
        // It will certainly be fixed next release
        let outfile = File::create(outdir.to_str().unwrap().to_owned() + "/" + &num_files.to_string() + "." + ext).unwrap();
        let mut outbuf = BufWriter::new(outfile);
        let mut current_file_size = 0;
        while current_file_size < max_file_size {
            let zmq_msg = &socket.recv_bytes(0).unwrap();
            let cero_msg = deserialize(zmq_msg);
            current_file_size += zmq_msg.len();
            match format {
                "json" => { 
                    outbuf.write(serde_json::to_string(&cero_msg).unwrap().as_bytes());  
                },
                "pretty" => {
                    outbuf.write(serde_json::to_string_pretty(&cero_msg).unwrap().as_bytes());
                },
                "csv" => {},
                _ => {},
            }
        }
        outbuf.flush();
        num_files += 1;
    }
}

fn write_log_ntimes(format: &str, outdir: PathBuf, ipaddr: &str, max_file_size: usize, num_msgs: usize) {
let ctx = zmq::Context::new();
    let socket = ctx.socket(zmq::SUB).expect("Unable to set socket type");
    socket.connect(ipaddr).expect("Unable to connect to socket");
    let subs = [];
    socket.set_subscribe(&subs).expect("Unable to set ZeroMQ subscriptions");

    let mut recvd_msgs = 0;
    let mut num_files = 0;
    let ext = match format {
        "pretty" => "json",
        _ => format,
    };

    while recvd_msgs < num_msgs {
        let outfile = File::create(outdir.to_str().unwrap().to_owned() + "/" + &num_files.to_string() + "." + ext).unwrap();
        let mut outbuf = BufWriter::new(outfile);
        let mut current_file_size = 0;
        while current_file_size < max_file_size && recvd_msgs < num_msgs {
            let zmq_msg = &socket.recv_bytes(0).unwrap();
            let cero_msg = deserialize(zmq_msg);
            current_file_size += zmq_msg.len();
            match format {
                "json" => { 
                    outbuf.write(serde_json::to_string(&cero_msg).unwrap().as_bytes());  
                },
                "pretty" => {
                    outbuf.write(serde_json::to_string_pretty(&cero_msg).unwrap().as_bytes());
                },
                "csv" => {},
                _ => {
                     outbuf.write(serde_json::to_string_pretty(&cero_msg).unwrap().as_bytes());
                },
            }
            recvd_msgs += 1;
        }
        outbuf.flush();
        num_files += 1;
    }
}