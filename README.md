# bsc

A complete [beanstalkd](https://beanstalkd.github.io/) client library and CLI.

The [`bsc`](/crates/lib) crate implements every commands defined in [protocol.txt](https://raw.githubusercontent.com/beanstalkd/beanstalkd/master/doc/protocol.txt), while [`bsc-cli`](/crates/cli) leverages [clap](https://docs.rs/clap/latest/clap/) to provide a user-friendly command-line interface (CLI).

## Example
```toml
[dependencies]
bsc = { version = "0.2.0" }
```

Then, on your main.rs:
```rs
use bsc::{Beanstalk, PutResponse, ReserveResponse};

fn main() {
    let mut bsc = Beanstalk::connect("172.21.0.2:11300").unwrap();

    let res = bsc
        .put(
            0,
            Duration::from_secs(0),
            Duration::from_secs(15),
            b"hello beanstalkd",
        )
        .unwrap();

    if let PutResponse::Inserted(id) = res {
        println!("New job inserted successfully: {id}");

        let res = bsc.reserve(None).unwrap();

        if let ReserveResponse::Reserved { id, data } = res {
            println!("id   = {id}");
            println!("data = {}", std::str::from_utf8(&data).unwrap());
        }

        bsc.delete(id).unwrap();
    }
}
```

Considering your Beanstalkd instance if available at `172.21.0.2:11300` and has already 41 jobs in queue, you should see:
```text
New job inserted successfully: 42
id   = 42
data = hello beanstalkd
```


