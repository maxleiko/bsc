use std::time::Duration;

use bsc_lib::*;

fn main() {
    let endpoint = std::env::args()
        .nth(1)
        .expect("missing breanstalk endpoint argument");
    let mut bs = Beanstalk::new(endpoint, None).unwrap();

    let res = bs.stats_job(3).unwrap();
    println!("stats_job: {res:#?}");

    let res = bs.stats_tube("default").unwrap();
    println!("stats_tube: {res:#?}");

    let res = bs.stats().unwrap();
    println!("stats: {res:#?}");

    let res = bs.list_tubes().unwrap();
    println!("list_tubes: {res:#?}");

    let res = bs.list_tube_used().unwrap();
    println!("list_tube_used: {res:#?}");

    let res = bs.list_tube_watched().unwrap();
    println!("list_tube_watched: {res:#?}");

    let res = bs.pause_tube("default", Duration::from_secs(15)).unwrap();
    println!("pause_tube: {res:#?}");
}

// INSERTED 1\r\n
