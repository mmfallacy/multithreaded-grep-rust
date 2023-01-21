#![allow(unused)]
use core::panic;
use std::{
    collections::VecDeque,
    fs,
    process::{Command, Output},
    sync::Arc,
    sync::Mutex,
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
    let n = 2;
    let rootpath = "E:\\Projects\\multithreaded-grep-rust\\test";
    let search_string = "Handwire";

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
            }

            if let Ok(dir) = fs::read_dir(path) {
                for entry in dir {
                    let entry = entry.unwrap();
                    let metadata = fs::metadata(entry.path()).unwrap();

                    if metadata.is_dir() {
                        let mut tq = tq.lock().unwrap();
                        tq.q.push_back(entry.path().to_str().unwrap().to_owned());
                        tq.rootnest += 1;
                        println!("[{}] DIR {:?}", wid, entry.path());
                    } else if metadata.is_file() {
                        let status = grep(entry.path().to_str().unwrap(), search_string)
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
