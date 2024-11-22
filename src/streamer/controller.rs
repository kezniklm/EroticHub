use crate::streamer::types::StreamConfig;
use gstreamer::prelude::{
    ElementExt, ElementExtManual, GObjectExtManualGst, GstBinExtManual, PadExt,
};
use gstreamer::{ClockTime, Element, ElementFactory, MessageView, Pad, Pipeline, State};
use std::error::Error;

pub fn init_gstream() -> Result<(), gstreamer::glib::Error> {
    gstreamer::init()
}

pub fn create_stream(stream_config: StreamConfig) -> Result<(), Box<dyn Error>> {
    let pipeline = Pipeline::new();

    create_link_elements(&pipeline, stream_config)?;

    match pipeline.set_state(State::Playing) {
        Ok(_) => {
            println!("Stream started!");
        }
        Err(_) => {
            println!("Failed to start the stream!");
        }
    };
    match pipeline.bus() {
        None => {
            // TODO: Some better error handling, logging
            return Err(Box::from("Failed to play the stream"));
        }
        Some(bus) => for msg in bus.iter_timed(ClockTime::NONE) {
            match msg.view() {
                MessageView::Eos(_) => {
                    stop_stream(&pipeline)?;
                    return Ok(())
                },
                MessageView::Error(err) => {
                    stop_stream(&pipeline)?;
                    return Err(Box::from(err.error()))
                }
                _ => (),
            }
        },
    }
    Ok(())
}

fn stop_stream(pipeline: &Pipeline) -> Result<(), Box<dyn Error>> {
    pipeline.set_state(State::Null)?;
    Ok(())
}

fn create_link_elements(pipeline: &Pipeline, stream_config: StreamConfig) -> Result<(), Box<dyn Error>> {
    let file_src = ElementFactory::make("filesrc").build()?;
    file_src.set_property_from_str("location", &stream_config.video_path);

    let decode_bin = ElementFactory::make("decodebin").build()?;
    decode_bin.set_property_from_str("name", "d");
    let queue = ElementFactory::make("queue").build()?;
    let video_convert = ElementFactory::make("videoconvert").build()?;
    let x264_enc = ElementFactory::make("x264enc").build()?;
    x264_enc.set_property_from_str("bitrate", "1000");
    x264_enc.set_property_from_str("tune", "zerolatency");

    let video_xh264 = ElementFactory::make("capsfilter").build()?;
    video_xh264.set_property_from_str("caps", "video/x-h264");
    let h264_parse = ElementFactory::make("h264parse").build()?;
    let video_xh264_2 = ElementFactory::make("capsfilter").build()?;
    video_xh264_2.set_property_from_str("caps", "video/x-h264");
    let flvmux = ElementFactory::make("flvmux").build()?;
    flvmux.set_property_from_str("name", "mux");
    flvmux.set_property_from_str("streamable", "true");
    let queue2 = ElementFactory::make("queue").build()?;

    let rtmp_sink = ElementFactory::make("rtmpsink").build()?;
    rtmp_sink.set_property_from_str("location", &stream_config.compose_stream_url());

    let queue3 = ElementFactory::make("queue").build()?;
    let audio_convert = ElementFactory::make("audioconvert").build()?;
    let audio_resample = ElementFactory::make("audioresample").build()?;
    let audio_xraw = ElementFactory::make("capsfilter").build()?;
    audio_xraw.set_property_from_str("caps", "audio/x-raw,rate=48000");

    let voaacenc = ElementFactory::make("voaacenc").build()?;
    voaacenc.set_property_from_str("bitrate", "96000");

    let audio_mpeg = ElementFactory::make("capsfilter").build()?;
    audio_mpeg.set_property_from_str("caps", "audio/mpeg");

    let aac_parse = ElementFactory::make("aacparse").build()?;

    let audio_mpeg2 = ElementFactory::make("capsfilter").build()?;
    audio_mpeg2.set_property_from_str("caps", "audio/mpeg, mpegversion=4");

    let pipeline_elements = [
        &file_src,
        &decode_bin,
        &queue.clone(),
        &video_convert,
        &x264_enc,
        &video_xh264,
        &h264_parse,
        &video_xh264_2,
        &flvmux,
        &queue2,
        &rtmp_sink,
        &queue3.clone(),
        &audio_convert,
        &audio_resample,
        &audio_xraw,
        &voaacenc,
        &audio_mpeg,
        &aac_parse,
        &audio_mpeg2,
    ];
    pipeline.add_many(pipeline_elements)?;

    file_src.link(&decode_bin)?;
    Element::link_many([
        &queue,
        &video_convert,
        &x264_enc,
        &video_xh264,
        &h264_parse,
        &video_xh264_2,
        &flvmux,
    ])
    .unwrap();
    Element::link_many([
        &queue3.clone(),
        &audio_convert,
        &audio_resample,
        &audio_xraw,
        &voaacenc,
        &audio_mpeg,
        &aac_parse,
        &audio_mpeg2,
        &flvmux,
    ])
    .unwrap();

    flvmux.link(&rtmp_sink)?;
    decode_bin.connect_pad_added(move |_, src_pad: &Pad| {
        let video_sink_pad = &queue
            .static_pad("sink")
            .expect("Failed to get static sink pad from queue");

        if !src_pad.is_linked() {
            src_pad.link(video_sink_pad).err();
        }

        let audio_sink_pad = &queue3
            .static_pad("sink")
            .expect("failed to get static sink pad from audio queue");
        if !audio_sink_pad.is_linked() {
            src_pad.link(audio_sink_pad).err();
        }
    });

    Ok(())
}
