use simple_eyre::eyre::{Report, WrapErr};
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
            println!("{res:?}");
            Ok(())
        }
        Cmd::Peek { id, utf8 } => {
            match bsc.peek(id)? {
                PeekResponse::Found { data, .. } if utf8 => {
                    let s = std::str::from_utf8(&data)
                        .wrap_err("Job's data appears to not be UTF-8 encoded")?;
                    println!("{s}");
                }
                PeekResponse::Found { data, .. } => {
                    io::stdout().write_all(&data)?;
                }
                res => println!("{res:?}"),
            }
            Ok(())
        }
        Cmd::Reserve { timeout } => {
            let res = bsc.reserve(timeout)?;
            println!("{res:?}");
            Ok(())
        }
        Cmd::Delete { id } => {
            let res = bsc.delete(id)?;
            println!("{res:?}");
            Ok(())
        }
        Cmd::Release { id, pri, delay } => {
            let res = bsc.release(id, pri, delay)?;
            println!("{res:?}");
            Ok(())
        }
        Cmd::Bury { id, pri } => {
            let res = bsc.bury(id, pri)?;
            println!("{res:?}");
            Ok(())
        }
        Cmd::Touch { id } => {
            let res = bsc.touch(id)?;
            println!("{res:?}");
            Ok(())
        }
        Cmd::Watch { tube } => {
            let n = bsc.watch(&tube)?;
            println!("Watching({n})");
            Ok(())
        }
        Cmd::Ignore { tube } => {
            let res = bsc.ignore(&tube)?;
            println!("{res:?}");
            Ok(())
        }
        Cmd::PeekReady => {
            let res = bsc.peek_ready()?;
            println!("{res:?}");
            Ok(())
        }
        Cmd::PeekDelayed => {
            let res = bsc.peek_delayed()?;
            println!("{res:?}");
            Ok(())
        }
        Cmd::PeekBuried => {
            let res = bsc.peek_buried()?;
            println!("{res:?}");
            Ok(())
        }
        Cmd::Kick { bound } => {
            let n = bsc.kick(bound)?;
            println!("Kicked({n})");
            Ok(())
        }
        Cmd::KickJob { id } => {
            let res = bsc.kick_job(id)?;
            println!("{res:?}");
            Ok(())
        }
        Cmd::StatsJob { id } => {
            match bsc.stats_job(id)? {
                StatsJobResponse::Ok(res) => serde_json::to_writer_pretty(io::stdout(), &res)?,
                StatsJobResponse::NotFound => println!("NotFound"),
            }
            Ok(())
        }
        Cmd::StatsTube { tube } => {
            match bsc.stats_tube(&tube)? {
                StatsTubeResponse::Ok(res) => serde_json::to_writer_pretty(io::stdout(), &res)?,
                StatsTubeResponse::NotFound => println!("NotFound"),
            }
            Ok(())
        }
        Cmd::Stats => {
            let res = bsc.stats()?;
            serde_json::to_writer_pretty(io::stdout(), &res)?;
            Ok(())
        }
        Cmd::ListTubes => {
            let res = bsc.list_tubes()?;
            serde_json::to_writer_pretty(io::stdout(), &res)?;
            Ok(())
        }
        Cmd::ListTubesUsed => {
            let res = bsc.list_tube_used()?;
            serde_json::to_writer_pretty(io::stdout(), &res)?;
            Ok(())
        }
        Cmd::ListTubesWatched => {
            let res = bsc.list_tube_watched()?;
            serde_json::to_writer_pretty(io::stdout(), &res)?;
            Ok(())
        }
        Cmd::PauseTube { tube, delay } => {
            let res = bsc.pause_tube(&tube, delay)?;
            println!("{res:?}");
            Ok(())
        }
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
    Reserve {
        #[arg(
            long,
            short,
            value_parser = parse_duration,
            help = "A timeout value of 0 will cause the server to immediately return either a response or TIMED_OUT.\nA positive value of timeout will limit the amount of time the client will block on the reserve request until a job becomes available.",
            env
        )]
        timeout: Option<Duration>,
    },

    #[command(
        about = "The delete command removes a job from the server entirely.",
        long_about = "It is normally used by the client when the job has successfully run to completion.\nA client can delete jobs that it has reserved, ready jobs, delayed jobs, and jobs that are buried."
    )]
    Delete {
        #[arg(index = 1, env, help = "The job <id>.")]
        id: Id,
    },

    #[command(
        about = "The release command puts a reserved job back into the ready queue (and marks its state as \"ready\") to be run by any client. It is normally used when the job fails because of a transitory error."
    )]
    Release {
        #[arg(index = 1, env, help = "The job <id>.")]
        id: Id,

        #[arg(index = 2, env, help = "The new priority to assign to the job.")]
        pri: u32,

        #[arg(index = 3, env, value_parser = parse_duration, help = "An integer number of seconds to wait before putting the job in the ready queue.")]
        delay: Duration,
    },

    #[command(
        about = "The bury command puts a job into the \"buried\" state.",
        long_about = "The bury command puts a job into the \"buried\" state.\nBuried jobs are put into a FIFO linked list and will not be touched by the server again until a client kicks them with the \"kick\" command."
    )]
    Bury {
        #[arg(index = 1, env, help = "The job <id>.")]
        id: Id,

        #[arg(index = 2, env, help = "The new priority to assign to the job.")]
        pri: u32,
    },

    #[command(
        about = "The \"touch\" command allows a worker to request more time to work on a job.",
        long_about = "The \"touch\" command allows a worker to request more time to work on a job.\nThis is useful for jobs that potentially take a long time, but you still want the benefits of a TTR pulling a job away from an unresponsive worker.\nA worker may periodically tell the server that it's still alive and processing a job (e.g. it may do this on DEADLINE_SOON).\nThe command postpones the auto release of a reserved job until TTR seconds from when the command is issued."
    )]
    Touch {
        #[arg(index = 1, env, help = "The job <id>.")]
        id: Id,
    },

    #[command(
        about = "The \"watch\" command adds the named tube to the watch list for the current connection.",
        long_about = "A reserve command will take a job from any of the tubes in the watch list.\nFor each new connection, the watch list initially consists of one tube, named \"default\"."
    )]
    Watch {
        #[arg(index = 1, env, help = "The <tube> name.")]
        tube: String,
    },

    #[command(
        about = "The \"ignore\" command is for consumers. It removes the named tube from the watch list for the current connection."
    )]
    Ignore {
        #[arg(index = 1, env, help = "The <tube> name.")]
        tube: String,
    },

    #[command(about = "Return the job <id>.")]
    Peek {
        #[arg(index = 1, env, help = "The job <id> to peek.")]
        id: Id,

        #[arg(long, env, help = "Tries to parse the body as UTF-8 encoded bytes.")]
        utf8: bool,
    },

    #[command(about = "Return the next ready job. Operates only on the currently used tube.")]
    PeekReady,

    #[command(
        about = "Return the delayed job with the shortest delay left. Operates only on the currently used tube."
    )]
    PeekDelayed,

    #[command(
        about = "Return the next job in the list of buried jobs. Operates only on the currently used tube."
    )]
    PeekBuried,

    #[command(
        about = "Kicks <n> number of jobs from the currently used tube.",
        long_about = "Kicks <n> number of jobs from the currently used tube.\nThe kick command applies only to the currently used tube.\nIt moves jobs into the ready queue.\nIf there are any buried jobs, it will only kick buried jobs.\nOtherwise it will kick delayed jobs."
    )]
    Kick {
        #[arg(index = 1, help = "Integer upper bound on the number of jobs to kick.")]
        bound: u32,
    },

    #[command(
        about = "The kick-job command is a variant of kick that operates with a single job identified by its job id.",
        long_about = "The kick-job command is a variant of kick that operates with a single job identified by its job id.\nIf the given job id exists and is in a buried or delayed state, it will be moved to the ready queue of\nthe the same tube where it currently belongs."
    )]
    KickJob {
        #[arg(index = 1, help = "The job <id>.")]
        id: Id,
    },

    #[command(
        about = "The stats-job command gives statistical information about the specified job if it exists."
    )]
    StatsJob {
        #[arg(index = 1, help = "The job <id>.")]
        id: Id,
    },

    #[command(
        about = "The stats-tube command gives statistical information about the specified tube if it exists."
    )]
    StatsTube {
        #[arg(index = 1, env, help = "The <tube> name.")]
        tube: String,
    },

    #[command(
        about = "The stats command gives statistical information about the system as a whole."
    )]
    Stats,

    #[command(about = "The list-tubes command returns a list of all existing tubes.")]
    ListTubes,

    #[command(
        about = "The list-tube-used command returns the tube currently being used by the client."
    )]
    ListTubesUsed,

    #[command(
        about = "The list-tubes-watched command returns a list tubes currently being watched by the client."
    )]
    ListTubesWatched,

    #[command(
        about = "The pause-tube command can delay any new job being reserved for a given time."
    )]
    PauseTube {
        #[arg(index = 1, env, help = "The <tube> name.")]
        tube: String,

        #[arg(
            index = 2,
            value_parser = parse_duration,
            env,
            help = "The pause duration in seconds to wait before reserving any more jobs from the queue."

        )]
        delay: Duration,
    },
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
