use std::env;
use std::fs::File;
use std::io::Read;
use eframe::{Frame, NativeOptions, run_native};
use egui::{CentralPanel, Context};
use log;
use std::process::Command;

extern crate yaml_rust;
use yaml_rust::{YamlLoader};

struct BlenderInstance {
	name: String,
	path: String,
}

struct Application {
	version: String,
	blender_instances: Vec<BlenderInstance>,
}

impl eframe::App for Application {
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

impl Default for Application {
	fn default() -> Self {
		Self {
			version: env!("CARGO_PKG_VERSION").to_string(),
			blender_instances: Vec::new(),
		}
	}
}

impl Application {
	fn new(_cc: &eframe::CreationContext<'_>) -> Self {
		let mut app = Application::default();

		let config_filepath = env::var("BLENDER_LAUNCHER_CONFIG_FILEPATH");
		Application::load_configuration(config_filepath.unwrap(), &mut app);

		return app;
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

	fn load_configuration(config_filepath: String, app: &mut Application) {
		log::info!("Loading configuration from: {}", config_filepath);

		// read config file
		let mut file = File::open(config_filepath).expect("Unable to open config file");
		let mut yaml = String::new();
		file.read_to_string(&mut yaml).expect("Unable to read config file");

		let docs = YamlLoader::load_from_str(&*yaml).unwrap();
		let _settings_doc = &docs[0]["settings"];
		let instances_doc = &docs[0]["instances"];

		instances_doc.as_vec().unwrap().iter().for_each(|instance_doc| {
			let name = instance_doc["name"].as_str().unwrap();
			let path = instance_doc["path"].as_str().unwrap();

			app.blender_instances.push(BlenderInstance {
				name: name.to_string(),
				path: path.to_string(),
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
