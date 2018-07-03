//mod vulkano_instance;
use vulkano_instance::PipelineImplementer;
use std::time::Duration;


use benchmarks::benchmark_base::BenchmarkCommon;
use benchmarks::benchmark_base::RenderBenchmark;
use benchmarks::benchmark_base::ToMilliseconds;

pub struct TriangleBenchmark
{
    benchmark_common : BenchmarkCommon,
    triangles_x : i32,
    triangles_y : i32,
}

impl TriangleBenchmark
{
    pub fn new(length : Duration, triangles_x : i32, triangles_y : i32) -> Self
    {
        TriangleBenchmark{benchmark_common: BenchmarkCommon::new(length), triangles_x, triangles_y}
    }
}

impl RenderBenchmark for TriangleBenchmark
{
    fn bench_frame(&mut self, renderer: &mut PipelineImplementer)
    {
        let width_per_triangle = 2f32 / self.triangles_x as f32;
        let height_per_triangle = 2f32 / self.triangles_y as f32;

        for x in 0..self.triangles_x
        {
            for y in 0..self.triangles_y
            {
                renderer.draw_triangle
                ([
                    [-1f32 + (0.5f32 + x as f32) * width_per_triangle - width_per_triangle / 2f32, -1f32 + (0.5f32 + y as f32) * height_per_triangle - height_per_triangle / 2f32], 
                    [-1f32 + (0.5f32 + x as f32) * width_per_triangle + width_per_triangle / 2f32, -1f32 + (0.5f32 + y as f32) * height_per_triangle - height_per_triangle / 2f32], 
                    [-1f32 + (0.5f32 + x as f32) * width_per_triangle, -1f32 + (0.5f32 + y as f32) * height_per_triangle + height_per_triangle / 2f32 + (self.benchmark_common.current_duration.to_milliseconds() * 0.003f32).sin()]
                ]);
            }
        }
    }

    fn print_result(&self)
    {
        println!("Triangle test complete: {} triangles over {} ms and {} frames took an average of {} per frame. A total of {} vertex buffers were killed in this process.\n", 
            self.triangles_x * self.triangles_y, 
            self.benchmark_common.target_duration.to_milliseconds(), 
            self.benchmark_common.frames_rendered,
            self.benchmark_common.target_duration.to_milliseconds()  / self.benchmark_common.frames_rendered as f32,
            self.benchmark_common.frames_rendered * self.triangles_x * self.triangles_y
        );
    }

    fn benchmark_common(&mut self) -> &mut BenchmarkCommon
    {
        &mut self.benchmark_common
    }
}

