use super::PLUGIN_NAME;
use failure::Error;
use std::ptr;
use vapoursynth::core::CoreRef;
use vapoursynth::format::ColorFamily;
use vapoursynth::plugins::{Filter, FrameContext};
use vapoursynth::prelude::*;
use vapoursynth::video_info::{Property, VideoInfo};

pub struct Mask<'core> {
    pub source: Node<'core>,
    pub luma_scaling: f32,
}

lazy_static! {
    static ref FLOAT_RANGE: Vec<f32> = {
        [0f32; 256]
            .iter()
            .enumerate()
            .map(|(i, _f)| (i as f32) / 256.0)
            .collect()
    };
}

#[inline]
fn get_mask_value(x: f32, y: f32, luma_scaling: f32) -> f32 {
    f32::powf(
        1.0 - (x * (1.124 + x * (-9.466 + x * (36.624 + x * (-45.47 + x * 18.188))))),
        (y * y) * luma_scaling,
    )
}

macro_rules! from_property {
    ($prop: expr) => {
        match $prop {
            Property::Constant(p) => p,
            Property::Variable => unreachable!(),
        }
    };
}

macro_rules! int_filter {
    ($type:ty, $fname:ident) => {
        fn $fname(
            frame: &mut FrameRefMut,
            src_frame: FrameRef,
            depth: u8,
            average: f32,
            luma_scaling: f32,
        ) {
            let max = ((1 << depth) - 1) as f32;
            let lut: Vec<$type> = FLOAT_RANGE
                .iter()
                .map(|x| (get_mask_value(*x, average, luma_scaling) * max) as $type)
                .collect();
            for row in 0..frame.height(0) {
                for (pixel, src_pixel) in frame
                    .plane_row_mut::<$type>(0, row)
                    .iter_mut()
                    .zip(src_frame.plane_row::<$type>(0, row))
                {
                    let i = (src_pixel.clone() >> (depth - 8)) as usize;
                    unsafe {
                        ptr::write(pixel, lut[i].clone());
                    }
                }
            }
        }
    };
}

fn filter_for_float(frame: &mut FrameRefMut, src_frame: FrameRef, average: f32, luma_scaling: f32) {
    let lut: Vec<f32> = FLOAT_RANGE
        .iter()
        .map(|x| get_mask_value(*x, average, luma_scaling))
        .collect();
    for row in 0..frame.height(0) {
        for (pixel, src_pixel) in frame
            .plane_row_mut::<f32>(0, row)
            .iter_mut()
            .zip(src_frame.plane_row::<f32>(0, row).iter())
        {
            unsafe {
                ptr::write(pixel, lut[(src_pixel * 255.99f32) as usize]);
            }
        }
    }
}

impl<'core> Filter<'core> for Mask<'core> {
    fn video_info(&self, _api: API, _core: CoreRef<'core>) -> Vec<VideoInfo<'core>> {
        let info = self.source.info();
        let format = match info.format {
            Property::Variable => unreachable!(),
            Property::Constant(format) => format,
        };
        vec![VideoInfo {
            format: Property::Constant(
                _core
                    .register_format(
                        ColorFamily::Gray,
                        format.sample_type(),
                        format.bits_per_sample(),
                        0,
                        0,
                    )
                    .unwrap(),
            ),
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
        let new_format = from_property!(self.video_info(_api, core)[0].format);
        let mut frame = unsafe {
            FrameRefMut::new_uninitialized(
                core,
                None,
                new_format,
                from_property!(self.source.info().resolution),
            )
        };
        let src_frame = self.source.get_frame_filter(context, n).ok_or_else(|| {
            format_err!("Could not retrieve source frame. This shouldn’t happen.")
        })?;
        let average = match src_frame.props().get::<f64>("PlaneStatsAverage") {
            Ok(average) => average as f32,
            Err(_) => bail!(format!(
                "{}: you need to run std.PlaneStats on the clip before calling this function.",
                PLUGIN_NAME
            )),
        };

        match from_property!(self.source.info().format).sample_type() {
            SampleType::Integer => {
                let depth = from_property!(self.source.info().format).bits_per_sample();
                match depth {
                    0..=8 => {
                        int_filter!(u8, filter_8bit);
                        filter_8bit(&mut frame, src_frame, depth, average, self.luma_scaling)
                    }
                    9..=16 => {
                        int_filter!(u16, filter_16bit);
                        filter_16bit(&mut frame, src_frame, depth, average, self.luma_scaling)
                    }
                    17..=32 => {
                        int_filter!(u32, filter_32bit);
                        filter_32bit(&mut frame, src_frame, depth, average, self.luma_scaling)
                    }
                    _ => bail!(format!(
                        "{}: input depth {} not supported",
                        PLUGIN_NAME, depth
                    )),
                }
            }
            SampleType::Float => {
                if let Err(e) = verify_input_range(&src_frame.props()) {
                    bail!(e);
                }
                filter_for_float(&mut frame, src_frame, average, self.luma_scaling);
            }
        }
        Ok(frame.into())
    }
}

fn verify_input_range<'a>(props: &vapoursynth::map::MapRef<'a, 'a>) -> Result<(), String> {
    let max = props.get::<f64>("PlaneStatsMax").unwrap_or(1.0);
    let min = props.get::<f64>("PlaneStatsMin").unwrap_or(0.0);
    if min < 0.0 || max > 1.0 {
        return Err(format!(
                "{}: found invalid input. Some pixels are outside of the valid range.
                You probably used a filter that operates on limited range without clipping properly, e.g. edgefixer, before converting to float.
                This can be fixed by clipping all pixels to (0, 1) with Expr or converting to an integer format.", PLUGIN_NAME
        ));
    }
    Ok(())
}
