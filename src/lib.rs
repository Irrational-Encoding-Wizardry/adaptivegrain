#[macro_use]
extern crate failure;
#[macro_use]
extern crate vapoursynth;

pub mod mask;

use self::mask::Mask;
use failure::Error;
use vapoursynth::api::API;
use vapoursynth::core::CoreRef;
use vapoursynth::format::SampleType;
use vapoursynth::map::Map;
use vapoursynth::node::Node;
use vapoursynth::plugins::{Filter, FilterArgument, Metadata};
use vapoursynth::video_info::Property;

pub const PLUGIN_NAME: &str = "adaptivegrain";
pub const PLUGIN_IDENTIFIER: &str = "moe.kageru.adaptivegrain";

make_filter_function! {
    MaskFunction, "Mask"
    fn create_mask<'core>(
        _api: API,
        _core: CoreRef<'core>,
        clip: Node<'core>,
        luma_scaling: Option<f64>
    ) -> Result<Option<Box<dyn Filter<'core> + 'core>>, Error> {
        let luma_scaling = luma_scaling.unwrap_or(10.0) as f32;
        if let Property::Constant(format) = clip.info().format {
            if !(format.sample_type() == SampleType::Float && format.bits_per_sample() != 32) {
                return Ok(Some(Box::new(Mask {
                    source: clip,
                    luma_scaling
                })));
            } else {
                bail!("Half precision float input is not supported");
            }
        }
        bail!("Variable format input is not supported")
    }
}

export_vapoursynth_plugin! {
    Metadata {
        identifier: PLUGIN_IDENTIFIER,
        namespace: "adg",
        name: "Adaptive grain",
        read_only: false,
    },
    [ MaskFunction::new() ]
}
