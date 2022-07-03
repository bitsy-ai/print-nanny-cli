#[macro_use]
extern crate clap;

use anyhow::Result;
use clap::{Arg, ArgMatches, Command};
use env_logger::Builder;
use git_version::git_version;
use log::LevelFilter;
use printnanny_gst::options::{AppModeOption, SinkOption, SrcOption, VideoEncodingOption};

fn main() -> Result<()> {
    // include git sha in version, which requires passing a boxed string to clap's .version() builder
    let version = Box::leak(format!("{} {}", crate_version!(), git_version!()).into_boxed_str());

    // parse args
    let app_name = "printnanny-gst";

    let default_sink = SinkOption::Udpsink.to_string();
    let default_src = SrcOption::Libcamerasrc.to_string();
    let default_app = AppModeOption::RtpVideo.to_string();
    let app = Command::new(app_name)
        .author(crate_authors!())
        .about(crate_description!())
        .version(&version[..])
        .subcommand_required(true)
        // generic app args
        .arg(
            Arg::new("v")
                .short('v')
                .multiple_occurrences(true)
                .help("Sets the level of verbosity"),
        )
        .arg(
            Arg::new("app")
                .default_value(&default_app)
                .possible_values(AppModeOption::possible_values())
                .help("Application mode to run"),
        )
        .arg(
            Arg::new("height")
                .long("height")
                .default_value("480")
                .takes_value(true)
                .help("Input resolution height"),
        )
        .arg(
            Arg::new("width")
                .long("width")
                .default_value("640")
                .takes_value(true)
                .help("Input resolution width"),
        )
        .arg(
            Arg::new("src")
                .long("src")
                .required(true)
                .takes_value(true)
                .possible_values(SrcOption::possible_values())
                .help(""),
        )
        .arg(
            Arg::new("encoder")
                .short('e')
                .long("encoder")
                .required(true)
                .takes_value(true)
                .possible_values(VideoEncodingOption::possible_values())
                .help("Run TensorFlow lite model on output"),
        )
        .arg(
            Arg::new("sink")
                .long("sink")
                .required(true)
                .takes_value(true)
                .default_value(&default_sink)
                .possible_values(SinkOption::possible_values())
                .help("Gstreamer sink"),
        )
        .arg(
            Arg::new("host")
                .long("host")
                .default_value("localhost")
                .takes_value(true)
                .required_if("sink", "udpsink")
                .help("udpsink host value"),
        )
        .arg(
            Arg::new("video_port")
                .long("video-port")
                .default_value("5104")
                .takes_value(true)
                .required_if("sink", "udpsink")
                .help("udpsink port value (original video stream)"),
        )
        .arg(
            Arg::new("overlay_port")
                .long("overlay-port")
                .default_value("5106")
                .takes_value(true)
                .required_if("sink", "udpsink")
                .help("udpsink port value (inference video overlay)"),
        )
        .arg(
            Arg::new("data_port")
                .long("data-port")
                .default_value("5107")
                .takes_value(true)
                .required_if("sink", "udpsink")
                .help("udpsink port value (inference tensor data)"),
        )
        .arg(
            Arg::new("tflite_model")
                .long("tflite-model")
                .default_value("/usr/share/printnanny/model/model.tflite")
                .takes_value(true)
                .required_if_eq_any(&[("app", "rtptfliteoverlay"), ("app", "rtptflitecomposite")])
                .help("Path to model.tflite file"),
        )
        .arg(
            Arg::new("tflite_labels")
                .long("tflite-labels")
                .default_value("/usr/share/printnanny/model/dict.txt")
                .takes_value(true)
                .required_if_eq_any(&[("app", "rtptfliteoverlay"), ("app", "rtptflitecomposite")])
                .help("Path to tflite labels file"),
        )
        .arg(
            Arg::new("tensor_height")
                .long("tensor-height")
                .default_value("320")
                .takes_value(true)
                .required_if_eq_any(&[("app", "rtptfliteoverlay"), ("app", "rtptflitecomposite")])
                .help("Height of input tensor"),
        )
        .arg(
            Arg::new("tensor_width")
                .long("tensor-width")
                .default_value("320")
                .takes_value(true)
                .required_if_eq_any(&[("app", "rtptfliteoverlay"), ("app", "rtptflitecomposite")])
                .help("Width of input tensor"),
        );

    let app_m = app.get_matches();
    // Vary the output based on how many times the user used the "verbose" flag
    // (i.e. 'printnanny v v v' or 'printnanny vvv' vs 'printnanny v'
    let verbosity = app_m.occurrences_of("v");
    let mut builder = Builder::new();
    match verbosity {
        0 => {
            builder.filter_level(LevelFilter::Warn).init();
            gst::debug_set_default_threshold(gst::DebugLevel::Warning);
        }
        1 => {
            builder.filter_level(LevelFilter::Info).init();
            gst::debug_set_default_threshold(gst::DebugLevel::Info);
        }
        2 => {
            builder.filter_level(LevelFilter::Debug).init();
            gst::debug_set_default_threshold(gst::DebugLevel::Debug);
        }
        _ => {
            gst::debug_set_default_threshold(gst::DebugLevel::Trace);
            builder.filter_level(LevelFilter::Trace).init()
        }
    };

    // Initialize GStreamer first
    gst::init()?;
    // Check required_plugins plugins are installed

    let (subcommand, sub_m) = app_m.subcommand().unwrap();
    let app = App::new(&app_m, sub_m, subcommand)?;

    app.check_plugins()?;
    app.play()?;

    Ok(())
}
