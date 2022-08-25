use anyhow::Result;
use env_logger::Builder;
use log::LevelFilter;
use printnanny_gst::pipeline::GstPipeline;
use printnanny_gst::video_socket::VideoSocketPipeline;
fn main() -> Result<()> {
    // include git sha in version, which requires passing a boxed string to clap's .version() builder
    // parse args
    let cmd = VideoSocketPipeline::clap_command();
    let app_m = cmd.get_matches();
    let app = VideoSocketPipeline::from(&app_m);
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
    app.run()?;
    Ok(())
}
