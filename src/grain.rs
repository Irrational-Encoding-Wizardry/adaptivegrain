use failure::Error;
use num::Integer;
use rand::distributions::Uniform;
use rand::Rng;
use rand_xorshift::XorShiftRng;
use vapoursynth::api::API;
use vapoursynth::core::CoreRef;
use vapoursynth::frame::{FrameRef, FrameRefMut};
use vapoursynth::node::Node;
use vapoursynth::plugins::{Filter, FrameContext};
use vapoursynth::video_info::VideoInfo;

pub struct Grain<'core> {
    pub source: Node<'core>,
}

#[inline]
fn clip<T: Integer>(input: T, lower: T, upper: T) -> T {
    input.min(upper).max(lower)
}

impl<'core> Filter<'core> for Grain<'core> {
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
        let var = 20i16;
        // these have to be defined explicitly for lifetime reasons
        //let mut rng = thread_rng();
        let distribution = Uniform::new_inclusive(-var, var);
        let mut rng: XorShiftRng = rand::SeedableRng::seed_from_u64(653334623);
        let mut spread = rng.sample_iter(&distribution);

        let mut frame = FrameRefMut::copy_of(core, &frame);
        for plane in 0..frame.format().plane_count() {
            for row in 0..frame.height(plane) {
                for mut_pixel in frame.plane_row_mut::<u8>(plane, row) {
                    let pixel = *mut_pixel as i16 + spread.next().unwrap();
                    *mut_pixel = clip(pixel, 0, 255) as u8;
                }
            }
        }
        Ok(frame.into())
    }
}
