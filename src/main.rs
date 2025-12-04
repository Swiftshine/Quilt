#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod quilt;
use quilt::QuiltApp;

fn main() -> Result<(), eframe::Error> {
    QuiltApp::run()
}
