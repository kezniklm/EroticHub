use crate::streamer::types::{CompoundStreamInfoTrait, StreamResolution, StreamStorageTrait};
use anyhow::Result;
use gstreamer::prelude::{
    ElementExt, ElementExtManual, GObjectExtManualGst, GstBinExtManual, PadExt,
};
use gstreamer::{ClockTime, Element, ElementFactory, MessageView, Pad, Pipeline, State};
use log::{debug, error, info};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;

pub fn init_gstreamer() -> std::result::Result<(), gstreamer::glib::Error> {
    gstreamer::init()
}

pub fn create_streams(
    stream_storage: Arc<dyn StreamStorageTrait>,
    compound_stream: Arc<dyn CompoundStreamInfoTrait>,
) -> Result<Vec<JoinHandle<()>>> {
    let mut pipelines = Vec::new();
    let mut handles = Vec::new();
    for resolution in compound_stream.get_resolutions().clone() {
        let pipeline = Arc::new(create_stream_pipeline(
            compound_stream.clone(),
            &resolution,
        )?);
        let stream_storage = stream_storage.clone();
        pipelines.push(pipeline.clone());

        let stream_id = compound_stream.clone().get_stream_id().clone();
        start_stream(pipeline.clone())?;

        info!(
            "Stream with ID: {}: {} started",
            stream_id,
            resolution.as_str()
        );
        handles.push(thread::spawn(move || {
            match pipeline_listen(pipeline, stream_id.as_str(), stream_storage) {
                Ok(_) => {}
                Err(err) => error!("Error occurred during live stream, {:?}", err),
            }
        }));
    }

    stream_storage.push(compound_stream.clone(), pipelines);

    Ok(handles)
}

fn start_stream(pipeline: Arc<Pipeline>) -> Result<()> {
    pipeline.set_state(State::Playing)?;
    Ok(())
}

fn pipeline_listen(
    pipeline: Arc<Pipeline>,
    stream_id: &str,
    stream_storage: Arc<dyn StreamStorageTrait>,
) -> Result<()> {
    if pipeline.bus().is_none() {
        return Err(anyhow::Error::msg(
            "Error while initializing bus for stream: {}",
        ));
    }

    for msg in pipeline.bus().unwrap().iter_timed(ClockTime::NONE) {
        match msg.view() {
            MessageView::Eos(_) => {
                info!("Stream with ID: {} ended", stream_id);
                stop_stream(&pipeline)?;
                stream_storage.remove(stream_id);
                debug!("Stream storage size: {}", stream_storage.size());
                break;
            }
            MessageView::Error(err) => {
                error!(
                    "Error occurred during stream with ID: {}, {}",
                    stream_id, err
                );
                stop_stream(&pipeline)?;
                stream_storage.remove(stream_id);
                debug!("Stream storage size: {}", stream_storage.size());
                break;
            }
            _ => (),
        }
    }

    Ok(())
}

fn create_stream_pipeline(
    parent_stream: Arc<dyn CompoundStreamInfoTrait>,
    resolution: &StreamResolution,
) -> Result<Pipeline> {
    let pipeline = Pipeline::new();

    add_elements_to_pipeline(&pipeline, parent_stream, resolution)?;
    Ok(pipeline)
}

pub fn stop_stream(pipeline: &Pipeline) -> Result<()> {
    pipeline.set_state(State::Null)?;

    Ok(())
}

fn build_element(name: &str, props: Option<&[(&str, &str)]>) -> Result<Element> {
    let element = ElementFactory::make(name).build()?;
    if let Some(props) = props {
        for &(key, val) in props {
            element.set_property_from_str(key, val);
        }
    }

    Ok(element)
}

fn add_elements_to_pipeline(
    pipeline: &Pipeline,
    stream: Arc<dyn CompoundStreamInfoTrait>,
    resolution: &StreamResolution,
) -> Result<()> {
    let (width, height, bitrate) = resolution.get_resolution();
    let rtmp_url = stream.compose_rtmp_url(resolution.clone());

    // Video elements
    let file_src = build_element(
        "filesrc",
        Some(&[("location", stream.get_video_path().as_str())]),
    )?;
    let decode_bin = build_element("decodebin", Some(&[("name", "d")]))?;
    let queue = build_element("queue", None)?;
    let video_convert = build_element("videoconvert", None)?;
    let video_scale = build_element("videoscale", None)?;
    let video_h264 = build_element(
        "capsfilter",
        Some(&[(
            "caps",
            format!("video/x-raw, width={width}, height={height}").as_str(),
        )]),
    )?;

    let x264_enc = build_element("x264enc", Some(&[("bitrate", &bitrate.to_string())]))?;
    let flv_mux = build_element("flvmux", Some(&[("name", "mux"), ("streamable", "true")]))?;
    let queue2 = build_element("queue", None)?;
    let rtmp_sink = build_element("rtmpsink", Some(&[("location", rtmp_url.as_str())]))?;

    // Audio elements
    let queue3 = build_element("queue", None)?;
    let audio_convert = build_element("audioconvert", None)?;
    let audio_resample = build_element("audioresample", None)?;
    let audio_xraw = build_element("capsfilter", Some(&[("caps", "audio/x-raw")]))?;
    let avenc_aac = build_element("avenc_aac", Some(&[("bitrate", "128000")]))?;
    let audio_mpeg = build_element("capsfilter", Some(&[("caps", "audio/mpeg")]))?;
    let aac_parse = build_element("aacparse", None)?;
    let audio_mpeg4 = build_element("capsfilter", Some(&[("caps", "audio/mpeg, mpegversion=4")]))?;

    let pipeline_elements = [
        &file_src,
        &decode_bin,
        &queue.clone(),
        &video_convert,
        &video_scale,
        &video_h264,
        &x264_enc,
        &flv_mux,
        &queue2,
        &rtmp_sink,
        &queue3.clone(),
        &audio_convert,
        &audio_resample,
        &audio_xraw,
        &avenc_aac,
        &audio_mpeg,
        &aac_parse,
        &audio_mpeg4,
    ];
    pipeline.add_many(pipeline_elements)?;

    file_src.link(&decode_bin)?;
    Element::link_many([
        &queue,
        &video_convert,
        &video_scale,
        &video_h264,
        &x264_enc,
        &flv_mux,
    ])?;
    Element::link_many([
        &queue3.clone(),
        &audio_convert,
        &audio_resample,
        &audio_xraw,
        &avenc_aac,
        &audio_mpeg,
        &aac_parse,
        &audio_mpeg4,
        &flv_mux,
    ])?;

    flv_mux.link(&rtmp_sink)?;
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
#[allow(unused_imports)]
mod test {
    use crate::business::models::stream::CompoundStreamInfo;
    use crate::streamer::gstreamer_controller::{create_stream_pipeline, init_gstreamer};
    use crate::streamer::types::StreamResolution;
    use gstreamer::prelude::ElementExt;
    use gstreamer::{ClockTime, MessageView, State};
    use std::env;
    use std::sync::Arc;

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

        let pipeline = create_stream_pipeline(Arc::new(main_stream), &StreamResolution::P360)?;
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
