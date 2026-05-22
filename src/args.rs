use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "lsuser",
    author = "Vincent Yang",
    version,
    about = "List system users in a clean, block-device-like columnar layout.",
    help_template = "Usage:\n    {usage}\n\nOptions:\n{options}"
)]
pub struct Args {
    #[arg(short, long, help = "Disable built-in filters and list all system daemon accounts.")]
    pub all: bool,

    #[arg(short, long, help = "Do not print a header line.")]
    pub noheadings: bool,

    #[arg(
        short,
        long,
        value_name = "LIST",
        help = "Specify which output columns to print (comma-separated). Available: USER,UID,GID,PRIMARY_GROUP,ALL_GROUP,REAL_NAME,HOME,SHELL."
    )]
    pub output: Option<String>,

    #[arg(short = 'O', long, help = "Output all available columns.")]
    pub output_all: bool,

    #[arg(short = 'J', long, help = "Use JSON output format.")]
    pub json: bool,

    #[arg(long, value_name = "RANGE", help = "Filter by UID range (e.g. 0, 0-1000, 1000-).")]
    pub uid: Option<String>,

    #[arg(short = 'g', long, value_name = "RANGE", help = "Filter by GID range (e.g. 0, 0-1000, 1000-).")]
    pub gid: Option<String>,

    #[arg(long, value_name = "NAME", help = "Filter by group name.")]
    pub group: Option<String>,

    #[arg(long, help = "Display all group memberships in an ALL_GROUP column.")]
    pub groups: bool,
}
