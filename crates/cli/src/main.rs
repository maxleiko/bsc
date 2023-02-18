use simple_eyre::eyre::{eyre, Report, WrapErr};
use std::io::{self, Read, Write};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

use clap::{Parser, Subcommand};

use bsc_lib::*;

fn main() -> Result<(), Report> {
    simple_eyre::install()?;

    let cli = Cli::parse();

    let mut bsc = Beanstalk::connect(cli.addr)?;

    if let Some(used) = cli.use_ {
        bsc.use_(&used)?;
    }

    match cli.cmd {
        Cmd::Put {
            pri,
            delay,
            ttr,
            filepath,
        } => {
            let data = match filepath {
                Some(fp) => std::fs::read(fp).wrap_err("unable to read <filepath>")?,
                None => {
                    let mut buf = Vec::new();
                    io::stdin()
                        .read_to_end(&mut buf)
                        .wrap_err("unable to read <stdin>")?;
                    buf
                }
            };
            let res = bsc.put(pri, delay, ttr, &data[..])?;
            eprintln!("{res:?}");
            Ok(())
        }
        Cmd::Peek { id, utf8 } => {
            match bsc.peek(id)? {
                PeekResponse::Found { data, .. } if utf8 => {
                    let s = std::str::from_utf8(&data)
                        .wrap_err("Job's data appears to not be UTF-8 encoded")?;
                    eprintln!("{s}");
                }
                PeekResponse::Found { data, .. } => {
                    io::stdout().write_all(&data)?;
                }
                res => eprintln!("{res:?}"),
            }
            Ok(())
        }
        Cmd::Reserve { .. } => Err(eyre!("cmd <reserve> not implemented yet")),
        Cmd::Delete { .. } => Err(eyre!("cmd <delete> not implemented yet")),
        Cmd::Release { .. } => Err(eyre!("cmd <release> not implemented yet")),
        Cmd::Bury { .. } => Err(eyre!("cmd <bury> not implemented yet")),
        Cmd::Touch { .. } => Err(eyre!("cmd <touch> not implemented yet")),
        Cmd::Watch {  } => Err(eyre!("cmd <watch> not implemented yet")),
        Cmd::Ignore {  } => Err(eyre!("cmd <ignore> not implemented yet")),
        Cmd::PeekReady {  } => Err(eyre!("cmd <peekready> not implemented yet")),
        Cmd::PeekDelayed {  } => Err(eyre!("cmd <peekdelayed> not implemented yet")),
        Cmd::PeekBuried {  } => Err(eyre!("cmd <peekburied> not implemented yet")),
        
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None, propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    cmd: Cmd,

    #[arg(
        name = "use",
        long,
        short,
        help = "The <tube> name to use for the command. The default tube is \"default\".",
        global = true,
        env
    )]
    use_: Option<String>,

    #[arg(
        long,
        short,
        help = "The Beanstalkd endpoint to communicate with",
        default_value = "127.0.0.1:11300",
        global = true,
        env = "BEANSTALKD"
    )]
    addr: SocketAddr,
}

#[derive(Subcommand)]
pub enum Cmd {
    #[command(
        about = "Inserts a job into the queue. If <filepath> is not specified, reads content from <stdin>."
    )]
    Put {
        #[arg(
            long,
            short,
            default_value = "0",
            help = "Jobs with smaller priority values will be scheduled before jobs with larger priorities.\nThe most urgent priority is 0; the least urgent priority is 4,294,967,295.",
            env
        )]
        pri: u32,

        #[arg(
            long,
            short,
            default_value = "0",
            value_parser = parse_duration,
            help = "An integer number of seconds to wait before putting the job in the ready queue.\nThe job will be in the \"delayed\" state during this time",
            env
        )]
        delay: Duration,

        #[arg(long, short, default_value = "0", help = TTR_HELP)]
        ttr: u32,

        #[arg(
            index = 1,
            help = "Uses the content of the specified file for the job data.\nIf no <filepath> is given, the data is read from <stdin>.",
            env
        )]
        filepath: Option<PathBuf>,
    },

    #[command(
        about = "This will return a newly-reserved job. If no job is available to be reserved, beanstalkd will wait to send a response until one becomes available."
    )]
    Reserve {},

    #[command(
        about = "The delete command removes a job from the server entirely.",
        long_about = "It is normally used by the client when the job has successfully run to completion.\nA client can delete jobs that it has reserved, ready jobs, delayed jobs, and jobs that are buried."
    )]
    Delete {},

    #[command(
        about = "The release command puts a reserved job back into the ready queue (and marks its state as \"ready\") to be run by any client. It is normally used when the job fails because of a transitory error."
    )]
    Release {},

    #[command(
        about = "The bury command puts a job into the \"buried\" state. Buried jobs are put into a kicks them with the \"kick\" command."
    )]
    Bury {},

    #[command(
        about = "The \"touch\" command allows a worker to request more time to work on a job.",
        long_about = "This is useful for jobs that potentially take a long time, but you still want the benefits of a TTR pulling a job away from an unresponsive worker.\nA worker may periodically tell the server that it's still alive and processing a job (e.g. it may do this on DEADLINE_SOON).\nThe command postpones the auto release of a reserved job until TTR seconds from when the command is issued."
    )]
    Touch {},

    #[command(
        about = "The \"watch\" command adds the named tube to the watch list for the current connection.",
        long_about = "A reserve command will take a job from any of the tubes in the watch list.\nFor each new connection, the watch list initially consists of one tube, named \"default\"."
    )]
    Watch {},

    #[command(
        about = "The \"ignore\" command is for consumers. It removes the named tube from the watch list for the current connection."
    )]
    Ignore {},

    #[command(about = "Return the job <id>.")]
    Peek {
        #[arg(index = 1, help = "The job <id> to peek.", env)]
        id: Id,

        #[arg(long, help = "Tries to parse the body as UTF-8 encoded bytes.", env)]
        utf8: bool,
    },

    #[command(about = "Return the next ready job. Operates only on the currently used tube.")]
    PeekReady {},

    #[command(
        about = "Return the delayed job with the shortest delay left. Operates only on the currently used tube."
    )]
    PeekDelayed {},

    #[command(
        about = "Return the next job in the list of buried jobs. Operates only on the currently used tube."
    )]
    PeekBuried {},
}

fn parse_duration(arg: &str) -> Result<Duration, std::num::ParseIntError> {
    Ok(Duration::from_secs(arg.parse()?))
}

const TTR_HELP: &str = r#"-- time to run -- is an integer number of seconds to allow a worker to run this job.
This time is counted from the moment a worker reserves this job.
If the worker does not delete, release, or bury the job within `ttr` seconds,
the job will time out and the server will release the job. The minimum ttr is 1.
If the  client sends 0, the server will silently increase the ttr to 1.
Maximum ttr is 2**32-1."#;
