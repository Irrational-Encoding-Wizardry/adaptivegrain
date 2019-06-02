use failure::Error;
use super::PLUGIN_NAME;
use vapoursynth::api::API;
use vapoursynth::core::CoreRef;
use vapoursynth::frame::{FrameRef, FrameRefMut};
use vapoursynth::node::Node;
use vapoursynth::plugins::{Filter, FrameContext};
use vapoursynth::video_info::{VideoInfo, Property};
use vapoursynth::format::ColorFamily;
use std::fmt::Debug;


pub struct Mask<'core> {
    pub source: Node<'core>,
    pub luma_scaling: f32,
}

lazy_static! {
    static ref FLOAT_RANGE: Vec<f32> = {
        [0f32; 1000].iter()
            .enumerate()
            .map(|(i, _f)| (i as f32) * 0.001)
            .collect::<Vec<_>>()
    };
}

#[inline]
fn get_mask_value(x: f32, y: f32, luma_scaling: f32) -> f32 {
    f32::powf(1.0 - (x * (1.124 + x * (-9.466 + x * (36.624 + x * (-45.47 + x * 18.188))))), (y * y) * luma_scaling)
}

#[inline]
fn from_property<T: Debug + Clone + Copy + Eq + PartialEq>(prop: Property<T>) -> T {
    match prop {
        Property::Constant(p) => p,
        Property::Variable => panic!()
    }
}

impl<'core> Filter<'core> for Mask<'core> {
    fn video_info(&self, _api: API, _core: CoreRef<'core>) -> Vec<VideoInfo<'core>> {
        let info = self.source.info();
        let format = match info.format {
            Property::Variable => panic!("adaptivegrain: only constant format input supported"),
            Property::Constant(format) => format
        };
        vec![VideoInfo {
            format: Property::Constant(_core.register_format(
                ColorFamily::Gray,
                format.sample_type(),
                format.bits_per_sample(),
                0,
                0,
            ).unwrap()),
            flags: info.flags,
            framerate: info.framerate,
            num_frames: info.num_frames,
            resolution: info.resolution,
        }]
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
        let new_format = from_property(self.video_info(_api, core)[0].format);
        let mut frame = unsafe {
            FrameRefMut::new_uninitialized(
                core, None, new_format, from_property(self.source.info().resolution))
        };
        let src_frame = self.source
            .get_frame_filter(context, n)
            .ok_or_else(|| format_err!("Could not retrieve source frame. This shouldn’t happen."))?;
        let average = match src_frame.props().get::<f64>("PlaneStatsAverage") {
            Ok(average) => average as f32,
            Err(_) => panic!(format!("{}: you need to run std.PlaneStats on the clip before calling this function.", PLUGIN_NAME))
        };

        let lut: Vec<f32> = FLOAT_RANGE.iter().map(|x| get_mask_value(*x, average, self.luma_scaling)).collect();

        for row in 0..frame.height(0) {
            for (pixel, src_pixel) in frame.plane_row_mut::<f32>(0, row).iter_mut()
                .zip(src_frame.plane_row::<f32>(0, row).iter()) {
                *pixel = lut[(src_pixel * 1000f32) as usize];
            }
        }
        Ok(frame.into())
    }
}
