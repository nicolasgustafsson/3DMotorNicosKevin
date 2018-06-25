//mod vulkano_instance;
use vulkano_instance::PipelineImplementer;
use std::time::Instant;
use std::time::Duration;

trait ToMilliseconds
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

/*
* These are rendering benchmarks we can use to measure performance. 
* When adding a benchmark, please write what the benchmark's purpose is.
*/
#[derive(Debug, Clone, Copy)]
pub enum TestProgress
{
    NotStarted,
    RenderOverhead,
    Triangles,
    Done
}

pub struct TestHarness
{
    test_progress: TestProgress,
    current_ticks : i32,
    ticks_per_test : i32,
    test_time_elapsed : Duration
}

impl TestHarness
{
    pub fn new(ticks_per_test : i32) -> TestHarness
    {
        TestHarness{
            test_progress : TestProgress::NotStarted, 
            current_ticks : 0,
            ticks_per_test,
            test_time_elapsed : Duration::new(0, 0),
            }
    }

    pub fn increment_test(&mut self)
    {
        let delta_ms = self.test_time_elapsed.to_milliseconds();
        
        println!("Test {:?} completed, took {} ms over {} ticks.\nAverage delta per tick was {} ms.\n", 
        self.test_progress, delta_ms, self.current_ticks, delta_ms / self.current_ticks as f32);

        match self.test_progress
        {
            TestProgress::NotStarted => 
            {
                self.test_progress = TestProgress::RenderOverhead;
            },
            TestProgress::RenderOverhead => 
            {
                self.test_progress = TestProgress::Triangles
            },
            TestProgress::Triangles => 
            {
                self.test_progress = TestProgress::Done
            },
            TestProgress::Done => 
            {
                self.test_progress = TestProgress::NotStarted
            }
        }

        self.current_ticks = 0;
        self.test_time_elapsed = Duration::new(0, 0);
    }

    //runs  tests.
    pub fn tick_tests(&mut self, renderer : &mut PipelineImplementer)
    {
        match self.test_progress
        {
            TestProgress::NotStarted => {},
            TestProgress::RenderOverhead => 
            {
                self.test_time_elapsed += self.no_render(renderer);
            },
            TestProgress::Triangles =>
            {
                self.test_time_elapsed += self.triangles(renderer, 10, 10);
            }
            TestProgress::Done => {return;}
        };

        if self.current_ticks >= self.ticks_per_test
        {
            self.increment_test();
        }

        self.current_ticks += 1;
    }

    //measure the performance of drawing the simplest of geometry, as well as see how it scales when there are more of them
    pub fn triangles(&self, renderer : &mut PipelineImplementer, triangle_count_x : i32, triangle_count_y : i32) -> Duration
    {
        let start = Instant::now();
        if renderer.begin_render().is_err()
        {
            return Instant::now() - start;
        }

        let width_per_triangle = 2f32 / triangle_count_x as f32;
        let height_per_triangle = 2f32 / triangle_count_y as f32;

        for x in 0..=triangle_count_x
        {
            for y in 0..=triangle_count_y
            {
                renderer.draw_triangle
                ([
                    [-1f32 + x as f32 * width_per_triangle - width_per_triangle / 2f32, -1f32 + y as f32 * height_per_triangle - height_per_triangle / 2f32], 
                    [-1f32 + x as f32 * width_per_triangle + width_per_triangle / 2f32, -1f32 + y as f32 * height_per_triangle - height_per_triangle / 2f32], 
                    [-1f32 + x as f32 * width_per_triangle, -1f32 + y as f32 * height_per_triangle + height_per_triangle / 2f32 + (self.test_time_elapsed.to_milliseconds() * 0.003f32).sin()]
                ]);
            }
        }

        renderer.end_render();
        let end = Instant::now();
        
        end - start
    }

    //measure the performance overhead of starting and ending a frame.
    pub fn no_render(&self, renderer : &mut PipelineImplementer) -> Duration
    {
        let start = Instant::now();
        if renderer.begin_render().is_err()
        {
            return Instant::now() - start;
        }

        renderer.end_render();
        let end = Instant::now();
        end - start
    }
}
