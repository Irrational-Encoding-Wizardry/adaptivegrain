#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate vapoursynth;

mod grain;
mod mask;

use self::grain::Grain;
use self::mask::Mask;
use failure::Error;
use vapoursynth::api::API;
use vapoursynth::core::CoreRef;
use vapoursynth::format::SampleType;
use vapoursynth::frame::{FrameRef, FrameRefMut};
use vapoursynth::map::Map;
use vapoursynth::node::Node;
use vapoursynth::plugins::{Filter, FilterArgument, FrameContext, Metadata};
use vapoursynth::video_info::{Property, VideoInfo};

pub const PLUGIN_NAME: &str = "adaptivegrain";
pub const PLUGIN_IDENTIFIER: &str = "moe.kageru.adaptivegrain";

struct AdaptiveGrain<'core> {
    source: Node<'core>,
}

impl<'core> Filter<'core> for AdaptiveGrain<'core> {
    fn video_info(&self, _api: API, _core: CoreRef<'core>) -> Vec<VideoInfo<'core>> {
        vec![self.source.info()]
    }

    fn get_frame_initial(
        &self,
        _api: API,
        _core: CoreRef<'core>,
        context: FrameContext,
        n: usize,
    ) -> Result<Option<FrameRef<'core>>, Error> {
        self.source.request_frame_filter(context, n);
        Ok(None)
    }

    fn get_frame(
        &self,
        _api: API,
        core: CoreRef<'core>,
        context: FrameContext,
        n: usize,
    ) -> Result<FrameRef<'core>, Error> {
        let frame = self.source.get_frame_filter(context, n).ok_or_else(|| {
            format_err!("Could not retrieve source frame. This shouldn’t happen.")
        })?;
        let average = match frame.props().get::<f64>("PlaneStatsAverage") {
            Ok(average) => (average * 256.0) as u8,
            Err(_) => panic!(
                PLUGIN_NAME.to_owned()
                    + ": You need to run std.PlaneStats on the clip before calling this function."
            ),
        };

        let mut frame = FrameRefMut::copy_of(core, &frame);
        for plane in 0..frame.format().plane_count() {
            for row in 0..frame.height(plane) {
                for pixel in frame.plane_row_mut::<u8>(plane, row) {
                    *pixel = average;
                }
            }
        }
        Ok(frame.into())
    }
}

make_filter_function! {
    AdaptiveGrainFunction, "AdaptiveGrain"

    fn create_adaptivegrain<'core>(
        _api: API,
        _core: CoreRef<'core>,
        clip: Node<'core>,
    ) -> Result<Option<Box<dyn Filter<'core> + 'core>>, Error> {
        Ok(Some(Box::new(AdaptiveGrain { source: clip })))
    }
}

make_filter_function! {
    GrainFunction, "Grain"

    fn create_grain<'core>(
        _api: API,
        _core: CoreRef<'core>,
        clip: Node<'core>,
    ) -> Result<Option<Box<dyn Filter<'core> + 'core>>, Error> {
        Ok(Some(Box::new(Grain { source: clip })))
    }
}

make_filter_function! {
    MaskFunction, "Mask"

    fn create_mask<'core>(
        _api: API,
        _core: CoreRef<'core>,
        clip: Node<'core>,
        luma_scaling: Option<f64>
    ) -> Result<Option<Box<dyn Filter<'core> + 'core>>, Error> {
        let luma_scaling = match luma_scaling {
            Some(i) => i as f32,
            None => 10.0
        };
        if let Property::Constant(format) = clip.info().format {
            if format.bits_per_sample() == 32 && format.sample_type() == SampleType::Float {
                return Ok(Some(Box::new(Mask {
                    source: clip,
                    luma_scaling
                })));
            }
        }
        bail!("Currently, only constant 32bit float input is supported" )
    }
}

export_vapoursynth_plugin! {
    Metadata {
        identifier: PLUGIN_IDENTIFIER,
        namespace: "adg",
        name: "Adaptive grain",
        read_only: false,
    },
    [
        //AdaptiveGrainFunction::new(),
        //GrainFunction::new(),
        MaskFunction::new()
    ]
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
