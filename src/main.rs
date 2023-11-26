use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use eframe::{Frame, NativeOptions, run_native};
use egui::{CentralPanel, Context, ScrollArea, TopBottomPanel};
use log;
use std::process::Command;
use egui_modal::{Modal};

extern crate yaml_rust;
use yaml_rust::{Yaml, YamlLoader};

const CONFIG_FILE_ENVIRONMENT_KEY: &str = "BLENDER_LAUNCHER_CONFIG_FILEPATH";
const CONFIG_FILE_FILENAME: &str = "blender_launcher_config.yaml";

struct BlenderInstance {
	name: String,
	path: String,
}

enum AppView {
	Instances, // "home" view
	Settings,
}

struct Application {
	current_view: AppView,
	version: String,
	instances: Vec<BlenderInstance>,
}

impl eframe::App for Application {
	fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
		match self.current_view {
			AppView::Instances => self.build_instances_list_ui(ctx),
			AppView::Settings => self.build_settings_ui(ctx),
		}

		// collect dropped files
		ctx.input(|i| {
			if i.raw.dropped_files.is_empty() == false {
				let dropped_files = i.raw.dropped_files.clone();

				for dropped_file in dropped_files {
					let dropped_filepath = dropped_file.path.clone().unwrap();
					self.add_instance_from_filepath(dropped_filepath);
				}
			}
		})
	}
}

impl Default for Application {
	fn default() -> Self {
		Self {
			current_view: AppView::Instances,
			version: env!("CARGO_PKG_VERSION").to_string(),
			instances: Vec::new(),
		}
	}
}

impl Application {
	fn new(_cc: &eframe::CreationContext<'_>) -> Self {
		let mut app = Application::default();

		Application::ensure_first_time_initialization();

		let config_filepath = env::var(CONFIG_FILE_ENVIRONMENT_KEY);
		Application::load_configuration(config_filepath.unwrap(), &mut app);

		return app;
	}

	fn add_instance_from_filepath(&mut self, executable_path: PathBuf) {
		let executable_filename = executable_path.file_stem().unwrap().to_str().unwrap();

		let instance_name = executable_filename.to_string();
		let instance_path = executable_path.to_str().unwrap().to_string();

		self.instances.push(BlenderInstance {
			name: instance_name,
			path: instance_path,
		});

		Application::save_configuration(self);
	}

	fn build_instances_list_ui(&mut self, ctx: &Context) {
		TopBottomPanel::top("top_panel").show(ctx, |ui| {
			ui.horizontal(|ui| {
				ui.label("Blender Launcher");
				ui.label(format!("v{}", self.version));

				ui.with_layout(egui::Layout::right_to_left(Default::default()), |ui| {
					if ui.button("Settings").clicked() {
						self.current_view = AppView::Settings;
					}
				});
			});
		});


		CentralPanel::default().show(ctx, |ui| {
			if self.instances.len() == 0 {
				self.build_no_instances_ui(ui);
			} else {
				ScrollArea::vertical().show(ui, |ui| {
					self.instances.iter().for_each(|instance| {
						ui.push_id(&instance.name, |ui| {
							self.build_instance_ui(ui, instance);
						});

						ui.separator();
					});
				});
			}
		});
	}

	fn build_no_instances_ui(&mut self, ui: &mut egui::Ui) {
		ui.centered_and_justified(|ui| {
			ui.add_space(16.0);
			ui.group(|ui| {
				ui.centered_and_justified(|ui| {
					ui.vertical_centered(|ui| {
						ui.add_space(64.0); // TODO(mathias): couldn't figure out how to center the text vertically too
						ui.heading("No Blender instances found!");
						ui.label("Drag and drop a Blender executable here to add it to the list.");
					});
				});
			});
		});
	}

	fn build_settings_ui(&mut self, ctx: &Context) {
		TopBottomPanel::top("top_panel").show(ctx, |ui| {
			ui.horizontal(|ui| {
				ui.label("Blender Launcher");
				ui.label(format!("v{}", self.version));

				ui.with_layout(egui::Layout::right_to_left(Default::default()), |ui| {
					if ui.button("Instances").clicked() {
						self.current_view = AppView::Instances;
					}
				});
			});
		});

		let mut reset_modal = Modal::new(ctx, "factory reset");
		reset_modal.show(|ui| {
			reset_modal.title(ui,"Confirm factory reset");
			reset_modal.frame(ui, |ui| {
				reset_modal.body(ui,"Are you sure you want to reset Blender Launcher to its default settings?");
			});

			reset_modal.buttons(ui, |ui| {
				if ui.button("Cancel").clicked() {
					reset_modal.close();
				}

				if ui.button("Confirm").clicked() {
					log::info!("Factory resetting Blender Launcher");
					*self = Application::default();
					Application::save_configuration(self);
					reset_modal.close();
				}
			});
		});

		CentralPanel::default().show(ctx, |ui| {
			if ui.button("Factory reset settings").clicked() {
				reset_modal.open();
			}
		});
	}

	fn build_instance_ui(&self, ui: &mut egui::Ui, instance: &BlenderInstance) {
		let mut modal = Modal::new(ui.ctx(), "Error!");

		ui.horizontal(|ui| {
			if ui.button("Launch").clicked() {
				if let Err(error) = self.launch_instance(instance) {
					log::error!("Unable to launch Blender instance: {}", error);

					modal.dialog()
						.with_title("Error!")
						.with_body(format!("Unable to launch Blender instance: {}", error))
						.open();
				}
			}

			ui.vertical(|ui| {
				ui.label(&instance.name);
				ui.label(&instance.path);
			});
		});

		modal.show_dialog();
	}

	fn ensure_first_time_initialization() {
		// create environment variable
		if let Err(_) = env::var(CONFIG_FILE_ENVIRONMENT_KEY) {
			log::info!("Creating environment variable: {}", CONFIG_FILE_ENVIRONMENT_KEY);
			// set environment variable to %USERPROFILE%/blender_launcher_config.yaml
			let value = Path::new(".")
				.join(simple_home_dir::home_dir().unwrap().as_path())
				.join(CONFIG_FILE_FILENAME)
				.to_str().unwrap().to_owned();

			set_env::set(CONFIG_FILE_ENVIRONMENT_KEY, value.clone()).expect("Failed to set environment variable");

			// set_env::set doesn't seem to work perfectly,
			// env::var() returns an error even though the environment variable is set
			// so, for now we set the variable both ways
			env::set_var(CONFIG_FILE_ENVIRONMENT_KEY, value.clone());
		}

		// create default config file
		let config_filepath = env::var(CONFIG_FILE_ENVIRONMENT_KEY).unwrap();
		if Path::new(&config_filepath).exists() == false {
			log::info!("Creating default config file: {}", config_filepath);
			Application::save_configuration(config_filepath, &Application::default());
		}
	}

	/// Load the configuration of Blender Launcher from a YAML file
	fn load_configuration(config_filepath: String, app: &mut Application) {
		log::info!("Loading configuration from: {}", config_filepath);

		// read config file
		let mut file = File::open(config_filepath).expect("Unable to open config file");
		let mut yaml = String::new();
		file.read_to_string(&mut yaml).expect("Unable to read config file");

		let doc = YamlLoader::load_from_str(&*yaml).unwrap()[0].clone();
		let _settings_doc = &doc["settings"];
		let instances_doc = &doc["instances"];

		instances_doc.as_vec().unwrap().iter().for_each(|instance_doc| {
			let name = instance_doc["name"].as_str().unwrap();
			let path = instance_doc["path"].as_str().unwrap();

			app.instances.push(BlenderInstance {
				name: name.to_string(),
				path: path.to_string(),
			});
		});
	}

	/// Save the configuration of Blender Launcher to a YAML file
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

	/// Launch a Blender instance
	fn launch_instance(&self, instance: &BlenderInstance) -> Result<std::process::Child, std::io::Error> {
		log::info!("Launching Blender instance: {}", instance.name);

		// pass arguments to the Blender instance
		// except the first one which is the path to this executable
		let args: Vec<String> = env::args().into_iter().skip(1).collect();
		Command::new(&instance.path)
			.args(args)
			.spawn()
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
