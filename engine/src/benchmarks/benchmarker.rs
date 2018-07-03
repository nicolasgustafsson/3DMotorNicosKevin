use benchmarks::benchmark_base::RenderBenchmark;
use benchmarks::benchmark_base::BenchmarkStatus;
use vulkano_instance::PipelineImplementer;

pub struct Benchmarker
{
    tests : Vec<Box<RenderBenchmark>>,
    test_index : usize,
}

impl Benchmarker
{
    pub fn new(tests : Vec<Box<RenderBenchmark>>) -> Self
    {
        Self{
            tests,
            test_index : 0
            }
    }

    pub fn increment_test(&mut self)
    {
        self.tests[self.test_index].print_result();

        self.test_index += 1;

        self.test_index %= self.tests.len();

        self.tests[self.test_index].begin_bench();
    }

    //runs  tests.
    pub fn tick_tests(&mut self, renderer : &mut PipelineImplementer)
    {
        let status = self.tests[self.test_index].bench_frame_with_boilerplate(renderer);

        match status
        {
            BenchmarkStatus::Finished => self.increment_test(),
            _ => {}
        }
    }
}
