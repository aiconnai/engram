//! Second-process helper for the C4 viability tests.
//!
//! Always opens the database through SQLite's stock "unix" VFS by pathname,
//! exactly as an unrelated Engram process (CLI, hook, second MCP server)
//! would. Prints one token on stdout.
//!
//! peer probe-reserved <db>  -> HELD | FREE   (fcntl F_GETLK on RESERVED byte)
//! peer try-immediate <db>   -> OK | BUSY     (BEGIN IMMEDIATE, busy_timeout=0)
//! peer insert <db> <value>  -> OK
//! peer count <db>           -> <n>
//! peer sentinel <db>        -> <first row>
//! peer make <db> <sentinel> -> OK  (new WAL database with one row)

use rusqlite::{Connection, OpenFlags};
use std::os::unix::io::AsRawFd;

const PENDING_BYTE: i64 = 0x4000_0000;
const RESERVED_BYTE: i64 = PENDING_BYTE + 1;

fn open(db: &str) -> Connection {
    let flags = OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX;
    Connection::open_with_flags(db, flags).expect("peer open")
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(String::as_str).unwrap_or("");
    let db = args.get(2).cloned().unwrap_or_default();
    match cmd {
        "probe-reserved" => {
            // Opening a descriptor in THIS process is harmless: the peer holds
            // no SQLite locks. F_GETLK reports a conflicting lock held by any
            // other process.
            let f = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(&db)
                .expect("peer open raw");
            let mut fl: libc::flock = unsafe { std::mem::zeroed() };
            fl.l_type = libc::F_WRLCK as _;
            fl.l_whence = libc::SEEK_SET as _;
            fl.l_start = RESERVED_BYTE as _;
            fl.l_len = 1;
            let rc = unsafe { libc::fcntl(f.as_raw_fd(), libc::F_GETLK, &mut fl) };
            assert_eq!(rc, 0, "F_GETLK failed");
            if fl.l_type == libc::F_UNLCK as _ {
                println!("FREE");
            } else {
                println!("HELD");
            }
        }
        "try-immediate" => {
            let c = open(&db);
            c.busy_timeout(std::time::Duration::from_millis(0)).unwrap();
            match c.execute_batch("BEGIN IMMEDIATE") {
                Ok(()) => {
                    c.execute_batch("ROLLBACK").unwrap();
                    println!("OK");
                }
                Err(e) => {
                    let busy = matches!(
                        &e,
                        rusqlite::Error::SqliteFailure(f, _)
                            if f.code == rusqlite::ErrorCode::DatabaseBusy
                    );
                    if busy {
                        println!("BUSY");
                    } else {
                        println!("ERR {e}");
                    }
                }
            }
        }
        "insert" => {
            let c = open(&db);
            c.busy_timeout(std::time::Duration::from_secs(5)).unwrap();
            c.execute("INSERT INTO t(v) VALUES (?1)", [&args[3]])
                .expect("peer insert");
            println!("OK");
        }
        "count" => {
            let c = open(&db);
            let n: i64 = c
                .query_row("SELECT count(*) FROM t", [], |r| r.get(0))
                .expect("peer count");
            println!("{n}");
        }
        "sentinel" => {
            let c = open(&db);
            let v: String = c
                .query_row("SELECT v FROM t ORDER BY rowid LIMIT 1", [], |r| r.get(0))
                .expect("peer sentinel");
            println!("{v}");
        }
        "make" => {
            c4spike::util::make_db(std::path::Path::new(&db), &args[3], "wal");
            println!("OK");
        }
        other => {
            eprintln!("unknown peer command {other:?}");
            std::process::exit(2);
        }
    }
}
