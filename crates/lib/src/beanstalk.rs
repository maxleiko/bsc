use std::borrow::Cow;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use crate::stats::*;
use crate::Result;

pub type Id = u32;

pub struct Beanstalk {
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
    buf: String,
}

impl Beanstalk {
    pub fn connect<A: ToSocketAddrs>(addr: A) -> Result<Self> {
        let conn = TcpStream::connect(addr)?;
        let read = BufReader::new(conn.try_clone()?);
        let write = BufWriter::new(conn);

        Ok(Self {
            reader: read,
            writer: write,
            buf: String::new(),
        })
    }

    /// The "put" command is for any process that wants to insert a job into the queue.
    /// It comprises a command line followed by the job body:
    ///
    ///     put <pri> <delay> <ttr> <bytes>\r\n
    ///     <data>\r\n
    ///
    /// It inserts a job into the client's currently used tube (see the "use" command
    /// below).
    ///
    ///  - `pri` is an integer < 2**32. Jobs with smaller priority values will be
    ///    scheduled before jobs with larger priorities. The most urgent priority is 0;
    ///    the least urgent priority is 4,294,967,295.
    ///
    ///  - `delay` is an integer number of seconds to wait before putting the job in
    ///    the ready queue. The job will be in the "delayed" state during this time.
    ///    Maximum delay is 2**32-1.
    ///
    ///  - `ttr` -- time to run -- is an integer number of seconds to allow a worker
    ///    to run this job. This time is counted from the moment a worker reserves
    ///    this job. If the worker does not delete, release, or bury the job within
    ///    `ttr` seconds, the job will time out and the server will release the job.
    ///    The minimum ttr is 1. If the client sends 0, the server will silently
    ///    increase the ttr to 1. Maximum ttr is 2**32-1.
    ///
    ///  - `bytes` is an integer indicating the size of the job body, not including the
    ///    trailing "\r\n". This value must be less than max-job-size (default: 2**16).
    ///
    ///  - `data` is the job body -- a sequence of bytes of length `bytes` from the
    ///    previous line.
    pub fn put(&mut self, pri: u32, delay: Duration, ttr: u32, data: &[u8]) -> Result<PutResponse> {
        // request
        write!(
            self.writer,
            "put {pri} {} {ttr} {}\r\n",
            delay.as_secs(),
            data.len()
        )?;
        self.writer.write_all(data)?;
        self.writer.write_all(b"\r\n")?;
        self.writer.flush()?;

        // response
        self.buf.clear();
        self.reader.read_line(&mut self.buf)?;
        PutResponse::parse(self.buf.trim_end_matches("\r\n"))
    }

    /// The "use" command is for producers. Subsequent put commands will put jobs into
    /// the tube specified by this command. If no use command has been issued, jobs
    /// will be put into the tube named "default".
    ///
    ///     use <tube>\r\n
    ///
    ///  - `tube` is a name at most 200 bytes. It specifies the tube to use. If the
    ///    tube does not exist, it will be created.
    pub fn use_(&mut self, tube: &str) -> Result<()> {
        // request
        write!(self.writer, "use {tube}\r\n")?;
        self.writer.flush()?;

        // response
        self.buf.clear();
        self.reader.read_line(&mut self.buf)?;
        Ok(())
    }

    /// A process that wants to consume jobs from the queue uses "reserve", "delete",
    /// "release", and "bury". The first worker command, "reserve", looks like this:
    ///
    ///     reserve\r\n
    ///
    /// Alternatively, you can specify a timeout as follows:
    ///
    ///     reserve-with-timeout <seconds>\r\n
    ///
    /// This will return a newly-reserved job. If no job is available to be reserved,
    /// beanstalkd will wait to send a response until one becomes available. Once a
    /// job is reserved for the client, the client has limited time to run (TTR) the
    /// job before the job times out. When the job times out, the server will put the
    /// job back into the ready queue. Both the TTR and the actual time left can be
    /// found in response to the stats-job command.
    ///
    /// If more than one job is ready, beanstalkd will choose the one with the
    /// smallest priority value. Within each priority, it will choose the one that
    /// was received first.
    ///
    /// A timeout value of 0 will cause the server to immediately return either a
    /// response or TIMED_OUT.  A positive value of timeout will limit the amount of
    /// time the client will block on the reserve request until a job becomes
    /// available.
    pub fn reserve(&mut self, timeout: Option<Duration>) -> Result<ReserveResponse> {
        // request
        match timeout {
            Some(timeout) => write!(
                self.writer,
                "reserve-with-timeout {}\r\n",
                timeout.as_secs()
            )?,
            None => write!(self.writer, "reserve\r\n")?,
        }
        self.writer.flush()?;

        // response
        self.buf.clear();
        self.reader.read_line(&mut self.buf)?;
        match self.buf.trim_end_matches("\r\n") {
            "DEADLINE_SOON" => Ok(ReserveResponse::DeadlineSoon),
            "TIMED_OUT" => Ok(ReserveResponse::TimedOut),
            input => {
                let (id, bytes) = read_reserved(input)?;
                let mut data_reader = (&mut self.reader).take(bytes);
                let mut data = Vec::with_capacity(bytes as usize);
                data_reader.read_to_end(&mut data)?;
                self.reader.read_line(&mut self.buf)?; // read ending \r\n
                Ok(ReserveResponse::Reserved { id, data })
            }
        }
    }

    /// A job can be reserved by its id. Once a job is reserved for the client,
    /// the client has limited time to run (TTR) the job before the job times out.
    /// When the job times out, the server will put the job back into the ready queue.
    /// The command looks like this:
    ///
    ///     reserve-job <id>\r\n
    ///
    /// - `id` is the job id to reserve
    pub fn reserve_by_id(&mut self, id: Id) -> Result<ReserveByIdResponse> {
        // request
        write!(self.writer, "reserve-job {id}\r\n")?;
        self.writer.flush()?;

        // response
        self.buf.clear();
        self.reader.read_line(&mut self.buf)?;
        match self.buf.trim_end_matches("\r\n") {
            "NOT_FOUND" => Ok(ReserveByIdResponse::NotFound),
            input => {
                let (id, bytes) = read_reserved(input)?;
                let mut data_reader = (&mut self.reader).take(bytes);
                let mut data = Vec::with_capacity(bytes as usize);
                data_reader.read_to_end(&mut data)?;
                self.reader.read_line(&mut self.buf)?; // read ending \r\n
                Ok(ReserveByIdResponse::Reserved { id, data })
            }
        }
    }

    /// The delete command removes a job from the server entirely. It is normally used
    /// by the client when the job has successfully run to completion. A client can
    /// delete jobs that it has reserved, ready jobs, delayed jobs, and jobs that are
    /// buried. The delete command looks like this:
    ///
    ///     delete <id>\r\n
    ///
    ///  - `id` is the job id to delete.
    pub fn delete(&mut self, id: Id) -> Result<DeleteResponse> {
        // request
        write!(self.writer, "delete {}\r\n", id)?;
        self.writer.flush()?;

        // response
        self.buf.clear();
        self.reader.read_line(&mut self.buf)?;
        match self.buf.trim_end_matches("\r\n") {
            "DELETED" => Ok(DeleteResponse::Deleted),
            "NOT_FOUND" => Ok(DeleteResponse::NotFound),
            input => Err(input.into()),
        }
    }

    /// The release command puts a reserved job back into the ready queue (and marks
    /// its state as "ready") to be run by any client. It is normally used when the job
    /// fails because of a transitory error. It looks like this:
    ///
    ///     release <id> <pri> <delay>\r\n
    ///
    ///  - `id` is the job id to release.
    ///
    ///  - `pri` is a new priority to assign to the job.
    ///
    ///  - `delay` is an integer number of seconds to wait before putting the job in
    ///    the ready queue. The job will be in the "delayed" state during this time.
    pub fn release(&mut self, id: Id, pri: u32, delay: Duration) -> Result<ReleaseResponse> {
        // request
        write!(self.writer, "release {id} {pri} {}\r\n", delay.as_secs())?;
        self.writer.flush()?;

        // response
        self.buf.clear();
        self.reader.read_line(&mut self.buf)?;
        match self.buf.trim_end_matches("\r\n") {
            "RELEASED" => Ok(ReleaseResponse::Released),
            "BURIED" => Ok(ReleaseResponse::Buried),
            "NOT_FOUND" => Ok(ReleaseResponse::NotFound),
            input => Err(input.into()),
        }
    }

    /// The bury command puts a job into the "buried" state. Buried jobs are put into a
    /// FIFO linked list and will not be touched by the server again until a client
    /// kicks them with the "kick" command.
    ///
    /// The bury command looks like this:
    ///
    ///     bury <id> <pri>\r\n
    ///
    ///  - `id` is the job id to bury.
    ///
    ///  - `pri` is a new priority to assign to the job.
    pub fn bury(&mut self, id: Id, pri: u32) -> Result<BuryResponse> {
        // request
        write!(self.writer, "bury {id} {pri}\r\n")?;
        self.writer.flush()?;

        // response
        self.buf.clear();
        self.reader.read_line(&mut self.buf)?;
        match self.buf.trim_end_matches("\r\n") {
            "BURIED" => Ok(BuryResponse::Buried),
            "NOT_FOUND" => Ok(BuryResponse::NotFound),
            input => Err(input.into()),
        }
    }

    /// The "touch" command allows a worker to request more time to work on a job.
    /// This is useful for jobs that potentially take a long time, but you still want
    /// the benefits of a TTR pulling a job away from an unresponsive worker.  A worker
    /// may periodically tell the server that it's still alive and processing a job
    /// (e.g. it may do this on DEADLINE_SOON). The command postpones the auto
    /// release of a reserved job until TTR seconds from when the command is issued.
    ///
    /// The touch command looks like this:
    ///
    ///     touch <id>\r\n
    ///
    ///  - `id` is the ID of a job reserved by the current connection.
    pub fn touch(&mut self, id: Id) -> Result<TouchResponse> {
        // request
        write!(self.writer, "touch {id}\r\n")?;
        self.writer.flush()?;

        // response
        self.buf.clear();
        self.reader.read_line(&mut self.buf)?;
        match self.buf.trim_end_matches("\r\n") {
            "TOUCHED" => Ok(TouchResponse::Touched),
            "NOT_FOUND" => Ok(TouchResponse::NotFound),
            input => Err(input.into()),
        }
    }

    /// The "watch" command adds the named tube to the watch list for the current
    /// connection. A reserve command will take a job from any of the tubes in the
    /// watch list. For each new connection, the watch list initially consists of one
    /// tube, named "default".
    ///
    ///     watch <tube>\r\n
    ///
    ///  - `tube` is a name at most 200 bytes. It specifies a tube to add to the watch
    ///    list. If the tube doesn't exist, it will be created.
    ///
    /// The response is:
    ///
    ///     WATCHING <count>\r\n
    ///
    /// - `count` is the integer number of tubes currently in the watch list.
    pub fn watch(&mut self, tube: &str) -> Result<usize> {
        // request
        write!(self.writer, "watch {tube}\r\n")?;
        self.writer.flush()?;

        // response
        self.buf.clear();
        self.reader.read_line(&mut self.buf)?;
        let input = self.buf.trim_end_matches("\r\n");
        if let Some(input) = input.strip_prefix("WATCHING ") {
            return Ok(input.parse()?);
        }
        Err(input.into())
    }

    /// The "ignore" command is for consumers. It removes the named tube from the
    /// watch list for the current connection.
    ///
    ///     ignore <tube>\r\n
    pub fn ignore(&mut self, tube: &str) -> Result<IgnoreResponse> {
        // request
        write!(self.writer, "ignore {tube}\r\n")?;
        self.writer.flush()?;

        // response
        self.buf.clear();
        self.reader.read_line(&mut self.buf)?;
        match self.buf.trim_end_matches("\r\n") {
            "NOT_IGNORED" => Ok(IgnoreResponse::NotIgnored),
            input => {
                if let Some(input) = input.strip_prefix("WATCHING ") {
                    return Ok(IgnoreResponse::Count(input.parse()?));
                }

                Err(input.into())
            }
        }
    }

    /// The peek command let the client inspect a job in the system.
    ///
    ///  - "peek <id>\r\n" - return job <id>.
    pub fn peek(&mut self, id: Id) -> Result<PeekResponse> {
        // request
        write!(self.writer, "peek {id}\r\n")?;
        self.peek_internal()
    }

    /// The peek command let the client inspect a job in the system.
    /// Operate only on the currently used tube.
    ///
    ///  - "peek-ready\r\n" - return the next ready job.
    pub fn peek_ready(&mut self) -> Result<PeekResponse> {
        // request
        write!(self.writer, "peek-ready\r\n")?;
        self.peek_internal()
    }

    /// The peek command let the client inspect a job in the system.
    /// Operate only on the currently used tube.
    ///
    ///  - "peek-delayed\r\n" - return the delayed job with the shortest delay left.
    pub fn peek_delayed(&mut self) -> Result<PeekResponse> {
        // request
        write!(self.writer, "peek-delayed\r\n")?;
        self.peek_internal()
    }

    /// The peek command let the client inspect a job in the system.
    /// Operate only on the currently used tube.
    ///
    ///  - "peek-buried\r\n" - return the next job in the list of buried jobs.
    pub fn peek_buried(&mut self) -> Result<PeekResponse> {
        // request
        write!(self.writer, "peek-buried\r\n")?;
        self.peek_internal()
    }

    /// Every peek commands work the same, so once the "command" is written
    /// to the `self.writer`, we can generalize the response behavior
    fn peek_internal(&mut self) -> Result<PeekResponse> {
        self.writer.flush()?;

        // response
        self.buf.clear();
        self.reader.read_line(&mut self.buf)?;
        match self.buf.trim_end_matches("\r\n") {
            "NOT_FOUND" => Ok(PeekResponse::NotFound),
            input => {
                let (id, bytes) = read_found(input)?;
                let mut data_reader = (&mut self.reader).take(bytes);
                let mut data = Vec::with_capacity(bytes as usize);
                data_reader.read_to_end(&mut data)?;
                self.reader.read_line(&mut self.buf)?; // read ending \r\n
                Ok(PeekResponse::Found { id, data })
            }
        }
    }

    /// The kick command applies only to the currently used tube. It moves jobs into
    /// the ready queue. If there are any buried jobs, it will only kick buried jobs.
    /// Otherwise it will kick delayed jobs. It looks like:
    ///
    ///     kick <bound>\r\n
    ///
    ///  - `bound` is an integer upper bound on the number of jobs to kick. The server
    ///    will kick no more than <bound> jobs.
    ///
    /// The response is of the form:
    ///
    ///     KICKED <count>\r\n
    ///
    ///  - `count` is an integer indicating the number of jobs actually kicked.
    pub fn kick(&mut self, bound: u32) -> Result<usize> {
        // request
        write!(self.writer, "kick {bound}\r\n")?;
        self.writer.flush()?;

        // response
        self.buf.clear();
        self.reader.read_line(&mut self.buf)?;
        let input = self.buf.trim_end_matches("\r\n");
        if let Some(input) = input.strip_prefix("KICKED ") {
            return Ok(input.parse()?);
        }
        Err(input.into())
    }

    /// The kick-job command is a variant of kick that operates with a single job
    /// identified by its job id. If the given job id exists and is in a buried or
    /// delayed state, it will be moved to the ready queue of the the same tube where it
    /// currently belongs. The syntax is:
    ///
    ///     kick-job <id>\r\n
    ///
    ///  - <id> is the job id to kick.
    pub fn kick_job(&mut self, id: Id) -> Result<KickJobResponse> {
        // request
        write!(self.writer, "kick-job {id}\r\n")?;
        self.writer.flush()?;

        // response
        self.buf.clear();
        self.reader.read_line(&mut self.buf)?;
        match self.buf.trim_end_matches("\r\n") {
            "KICKED" => Ok(KickJobResponse::Kicked),
            "NOT_FOUND" => Ok(KickJobResponse::NotFound),
            input => Err(input.into()),
        }
    }

    /// The stats-job command gives statistical information about the specified job if
    /// it exists. Its form is:
    ///
    ///     stats-job <id>\r\n
    ///
    ///  - <id> is a job id.
    pub fn stats_job(&mut self, id: Id) -> Result<StatsJobResponse> {
        // request
        write!(self.writer, "stats-job {id}\r\n")?;
        self.writer.flush()?;

        // response
        self.buf.clear();
        self.reader.read_line(&mut self.buf)?;
        match self.buf.trim_end_matches("\r\n") {
            "NOT_FOUND" => Ok(StatsJobResponse::NotFound),
            input => {
                let bytes = read_ok(input)?;
                let mut data_reader = (&mut self.reader).take(bytes);
                let mut data = Vec::with_capacity(bytes as usize);
                data_reader.read_to_end(&mut data)?;
                self.reader.read_line(&mut self.buf)?; // read ending \r\n
                Ok(StatsJobResponse::Ok(serde_yaml::from_slice(&data)?))
            }
        }
    }

    /// The stats-tube command gives statistical information about the specified tube
    /// if it exists. Its form is:
    ///
    ///     stats-tube <tube>\r\n
    ///
    ///  - <tube> is a name at most 200 bytes. Stats will be returned for this tube.
    pub fn stats_tube(&mut self, tube: &str) -> Result<StatsTubeResponse> {
        // request
        write!(self.writer, "stats-tube {tube}\r\n")?;
        self.writer.flush()?;

        // response
        self.buf.clear();
        self.reader.read_line(&mut self.buf)?;
        match self.buf.trim_end_matches("\r\n") {
            "NOT_FOUND" => Ok(StatsTubeResponse::NotFound),
            input => {
                let bytes = read_ok(input)?;
                let mut data_reader = (&mut self.reader).take(bytes);
                let mut data = Vec::with_capacity(bytes as usize);
                data_reader.read_to_end(&mut data)?;
                self.reader.read_line(&mut self.buf)?; // read ending \r\n
                Ok(StatsTubeResponse::Ok(serde_yaml::from_slice(&data)?))
            }
        }
    }

    /// The stats command gives statistical information about the system as a whole.
    /// Its form is:
    ///
    ///     stats\r\n
    pub fn stats(&mut self) -> Result<Stats> {
        // request
        write!(self.writer, "stats\r\n")?;
        self.writer.flush()?;

        // response
        self.buf.clear();
        self.reader.read_line(&mut self.buf)?;
        let input = self.buf.trim_end_matches("\r\n");
        let bytes = read_ok(input)?;
        let mut data_reader = (&mut self.reader).take(bytes);
        let mut data = Vec::with_capacity(bytes as usize);
        data_reader.read_to_end(&mut data)?;
        self.reader.read_line(&mut self.buf)?; // read ending \r\n
        Ok(serde_yaml::from_slice(&data)?)
    }

    /// The list-tubes command returns a list of all existing tubes. Its form is:
    ///
    ///       list-tubes\r\n
    pub fn list_tubes(&mut self) -> Result<Vec<Cow<'_, str>>> {
        // request
        write!(self.writer, "list-tubes\r\n")?;
        self.writer.flush()?;

        // response
        self.buf.clear();
        self.reader.read_line(&mut self.buf)?;
        let input = self.buf.trim_end_matches("\r\n");
        let bytes = read_ok(input)?;
        let mut data_reader = (&mut self.reader).take(bytes);
        let mut data = Vec::with_capacity(bytes as usize);
        data_reader.read_to_end(&mut data)?;
        self.reader.read_line(&mut self.buf)?; // read ending \r\n
        Ok(serde_yaml::from_slice(&data)?)
    }

    /// The list-tube-used command returns the tube currently being used by the
    /// client. Its form is:
    ///
    ///     list-tube-used\r\n
    pub fn list_tube_used(&mut self) -> Result<Cow<'_, str>> {
        // request
        write!(self.writer, "list-tube-used\r\n")?;
        self.writer.flush()?;

        // response
        self.buf.clear();
        self.reader.read_line(&mut self.buf)?;
        let input = self.buf.trim_end_matches("\r\n");
        if let Some(input) = input.strip_prefix("USING ") {
            return Ok(Cow::Borrowed(input));
        }
        Err(input.into())
    }

    /// The list-tubes-watched command returns a list tubes currently being watched by
    /// the client. Its form is:
    ///
    ///     list-tubes-watched\r\n
    pub fn list_tube_watched(&mut self) -> Result<Vec<Cow<'_, str>>> {
        // request
        write!(self.writer, "list-tubes-watched\r\n")?;
        self.writer.flush()?;

        // response
        self.buf.clear();
        self.reader.read_line(&mut self.buf)?;
        let input = self.buf.trim_end_matches("\r\n");
        let bytes = read_ok(input)?;
        let mut data_reader = (&mut self.reader).take(bytes);
        let mut data = Vec::with_capacity(bytes as usize);
        data_reader.read_to_end(&mut data)?;
        self.reader.read_line(&mut self.buf)?; // read ending \r\n
        Ok(serde_yaml::from_slice(&data)?)
    }

    /// The pause-tube command can delay any new job being reserved for a given time. Its form is:
    ///
    ///      pause-tube <tube-name> <delay>\r\n
    ///
    /// - `tube` is the tube to pause
    ///
    /// - `delay` is an integer number of seconds < 2**32 to wait before reserving any more
    ///   jobs from the queue
    pub fn pause_tube(&mut self, tube: &str, delay: Duration) -> Result<PauseTubeResponse> {
        // request
        write!(self.writer, "pause-tube {tube} {}\r\n", delay.as_secs())?;
        self.writer.flush()?;

        // response
        self.buf.clear();
        self.reader.read_line(&mut self.buf)?;
        match self.buf.trim_end_matches("\r\n") {
            "PAUSED" => Ok(PauseTubeResponse::Paused),
            "NOT_FOUND" => Ok(PauseTubeResponse::NotFound),
            err => Err(err.into()),
        }
    }

    /// The quit command simply closes the connection. Its form is:
    ///
    ///      quit\r\n
    pub fn quit(mut self) -> Result<()> {
        write!(self.writer, "quit\r\n")?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum PutResponse {
    /// Indicates success, `id` is the integer id of the new job.
    Inserted(Id),
    /// The server ran out of memory trying to grow the priority queue data structure.
    /// `id` is the integer id of the new job.
    Buried(Id),
    /// The job body must be followed by a CR-LF pair, that is, "\r\n".
    /// These two bytes are not counted in the job size given by the client in the put command line.
    ExpectedCrlf,
    /// The client has requested to put a job with a body larger than max-job-size bytes.
    JobTooBig,
    /// This means that the server has been put into "drain mode" and
    /// is no longer accepting new jobs. The client should try another server or
    /// disconnect and try again later. To put the server in drain mode, send the
    /// SIGUSR1 signal to the process.
    Draining,
}

impl PutResponse {
    pub fn parse(input: &str) -> Result<Self> {
        if let Some(input) = input.strip_prefix("INSERTED ") {
            return Ok(PutResponse::Inserted(input.parse()?));
        }
        if let Some(input) = input.strip_prefix("BURIED ") {
            return Ok(PutResponse::Buried(input.parse()?));
        }
        if input.starts_with("EXPECTED_CRLF") {
            return Ok(PutResponse::ExpectedCrlf);
        }
        if input.starts_with("JOB_TOO_BIG") {
            return Ok(PutResponse::JobTooBig);
        }
        if input.starts_with("DRAINING") {
            return Ok(PutResponse::Draining);
        }
        Err(input.into())
    }
}

#[derive(Debug)]
pub enum ReserveResponse {
    /// During the TTR of a reserved job, the last second is kept by the server as a
    /// safety margin, during which the client will not be made to wait for another
    /// job. If the client issues a reserve command during the safety margin, or if
    /// the safety margin arrives while the client is waiting on a reserve command
    DeadlineSoon,
    /// If a non-negative timeout was specified and the timeout exceeded before a job
    /// became available, or if the client's connection is half-closed, the server
    /// will respond with TIMED_OUT.
    TimedOut,
    /// Successful reservation
    Reserved {
        /// the job id -- an integer unique to this job in this instance of beanstalkd
        id: Id,
        /// a sequence of bytes of length `bytes` from the
        /// previous line. This is a verbatim copy of the bytes that were originally
        /// sent to the server in the put command for this job
        data: Vec<u8>,
    },
}

#[derive(Debug)]
pub enum ReserveByIdResponse {
    /// If the job does not exist or reserved by a client or
    /// is not either ready, buried or delayed.
    NotFound,
    /// Successful reservation
    Reserved {
        /// the job id -- an integer unique to this job in this instance of beanstalkd
        id: Id,
        /// a sequence of bytes of length `bytes` from the
        /// previous line. This is a verbatim copy of the bytes that were originally
        /// sent to the server in the put command for this job
        data: Vec<u8>,
    },
}

#[inline]
fn read_reserved(input: &str) -> Result<(Id, u64)> {
    if let Some(input) = input.strip_prefix("RESERVED ") {
        let mut iter = input.split_ascii_whitespace();
        let id = iter
            .next()
            .map(|s| s.parse::<u32>())
            .ok_or("missing 'id' in RESERVED response")??;
        let bytes = iter
            .next()
            .map(|s| s.parse::<u64>())
            .ok_or("missing 'bytes' in RESERVED response")??;

        return Ok((id, bytes));
    }
    Err(input.into())
}

#[derive(Debug)]
pub enum DeleteResponse {
    /// Indicate success
    Deleted,
    /// If the job does not exist or is not either reserved by the
    /// client, ready, or buried. This could happen if the job timed out before the
    /// client sent the delete command.
    NotFound,
}

#[derive(Debug)]
pub enum ReleaseResponse {
    /// Indicate success.
    Released,
    /// If the server ran out of memory trying to grow the priority
    /// queue data structure.
    Buried,
    /// If the job does not exist or is not reserved by the client.
    NotFound,
}

#[derive(Debug)]
pub enum BuryResponse {
    /// Indicate success
    Buried,
    /// If the job does not exist or is not reserved by the client.
    NotFound,
}

#[derive(Debug)]
pub enum TouchResponse {
    /// Indicate success
    Touched,
    /// If the job does not exist or is not reserved by the client.
    NotFound,
}

#[derive(Debug)]
pub enum IgnoreResponse {
    /// Is the integer number of tubes currently in the watch list.
    Count(usize),
    /// If the client attempts to ignore the only tube in its watch list.
    NotIgnored,
}

#[derive(Debug)]
pub enum PeekResponse {
    /// If the requested job doesn't exist or there are no jobs in
    /// the requested state.
    NotFound,
    /// Indicate success
    Found {
        /// The job id.
        id: Id,
        /// a sequence of bytes of length `bytes` from the
        /// previous line.
        data: Vec<u8>,
    },
}

#[inline]
fn read_found(input: &str) -> Result<(Id, u64)> {
    if let Some(input) = input.strip_prefix("FOUND ") {
        let mut iter = input.split_ascii_whitespace();
        let id = iter
            .next()
            .map(|s| s.parse::<u32>())
            .ok_or("missing 'id' in FOUND response")??;
        let bytes = iter
            .next()
            .map(|s| s.parse::<u64>())
            .ok_or("missing 'bytes' in FOUND response")??;

        return Ok((id, bytes));
    }
    Err(input.into())
}

#[derive(Debug)]
pub enum KickJobResponse {
    /// If the job does not exist or is not in a kickable state. This
    /// can also happen upon internal errors.
    NotFound,
    /// Indicate success
    Kicked,
}

#[derive(Debug)]
pub enum StatsJobResponse {
    /// Indicate success
    ///
    /// Statistical information represented by a dictionary.
    Ok(StatsJob),
    /// If the job does not exist.
    NotFound,
}

#[inline]
fn read_ok(input: &str) -> Result<u64> {
    if let Some(input) = input.strip_prefix("OK ") {
        return Ok(input.parse::<u64>()?);
    }
    Err(input.into())
}

#[derive(Debug)]
pub enum StatsTubeResponse {
    /// Indicate success
    ///
    /// Statistical information represented by a dictionary.
    Ok(StatsTube),
    /// If the tube does not exist.
    NotFound,
}

#[derive(Debug)]
pub enum PauseTubeResponse {
    /// Indicate success
    Paused,
    /// If the tube does not exist.
    NotFound,
}
