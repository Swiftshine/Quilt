mod quilt;
use quilt::QuiltApp;

fn main() -> Result<(), eframe::Error> {
    QuiltApp::run()
}
