use std::time::Duration;

use serde::Deserialize;

use crate::Id;

#[derive(Debug, Deserialize)]
pub struct StatsJob {
    /// "id" is the job id
    pub id: Id,
    /// "tube" is the name of the tube that contains this job
    pub tube: String,
    /// "state" is "ready" or "delayed" or "reserved" or "buried"
    pub state: State,
    /// "pri" is the priority value set by the put, release, or bury commands.
    pub pri: u32,
    /// "age" is the time in seconds since the put command that created this job.
    #[serde(deserialize_with = "from_seconds")]
    pub age: Duration,
    /// "delay" is the integer number of seconds to wait before putting this job in
    ///   the ready queue.
    #[serde(deserialize_with = "from_seconds")]
    pub delay: Duration,
    /// "ttr" -- time to run -- is the integer number of seconds a worker is
    ///   allowed to run this job.
    pub ttr: u32,
    /// "time-left" is the number of seconds left until the server puts this job
    ///   into the ready queue. This number is only meaningful if the job is
    ///   reserved or delayed. If the job is reserved and this amount of time
    ///   elapses before its state changes, it is considered to have timed out.
    #[serde(rename = "time-left", deserialize_with = "from_seconds")]
    pub time_left: Duration,
    /// "file" is the number of the earliest binlog file containing this job.
    ///   If -b wasn't used, this will be 0.
    pub file: u32,
    /// "reserves" is the number of times this job has been reserved.
    pub reserves: u32,
    /// "timeouts" is the number of times this job has timed out during a
    ///   reservation.
    pub timeouts: u32,
    /// "releases" is the number of times a client has released this job from a
    ///   reservation.
    pub releases: u32,
    /// "buries" is the number of times this job has been buried.
    pub buries: u32,
    /// "kicks" is the number of times this job has been kicked.
    pub kicks: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum State {
    Ready,
    Delayed,
    Reserved,
    Buried,
}

#[derive(Debug, Deserialize)]
pub struct StatsTube {
    /// "name" is the tube's name.
    pub name: String,
    /// "current-jobs-urgent" is the number of ready jobs with priority < 1024 in
    /// this tube.
    #[serde(rename = "current-jobs-urgent")]
    pub current_jobs_urgent: u32,
    /// "current-jobs-ready" is the number of jobs in the ready queue in this tube.
    #[serde(rename = "current-jobs-ready")]
    pub current_jobs_ready: u32,
    /// "current-jobs-reserved" is the number of jobs reserved by all clients in
    /// this tube.
    #[serde(rename = "current-jobs-reserved")]
    pub current_jobs_reserved: u32,
    /// "current-jobs-delayed" is the number of delayed jobs in this tube.
    #[serde(rename = "current-jobs-delayed")]
    pub current_jobs_delayed: u32,
    /// "current-jobs-buried" is the number of buried jobs in this tube.
    #[serde(rename = "current-jobs-buried")]
    pub current_jobs_buried: u32,
    /// "total-jobs" is the cumulative count of jobs created in this tube in
    ///  the current beanstalkd process.
    #[serde(rename = "total-jobs")]
    pub total_jobs: u32,
    /// "current-using" is the number of open connections that are currently
    ///  using this tube.
    #[serde(rename = "current-using")]
    pub current_using: u32,
    /// "current-waiting" is the number of open connections that have issued a
    ///  reserve command while watching this tube but not yet received a response.
    #[serde(rename = "current-waiting")]
    pub current_waiting: u32,
    /// "current-watching" is the number of open connections that are currently
    ///  watching this tube.
    #[serde(rename = "current-watching")]
    pub current_watching: u32,
    /// "pause" is the number of seconds the tube has been paused for.
    pub pause: u32,
    /// "cmd-delete" is the cumulative number of delete commands for this tube
    #[serde(rename = "cmd-delete")]
    pub cmd_delete: u32,
    /// "cmd-pause-tube" is the cumulative number of pause-tube commands for this tube.
    #[serde(rename = "cmd-pause-tube")]
    pub cmd_pause_tube: u32,
    /// "pause-time-left" is the number of seconds until the tube is un-paused.
    #[serde(
        rename = "pause-time-left",
        serialize_with = "as_seconds",
        deserialize_with = "from_seconds"
    )]
    pub pause_time_left: Duration,
}

#[derive(Debug, Deserialize)]
pub struct Stats {
    /// "current-jobs-urgent" is the number of ready jobs with priority < 1024.
    #[serde(rename = "current-jobs-urgent")]
    pub current_jobs_urgent: u32,
    /// "current-jobs-ready" is the number of jobs in the ready queue.
    #[serde(rename = "current-jobs-ready")]
    pub current_jobs_ready: u32,
    /// "current-jobs-reserved" is the number of jobs reserved by all clients.
    #[serde(rename = "current-jobs-reserved")]
    pub current_jobs_reserved: u32,
    /// "current-jobs-delayed" is the number of delayed jobs.
    #[serde(rename = "current-jobs-delayed")]
    pub current_jobs_delayed: u32,
    /// "current-jobs-buried" is the number of buried jobs.
    #[serde(rename = "current-jobs-buried")]
    pub current_jobs_buried: u32,
    /// "cmd-put" is the cumulative number of put commands.
    #[serde(rename = "cmd-put")]
    pub cmd_put: u32,
    /// "cmd-peek" is the cumulative number of peek commands.
    #[serde(rename = "cmd-peek")]
    pub cmd_peek: u32,
    /// "cmd-peek-ready" is the cumulative number of peek-ready commands.
    #[serde(rename = "cmd-peek-ready")]
    pub cmd_peek_ready: u32,
    /// "cmd-peek-delayed" is the cumulative number of peek-delayed commands.
    #[serde(rename = "cmd-peek-delayed")]
    pub cmd_peek_delayed: u32,
    /// "cmd-peek-buried" is the cumulative number of peek-buried commands.
    #[serde(rename = "cmd-peek-buried")]
    pub cmd_peek_buried: u32,
    /// "cmd-reserve" is the cumulative number of reserve commands.
    #[serde(rename = "cmd-reserve")]
    pub cmd_reserve: u32,
    /// "cmd-use" is the cumulative number of use commands.
    #[serde(rename = "cmd-use")]
    pub cmd_use: u32,
    /// "cmd-watch" is the cumulative number of watch commands.
    #[serde(rename = "cmd-watch")]
    pub cmd_watch: u32,
    /// "cmd-ignore" is the cumulative number of ignore commands.
    #[serde(rename = "cmd-ignore")]
    pub cmd_ignore: u32,
    /// "cmd-delete" is the cumulative number of delete commands.
    #[serde(rename = "cmd-delete")]
    pub cmd_delete: u32,
    /// "cmd-release" is the cumulative number of release commands.
    #[serde(rename = "cmd-release")]
    pub cmd_release: u32,
    /// "cmd-bury" is the cumulative number of bury commands.
    #[serde(rename = "cmd-bury")]
    pub cmd_bury: u32,
    /// "cmd-kick" is the cumulative number of kick commands.
    #[serde(rename = "cmd-kick")]
    pub cmd_kick: u32,
    /// "cmd-stats" is the cumulative number of stats commands.
    #[serde(rename = "cmd-stats")]
    pub cmd_stats: u32,
    /// "cmd-stats-job" is the cumulative number of stats-job commands.
    #[serde(rename = "cmd-stats-job")]
    pub cmd_stats_job: u32,
    /// "cmd-stats-tube" is the cumulative number of stats-tube commands.
    #[serde(rename = "cmd-stats-tube")]
    pub cmd_stats_tube: u32,
    /// "cmd-list-tubes" is the cumulative number of list-tubes commands.
    #[serde(rename = "cmd-list-tubes")]
    pub cmd_list_tubes: u32,
    /// "cmd-list-tube-used" is the cumulative number of list-tube-used commands.
    #[serde(rename = "cmd-list-tube-used")]
    pub cmd_list_tube_used: u32,
    /// "cmd-list-tubes-watched" is the cumulative number of list-tubes-watched commands.
    #[serde(rename = "cmd-list-tubes-watched")]
    pub cmd_list_tubes_watched: u32,
    /// "cmd-pause-tube" is the cumulative number of pause-tube commands.
    #[serde(rename = "cmd-pause-tube")]
    pub cmd_pause_tube: u32,
    /// "job-timeouts" is the cumulative count of times a job has timed out.
    #[serde(rename = "job-timeouts")]
    pub job_timeouts: u32,
    /// "total-jobs" is the cumulative count of jobs created.
    #[serde(rename = "total-jobs")]
    pub total_jobs: u32,
    /// "max-job-size" is the maximum number of bytes in a job.
    #[serde(rename = "max-job-size")]
    pub max_job_size: u32,
    /// "current-tubes" is the number of currently-existing tubes.
    #[serde(rename = "current-tubes")]
    pub current_tubes: u32,
    /// "current-connections" is the number of currently open connections.
    #[serde(rename = "current-connections")]
    pub current_connections: u32,
    /// "current-producers" is the number of open connections that have each issued at least one put command.
    #[serde(rename = "current-producers")]
    pub current_producers: u32,
    /// "current-workers" is the number of open connections that have each issued at least one reserve command.
    #[serde(rename = "current-workers")]
    pub current_workers: u32,
    /// "current-waiting" is the number of open connections that have issued a reserve command but not yet received a response.
    #[serde(rename = "current-waiting")]
    pub current_waiting: u32,
    /// "total-connections" is the cumulative count of connections.
    #[serde(rename = "total-connections")]
    pub total_connections: u32,
    /// "pid" is the process id of the server.
    #[serde(rename = "pid")]
    pub pid: u32,
    /// "version" is the version string of the server.
    #[serde(rename = "version")]
    pub version: String,
    /// "rusage-utime" is the cumulative user CPU time of this process in seconds and microseconds.
    #[serde(rename = "rusage-utime")]
    pub rusage_utime: f32,
    /// "rusage-stime" is the cumulative system CPU time of this process in seconds and microseconds.
    #[serde(rename = "rusage-stime")]
    pub rusage_stime: f32,
    /// "uptime" is the number of seconds since this server process started running.
    #[serde(rename = "uptime", deserialize_with = "from_seconds")]
    pub uptime: Duration,
    /// "binlog-oldest-index" is the index of the oldest binlog file needed to store the current jobs.
    #[serde(rename = "binlog-oldest-index")]
    pub binlog_oldest_index: usize,
    /// "binlog-current-index" is the index of the current binlog file being written to. If binlog is not active this value will be 0.
    #[serde(rename = "binlog-current-index")]
    pub binlog_current_index: usize,
    /// "binlog-max-size" is the maximum size in bytes a binlog file is allowed to get before a new binlog file is opened.
    #[serde(rename = "binlog-max-size")]
    pub binlog_max_size: usize,
    /// "binlog-records-written" is the cumulative number of records written to the binlog.
    #[serde(rename = "binlog-records-written")]
    pub binlog_records_written: u32,
    /// "binlog-records-migrated" is the cumulative number of records written as part of compaction.
    #[serde(rename = "binlog-records-migrated")]
    pub binlog_records_migrated: u32,
    /// "draining" is set to "true" if the server is in drain mode, "false" otherwise.
    #[serde(default)]
    pub draining: bool,
    /// "id" is a random id string for this server process, generated every time beanstalkd process starts.
    pub id: String,
    /// "hostname" is the hostname of the machine as determined by uname.
    pub hostname: String,
    /// "os" is the OS version as determined by uname
    pub os: Option<String>,
    /// "platform" is the machine architecture as determined by uname
    pub platform: Option<String>,
}

pub fn from_seconds<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: serde::Deserializer<'de>,
{
    u64::deserialize(deserializer).map(Duration::from_secs)
}
