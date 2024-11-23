use std::sync::Arc;
use actix_web::web::Data;
use crate::streamer::types::{CompoundStreamInfo, StreamInfo, StreamStorage};
use gstreamer::prelude::{
    ElementExt, ElementExtManual, GObjectExtManualGst, GstBinExtManual, PadExt,
};
use gstreamer::{ClockTime, Element, ElementFactory, MessageView, Pad, Pipeline, State};
use anyhow::Result;

pub async fn create_streams(stream_storage: Data<StreamStorage>, compound_stream: CompoundStreamInfo) -> Result<()> {
    let mut pipelines = Vec::new();
    for stream in &compound_stream.streams {
        let pipeline = Arc::new(create_stream(&compound_stream, stream)?);
        pipelines.push(pipeline.clone());

        actix_rt::spawn(async move {
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
                    println!("Error");
                }
                Some(bus) => {
                    for msg in bus.iter_timed(ClockTime::NONE) {
                        match msg.view() {
                            MessageView::Eos(_) => {
                                println!("stream ended");
                                stop_stream(&pipeline).expect("fail");
                                return;
                            }
                            MessageView::Error(err) => {
                                stop_stream(&pipeline).expect("Fail");
                                return;
                            }
                            _ => (),
                        }
                    }
                }
            }
        });

    }

    stream_storage.push(compound_stream, pipelines).await;


    Ok(())
}

pub fn init_gstream() -> Result<(), gstreamer::glib::Error> {
    gstreamer::init()
}

fn create_stream(
    parent_stream: &CompoundStreamInfo,
    resolution_stream: &StreamInfo,
) -> Result<Pipeline> {
    let pipeline = Pipeline::new();

    create_link_elements(&pipeline, parent_stream, resolution_stream)?;
    Ok(pipeline)
}

fn stop_stream(pipeline: &Pipeline) -> Result<()> {
    pipeline.set_state(State::Null)?;
    Ok(())
}

fn create_link_elements(
    pipeline: &Pipeline,
    parent_stream: &CompoundStreamInfo,
    resolution_stream: &StreamInfo,
) -> Result<()> {
    let file_src = ElementFactory::make("filesrc").build()?;
    file_src.set_property_from_str("location", &parent_stream.video_path);

    let decode_bin = ElementFactory::make("decodebin").build()?;
    decode_bin.set_property_from_str("name", "d");
    let queue = ElementFactory::make("queue").build()?;
    let video_convert = ElementFactory::make("videoconvert").build()?;

    let video_scale = ElementFactory::make("videoscale").build()?;

    let (width, height) = resolution_stream.resolution.get_resolution();
    let video_xh264 = ElementFactory::make("capsfilter").build()?;
    video_xh264.set_property_from_str(
        "caps",
        &format!(
            "video/x-raw, width={width}, height={height}",

        ),
    );

    let x264_enc = ElementFactory::make("x264enc").build()?;
    let flvmux = ElementFactory::make("flvmux").build()?;
    flvmux.set_property_from_str("name", "mux");
    flvmux.set_property_from_str("streamable", "true");
    let queue2 = ElementFactory::make("queue").build()?;

    let rtmp_sink = ElementFactory::make("rtmpsink").build()?;
    rtmp_sink.set_property_from_str(
        "location",
        &parent_stream.compose_stream_url(resolution_stream.resolution.clone()),
    );

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
        &video_scale,
        &video_xh264,
        &x264_enc,
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
        &video_scale,
        &video_xh264,
        &x264_enc,
        &flvmux,
    ])?;
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
    ])?;

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

mod test {
    use std::env;
    use gstreamer::prelude::ElementExt;
    use gstreamer::{ClockTime, MessageView, State};
    use crate::streamer::gstream_controller::{create_stream, create_streams, init_gstream};
    use crate::streamer::types::{CompoundStreamInfo, StreamInfo, StreamResolution};

    #[test]
    fn test01() -> anyhow::Result<()> {
        println!("{:?}", env::current_dir());
        init_gstream()?;
        let stream_360p = StreamInfo::new(StreamResolution::P360);
        let main_stream = CompoundStreamInfo::new(
            String::from("1"),
            String::from("video_resources/video3.mp4"),
            vec![stream_360p.clone()],
        );

        let pipeline = create_stream(&main_stream, &stream_360p)?;
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
                println!("Error");
            }
            Some(bus) => {
                for msg in bus.iter_timed(ClockTime::NONE) {
                    match msg.view() {
                        MessageView::Eos(_) => {
                            println!("stream ended");
                            break;
                        }
                        MessageView::Error(err) => {
                            println!("error, {}", err);
                            break;
                        }
                        _ => (),
                    }
                }
            }
        }
        Ok(())
    }
}
