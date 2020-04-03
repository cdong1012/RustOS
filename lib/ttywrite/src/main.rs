mod parsers;

use serial;
use structopt;
use structopt_derive::StructOpt;
use xmodem::Xmodem;
use xmodem::Progress;
use std::path::PathBuf;
use std::time::Duration;

use structopt::StructOpt;
use serial::core::{CharSize, BaudRate, StopBits, FlowControl, SerialDevice, SerialPortSettings};

use parsers::{parse_width, parse_stop_bits, parse_flow_control, parse_baud_rate};

#[derive(StructOpt, Debug)]
#[structopt(about = "Write to TTY using the XMODEM protocol by default.")]
struct Opt {
    #[structopt(short = "i", help = "Input file (defaults to stdin if not set)", parse(from_os_str))]
    input: Option<PathBuf>,

    #[structopt(short = "b", long = "baud", parse(try_from_str = "parse_baud_rate"),
                help = "Set baud rate", default_value = "115200")]
    baud_rate: BaudRate, //set

    #[structopt(short = "t", long = "timeout", parse(try_from_str),
                help = "Set timeout in seconds", default_value = "10")]
    timeout: u64, //set

    #[structopt(short = "w", long = "width", parse(try_from_str = "parse_width"),
                help = "Set data character width in bits", default_value = "8")]
    char_width: CharSize, //set 

    #[structopt(help = "Path to TTY device", parse(from_os_str))]
    tty_path: PathBuf,

    #[structopt(short = "f", long = "flow-control", parse(try_from_str = "parse_flow_control"),
                help = "Enable flow control ('hardware' or 'software')", default_value = "none")]
    flow_control: FlowControl, //set

    #[structopt(short = "s", long = "stop-bits", parse(try_from_str = "parse_stop_bits"),
                help = "Set number of stop bits", default_value = "1")]
    stop_bits: StopBits, // set

    #[structopt(short = "r", long = "raw", help = "Disable XMODEM")]
    raw: bool,
}

fn main() {
    use std::fs::File;
    use std::io::{self, BufReader};
    use std::convert::TryInto;
    let opt = Opt::from_args();
    let mut port = serial::open(&opt.tty_path).expect("path points to invalid TTY");
    let mut port_setting = port.read_settings().expect("Can't read setting");
    // port configuration according to opt variable
    port_setting.set_baud_rate(opt.baud_rate).expect("Can't set baud rate");
    port_setting.set_flow_control(opt.flow_control);
    port_setting.set_char_size(opt.char_width);
    port_setting.set_stop_bits(opt.stop_bits);
    port.write_settings(&port_setting).expect("bad settings write");
    port.set_timeout(std::time::Duration::from_secs(opt.timeout));
    // reader configuration 
    let mut reader : Box<io::Read> = match opt.input {
        Some(file_name) => {
            let f = File::open(file_name).expect("File not found. Can't open");
            Box::new(BufReader::new(f))
        },
        None            => Box::new(io::stdin())
    };
    // if opt raw, use io::copy which return the written bytes
    let written_byte= if opt.raw {
        io::copy(&mut reader, &mut port).expect("Can't copy raw from reader")
    } else {
    // if not raw, use xmodem transmit with progress,
        Xmodem::transmit_with_progress(&mut reader, &mut port, progress_fn).expect("Can't
            transmit with xmodem") as u64
    };
    println!("wrote {:?} bytes to input", written_byte);
}

fn progress_fn(progress: Progress) {
    println!("Progress: {:?}", progress);
} 