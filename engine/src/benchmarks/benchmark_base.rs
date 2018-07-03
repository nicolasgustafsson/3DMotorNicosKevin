use vulkano_instance::PipelineImplementer;
use std::time::Instant;
use std::time::Duration;

pub trait ToMilliseconds
{
    fn to_milliseconds(& self) -> f32;
}

impl ToMilliseconds for Duration
{
    fn to_milliseconds(& self) -> f32
    {
        (self.subsec_nanos() as f64 * 0.000001f64 + self.as_secs() as f64 * 1000f64) as f32
    }
}

#[derive(Clone, Copy)]
pub enum BenchmarkStatus
{
    InProgress,
    Finished
}

pub trait RenderBenchmark
{
    fn begin_bench(&mut self);

    fn bench_frame_with_boilerplate(&mut self, renderer: &mut PipelineImplementer) -> BenchmarkStatus
    {
        let start = Instant::now();
        if renderer.begin_render().is_err()
        {
            return BenchmarkStatus::InProgress;
        }

        self.bench_frame(renderer);

        renderer.end_render();
        let end = Instant::now();

        let benchmark_common = self.benchmark_common();
        
        benchmark_common.current_duration += end - start;
        benchmark_common.frames_rendered += 1;

        if benchmark_common.current_duration > benchmark_common.target_duration
        {
            BenchmarkStatus::Finished
        }
        else
        {
            BenchmarkStatus::InProgress
        }
    }

    fn bench_frame(&mut self, &mut PipelineImplementer);

    fn print_result(&self);

    fn benchmark_common(&mut self) -> &mut BenchmarkCommon; 
}

pub struct BenchmarkCommon
{
    pub current_duration : Duration,
    pub target_duration : Duration,
    pub frames_rendered : i32
}

impl BenchmarkCommon
{
    pub fn new(length: Duration) -> Self
    {
        BenchmarkCommon{current_duration: Duration::new(0, 0), target_duration: length, frames_rendered: 0}
    }
}
