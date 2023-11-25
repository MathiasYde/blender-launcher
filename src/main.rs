use std::env;
use std::fs::File;
use std::io::{Read, Write};
use eframe::{Frame, NativeOptions, run_native};
use egui::{CentralPanel, Context, ScrollArea};
use log;
use std::process::Command;

extern crate yaml_rust;
use yaml_rust::{Yaml, YamlLoader};

struct BlenderInstance {
	name: String,
	path: String,
}

struct Application {
	version: String,
	instances: Vec<BlenderInstance>,
}

impl eframe::App for Application {
	fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
		CentralPanel::default().show(ctx, |ui| {
			ui.heading(format!("Blender Launcher v{}", self.version));
			ui.separator();

			ScrollArea::vertical().show(ui, |ui| {
				self.instances.iter().for_each(|instance| {
					self.build_instance_ui(ui, instance);
					ui.separator();
				});
			})
		});
	}
}

impl Default for Application {
	fn default() -> Self {
		Self {
			version: env!("CARGO_PKG_VERSION").to_string(),
			instances: Vec::new(),
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

			app.instances.push(BlenderInstance {
				name: name.to_string(),
				path: path.to_string(),
			});
		});
	}

	fn save_configuration(config_filepath: String, app: &Application) {
		log::info!("Saving configuration to: {}", config_filepath);

		let mut yaml_text = String::new(); {
			let mut emitter = yaml_rust::YamlEmitter::new(&mut yaml_text);
			let mut doc = yaml_rust::yaml::Hash::new();

			let mut settings = yaml_rust::yaml::Hash::new();
			settings.insert(Yaml::String("launch_on_system_startup".to_string()), Yaml::Boolean(false));

			let mut instances = yaml_rust::yaml::Array::new();
			app.instances.iter().for_each(|instance| {
				let mut instance_doc = yaml_rust::yaml::Hash::new();
				instance_doc.insert(Yaml::String("name".to_string()), Yaml::String(instance.name.clone()));
				instance_doc.insert(Yaml::String("path".to_string()), Yaml::String(instance.path.clone()));
				instances.push(Yaml::Hash(instance_doc));
			});

			doc.insert(Yaml::String("settings".to_string()), Yaml::Hash(settings));
			doc.insert(Yaml::String("instances".to_string()), Yaml::Array(instances));
			emitter.dump(&Yaml::Hash(doc)).unwrap();
		}

		let mut yaml_file = File::create(config_filepath).expect("Unable to create config file");
		yaml_file.write_all(yaml_text.as_bytes()).expect("Unable to write config file");
	}

	fn launch_instance(&self, instance: &BlenderInstance) {
		log::info!("Launching Blender instance: {}", instance.name);

		// pass arguments to the Blender instance
		// except the first one which is the path to this executable
		let args: Vec<String> = env::args().into_iter().skip(1).collect();
		Command::new(&instance.path)
			.args(args)
			.spawn()
			.expect("Unable to launch Blender instance");
	}
}

fn main() {
	env_logger::init();
	log::info!("Initializing Blender Launcher application");

	run_native(
		"Blender Launcher",
		NativeOptions::default(),
		Box::new(|cc| Box::new(Application::new(cc)))
	).expect("Unable to initialize application");
}
