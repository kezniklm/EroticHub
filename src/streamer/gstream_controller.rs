use crate::streamer::types::{CompoundStreamInfo, StreamResolution, StreamStorage};
use actix_web::web::Data;
use anyhow::Result;
use gstreamer::prelude::{
    ElementExt, ElementExtManual, GObjectExtManualGst, GstBinExtManual, PadExt,
};
use gstreamer::{ClockTime, Element, ElementFactory, MessageView, Pad, Pipeline, State};
use log::{debug, error, info};
use std::sync::Arc;
use std::thread;

pub fn init_gstreamer() -> std::result::Result<(), gstreamer::glib::Error> {
    gstreamer::init()
}

pub async fn create_streams(
    stream_storage: Data<StreamStorage>,
    compound_stream: CompoundStreamInfo,
) -> Result<()> {
    let mut pipelines = Vec::new();
    let mut handles = Vec::new();
    for resolution in compound_stream.clone().streams {
        let pipeline = Arc::new(create_stream(&compound_stream, &resolution)?);
        let stream_storage = stream_storage.clone();
        pipelines.push(pipeline.clone());

        let stream_id = compound_stream.stream_id.clone();
        handles.push(thread::spawn(move || {
            match pipeline.set_state(State::Playing) {
                Ok(_) => {
                    info!(
                        "Stream with ID: {}: {} started",
                        stream_id,
                        resolution.as_str()
                    );
                }
                Err(_) => {
                    error!(
                        "Failed to start the with ID: {}: {}!",
                        stream_id,
                        resolution.as_str()
                    );
                }
            };

            pipeline_listen(pipeline, stream_id, stream_storage);
        }));
    }

    actix_rt::spawn(async move {
        for handle in handles {
            handle.join().expect("Cannot join the thread");
        }
    });

    stream_storage.push(compound_stream, pipelines);

    Ok(())
}

fn pipeline_listen(
    pipeline: Arc<Pipeline>,
    stream_id: String,
    stream_storage: Data<StreamStorage>,
) {
    if pipeline.bus().is_none() {
        error!("Error while initializing bus for stream: {}", stream_id);
        return;
    }

    for msg in pipeline.bus().unwrap().iter_timed(ClockTime::NONE) {
        match msg.view() {
            MessageView::Eos(_) => {
                info!("Stream with ID: {} ended", stream_id);
                stop_stream(&pipeline);
                stream_storage.remove(stream_id);
                debug!("Stream storage size: {}", stream_storage.size());
                break;
            }
            MessageView::Error(err) => {
                error!(
                    "Error occurred during stream with ID: {}, {}",
                    stream_id, err
                );
                stop_stream(&pipeline);
                stream_storage.remove(stream_id);
                debug!("Stream storage size: {}", stream_storage.size());
                break;
            }
            _ => (),
        }
    }
}

pub fn create_stream(
    parent_stream: &CompoundStreamInfo,
    resolution: &StreamResolution,
) -> Result<Pipeline> {
    let pipeline = Pipeline::new();

    create_link_elements(&pipeline, parent_stream, resolution)?;
    Ok(pipeline)
}

fn stop_stream(pipeline: &Pipeline) {
    match pipeline.set_state(State::Null) {
        Ok(_) => {
            debug!("Stream successfully ended");
        }
        Err(err) => {
            error!("Failed to end stream: {}", err);
        }
    }
}

fn create_link_elements(
    pipeline: &Pipeline,
    parent_stream: &CompoundStreamInfo,
    resolution: &StreamResolution,
) -> Result<()> {
    let file_src = ElementFactory::make("filesrc").build()?;
    file_src.set_property_from_str("location", &parent_stream.video_path);

    let decode_bin = ElementFactory::make("decodebin").build()?;
    decode_bin.set_property_from_str("name", "d");
    let queue = ElementFactory::make("queue").build()?;
    let video_convert = ElementFactory::make("videoconvert").build()?;

    let video_scale = ElementFactory::make("videoscale").build()?;

    let (width, height) = resolution.get_resolution();
    let video_xh264 = ElementFactory::make("capsfilter").build()?;
    video_xh264.set_property_from_str(
        "caps",
        &format!("video/x-raw, width={width}, height={height}",),
    );

    let x264_enc = ElementFactory::make("x264enc").build()?;
    let flvmux = ElementFactory::make("flvmux").build()?;
    flvmux.set_property_from_str("name", "mux");
    flvmux.set_property_from_str("streamable", "true");
    let queue2 = ElementFactory::make("queue").build()?;

    let rtmp_sink = ElementFactory::make("rtmpsink").build()?;
    rtmp_sink.set_property_from_str(
        "location",
        &parent_stream.compose_stream_url(resolution.clone()),
    );

    let queue3 = ElementFactory::make("queue").build()?;
    let audio_convert = ElementFactory::make("audioconvert").build()?;
    let audio_resample = ElementFactory::make("audioresample").build()?;
    let audio_xraw = ElementFactory::make("capsfilter").build()?;
    audio_xraw.set_property_from_str("caps", "audio/x-raw");

    let avenc_aac = ElementFactory::make("avenc_aac").build()?;
    avenc_aac.set_property_from_str("bitrate", "128000");

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
        &avenc_aac,
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
        &avenc_aac,
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

// Use this test only for offline tests of streaming controller
// Remove it before the project is done
mod test {
    use crate::streamer::gstream_controller::{create_stream, init_gstreamer};
    use crate::streamer::types::{CompoundStreamInfo, StreamResolution};
    use gstreamer::prelude::ElementExt;
    use gstreamer::{ClockTime, MessageView, State};
    use std::env;

    // #[test]
    #[allow(dead_code)]
    fn test01() -> anyhow::Result<()> {
        println!("{:?}", env::current_dir());
        init_gstreamer()?;
        let main_stream = CompoundStreamInfo::new(
            String::from("2"),
            String::from("video_resources/video3.mp4"),
            vec![StreamResolution::P360],
        );

        let pipeline = create_stream(&main_stream, &StreamResolution::P360)?;
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
