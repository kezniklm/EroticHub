use crate::business::models::stream::{CompoundStreamInfo, StreamResolution, StreamStorage};
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
        let pipeline = Arc::new(create_stream_pipeline(&compound_stream, &resolution)?);
        let stream_storage = stream_storage.clone();
        pipelines.push(pipeline.clone());

        let stream_id = compound_stream.stream_id.clone();
        handles.push(thread::spawn(move || {
            start_stream(stream_id.as_str(), pipeline.clone(), resolution);
            pipeline_listen(pipeline, stream_id.as_str(), stream_storage);
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

fn start_stream(stream_id: &str, pipeline: Arc<Pipeline>, resolution: StreamResolution) {
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
                        "Failed to start the stream with ID: {}: {}!",
                        stream_id,
                        resolution.as_str()
                    );
        }
    };
}

fn pipeline_listen(
    pipeline: Arc<Pipeline>,
    stream_id: &str,
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

pub fn create_stream_pipeline(
    parent_stream: &CompoundStreamInfo,
    resolution: &StreamResolution,
) -> Result<Pipeline> {
    let pipeline = Pipeline::new();

    add_elements_to_pipeline(&pipeline, parent_stream, resolution)?;
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
    stream: &CompoundStreamInfo,
    resolution: &StreamResolution,
) -> Result<()> {
    let (width, height) = resolution.get_resolution();
    let rtmp_url = stream.compose_stream_url(resolution.clone());
    
    // Video elements
    let file_src = build_element("filesrc", Some(&[("location", stream.video_path.as_str())]))?;
    let decode_bin = build_element("decodebin", Some(&[("name", "d")]))?;
    let queue = build_element("queue", None)?;
    let video_convert = build_element("videoconvert", None)?;
    let video_scale = build_element("videoscale", None)?;
    let video_h264 = build_element("capsfilter", Some(&[(
        "caps",
        format!("video/x-raw, width={width}, height={height}").as_str()
    )]))?;
    

    let x264_enc = build_element("x264enc", None)?;
    let flv_mux = build_element("flvmux", Some(&[
        ("name", "mux"),
        ("streamable", "true")
    ]))?;
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
mod test {
    use crate::business::models::stream::{CompoundStreamInfo, StreamResolution};
    use crate::streamer::gstream_controller::{create_stream_pipeline, init_gstreamer};
    use gstreamer::prelude::ElementExt;
    use gstreamer::{ClockTime, MessageView, State};
    use std::env;

    #[test]
    // #[allow(dead_code)]
    fn test01() -> anyhow::Result<()> {
        println!("{:?}", env::current_dir());
        init_gstreamer()?;
        let main_stream = CompoundStreamInfo::new(
            String::from("2"),
            String::from("video_resources/video3.mp4"),
            vec![StreamResolution::P360],
        );

        let pipeline = create_stream_pipeline(&main_stream, &StreamResolution::P360)?;
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
