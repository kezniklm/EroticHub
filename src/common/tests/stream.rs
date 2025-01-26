use crate::business::facades::stream::GStreamerProxyTrait;
use crate::streamer::types::{CompoundStreamInfoTrait, StreamStorageTrait};
use gstreamer::Pipeline;
use std::sync::Arc;
use std::thread::JoinHandle;

pub(crate) struct StreamProxyMock {}

impl GStreamerProxyTrait for StreamProxyMock {
    fn create_streams(
        &self,
        stream_storage: Arc<dyn StreamStorageTrait>,
        compound_stream: Arc<dyn CompoundStreamInfoTrait>,
    ) -> anyhow::Result<Vec<JoinHandle<()>>> {
        let pipelines = vec![Arc::new(Pipeline::new())];
        stream_storage.push(compound_stream, pipelines);
        Ok(Vec::new())
    }

    fn stop_stream(&self, _pipeline: &Pipeline) -> anyhow::Result<()> {
        Ok(())
    }
}
