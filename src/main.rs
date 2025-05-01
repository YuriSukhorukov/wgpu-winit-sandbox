mod examples;
mod render;
use render::Vertex;
fn main() {
    // examples::triangle::run();
    // examples::buffers_and_indexes::run();
    // examples::textures_and_bind_groups::run();
    // examples::perspective_camera::run();
    examples::compute::run();
}