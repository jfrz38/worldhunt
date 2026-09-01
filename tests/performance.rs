use std::time::{Duration, Instant};

use worldhunt::infrastructure::{random::RandomTargetSelector, tui, world_data::WorldData};

const SAMPLES: usize = 20;

#[test]
#[ignore = "engineering measurement; run explicitly in release mode"]
fn reports_release_decode_and_initial_render_percentiles() {
    for (width, height) in [(48, 20), (70, 30), (100, 30), (200, 60)] {
        let mut decode_samples = Vec::with_capacity(SAMPLES);
        let mut render_samples = Vec::with_capacity(SAMPLES);

        // Warm caches before collecting the reproducible sample set.
        let world_data = WorldData::decode_embedded().expect("embedded world data is valid");
        let mut selector = RandomTargetSelector::seeded(0);
        tui::render_initial_frame(&world_data, &world_data, &mut selector, width, height)
            .expect("warmup frame renders");

        for seed in 0..SAMPLES {
            let started = Instant::now();
            let world_data = WorldData::decode_embedded().expect("embedded world data is valid");
            decode_samples.push(started.elapsed());

            let mut selector = RandomTargetSelector::seeded(seed as u64);
            let started = Instant::now();
            tui::render_initial_frame(&world_data, &world_data, &mut selector, width, height)
                .expect("initial frame renders");
            render_samples.push(started.elapsed());
        }

        report("decode", width, height, &mut decode_samples);
        report("render", width, height, &mut render_samples);
    }
}

fn report(operation: &str, width: u16, height: u16, samples: &mut [Duration]) {
    samples.sort_unstable();
    let median = samples[samples.len() / 2];
    let p95 = samples[(samples.len() * 95).div_ceil(100) - 1];
    println!("{operation} {width}x{height}: median={median:?} p95={p95:?}");
}
