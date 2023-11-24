use eframe::{App, Frame, NativeOptions, run_native};
use egui::{CentralPanel, Context};

struct Application;

impl App for Application {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.label("Hello World!");
        });
    }
}

impl Default for Application {
			fn default() -> Self {
				Self {}
		}
}

impl Application {
	fn new(cc: &eframe::CreationContext<'_>) -> Self {
		Self
	}
}

fn main() {
    run_native(
			"Blender Launcher",
			NativeOptions::default(),
			Box::new(|cc| Box::new(Application::new(cc)))
		).expect("Unable to initialize application");
}
