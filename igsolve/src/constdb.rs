#[derive(clap::Args)]
pub struct ConstDBConf {
    /// Number of endgame database segments
    #[arg(long="edb_segments", default_value_t = 0)]
    pub segments: u32,
}