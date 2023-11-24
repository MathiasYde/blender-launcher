use eframe::{App, Frame, NativeOptions, run_native};
use egui::{CentralPanel, Context};
use log;
use std::process::Command;

struct BlenderInstance {
	name: String,
	path: String,
}

struct Application {
	version: String,
	blender_instances: Vec<BlenderInstance>,
}

impl App for Application {
	fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
		CentralPanel::default().show(ctx, |ui| {
			ui.heading(format!("Blender Launcher v{}", self.version));
			ui.separator();

			// TODO(mathias): refactor this into a ScrollArea
			self.blender_instances.iter().for_each(|instance| {
				self.build_instance_ui(ui, instance);
				ui.separator();
			});
		});
	}
}

impl Application {
	fn new(_cc: &eframe::CreationContext<'_>) -> Self {
		Self {
			version: "0.1.0".to_string(),
			blender_instances: vec![
				BlenderInstance {
					name: "Blender 4.0.1".to_string(),
					path: "C:\\src\\.blender\\blender-4.0.1-windows-x64\\blender.exe".to_string(),
				},
				BlenderInstance {
					name: "Blender 4.1.0 (Alpha)".to_string(),
					path: "C:\\src\\.blender\\blender-4.1.0-alpha+main.23430f4db868-windows.amd64-release\\blender.exe".to_string(),
				},
			],
		}
	}

	fn build_instance_ui(&self, ui: &mut egui::Ui, instance: &BlenderInstance) {
		ui.horizontal(|ui| {
			if ui.button("Launch").clicked() {
				self.launch_instance(instance);
			}

			ui.vertical(|ui| {
				ui.label(&instance.name);
				ui.label(&instance.path);
			});
		});
	}

	fn launch_instance(&self, instance: &BlenderInstance) {
		log::info!("Launching Blender instance: {}", instance.name);
		Command::new(&instance.path)
			.spawn()
			.expect("Unable to launch Blender instance");
	}
}

fn main() {
	log::info!("Initializing Blender Launcher application");

	run_native(
		"Blender Launcher",
		NativeOptions::default(),
		Box::new(|cc| Box::new(Application::new(cc)))
	).expect("Unable to initialize application");
}
