#![allow(unused)]
use core::panic;
use std::{
    collections::VecDeque,
    env::args,
    fs,
    path::Path,
    process::{Command, Output},
    sync::{Arc, Mutex},
    thread,
};

fn grep(path: &str, search_string: &str) -> Output {
    Command::new("grep")
        .arg(search_string)
        .arg(path)
        .output()
        .expect(" Failed to execute grep")
}
struct TaskQueue {
    q: VecDeque<String>,
    rootnest: i32,
}
fn main() {
    let args: Vec<String> = args().collect();
    let n: i32;
    let rootpath: String;
    let search_string: String;

    match args.as_slice() {
        [_, a1, a2, a3] => {
            n = a1
                .to_owned()
                .parse::<i32>()
                .unwrap_or_else(|_| panic!("Invalid argument 1"));
            rootpath = fs::canonicalize(Path::new(a2))
                .unwrap_or_else(|_| panic!("Something wrong with rootpath argument 2"))
                .to_str()
                .unwrap()
                .strip_prefix("\\\\?\\")
                .unwrap()
                .to_owned();
            search_string = a3.to_owned();
        }
        _ => panic!("Not enough Arguments!"),
    }

    let tq = Arc::new(Mutex::new(TaskQueue {
        q: VecDeque::new(),
        rootnest: 0,
    }));

    {
        let mut tq = tq.lock().unwrap();
        tq.q.push_back(String::from(rootpath));
        tq.rootnest += 1;
    }

    for wid in 0..n {
        let tq = tq.clone();
        let search_string = search_string.clone();
        let worker = thread::spawn(move || loop {
            // Test
            let path: String;
            {
                let mut tq = tq.lock().unwrap();

                if tq.rootnest == 0 {
                    break;
                };

                if let Some(elem) = tq.q.pop_front() {
                    path = elem;
                } else {
                    continue;
                }
                println!("[{}] DIR {:?}", wid, path);
            }

            if let Ok(dir) = fs::read_dir(path) {
                for entry in dir {
                    let entry = entry.unwrap();
                    let metadata = fs::metadata(entry.path()).unwrap();

                    if metadata.is_dir() {
                        let mut tq = tq.lock().unwrap();
                        tq.q.push_back(entry.path().to_str().unwrap().to_owned());
                        tq.rootnest += 1;
                    } else if metadata.is_file() {
                        let status = grep(entry.path().to_str().unwrap(), &search_string)
                            .status
                            .success();
                        println!(
                            "[{}] {} {:?}",
                            wid,
                            if status { "PRESENT" } else { "ABSENT" },
                            entry.path()
                        );
                    }
                }
            };

            {
                let mut tq = tq.lock().unwrap();
                tq.rootnest -= 1;
            }
        });
        worker.join().unwrap();
    }
}
