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

#[derive(Clone)]
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

	// settings
	launch_on_system_startup: bool,
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
			launch_on_system_startup: false,
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

	fn add_instance_from_filepath(&mut self, filepath: PathBuf) {
		let is_folder = filepath.is_dir();
		let is_executable = filepath.extension().unwrap().to_str().unwrap() == "exe";

		if is_folder == true && is_executable == false {
			// take on the name of the folder and just assume the executable is in the folder
			let instance_name = filepath.file_name().unwrap().to_str().unwrap();
			let instance_path = filepath.join("blender.exe").to_str().unwrap().to_string();

			self.instances.push(BlenderInstance {
				name: instance_name.to_string(),
				path: instance_path,
			});
		}

		if is_folder == false && is_executable == true {
			// take on the name from the parent folder
			let instance_name = filepath.parent().unwrap().file_name().unwrap().to_str().unwrap();
			let instance_path = filepath.to_str().unwrap().to_string();

			self.instances.push(BlenderInstance {
				name: instance_name.to_string(),
				path: instance_path,
			});
		}

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
				return;
			}

			ScrollArea::vertical().show(ui, |ui| {
				for i in 0..self.instances.len() {
					ui.push_id(i, |ui| {
						self.build_instance_ui(ui, i);
					});

					ui.separator();
				}
			});
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

		let reset_modal = Modal::new(ctx, "factory reset");
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

	fn build_instance_ui(&mut self, ui: &mut egui::Ui, instance_index: usize) {
		let mut modal = Modal::new(ui.ctx(), "Error!");
		let instance = self.instances.get(instance_index).unwrap().clone();

		ui.horizontal(|ui| {
			if ui.button("Launch").clicked() {
				if let Err(error) = self.launch_instance(&instance) {
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
		}).response.context_menu(|ui| {

			// if ui.button("Rename").clicked() {
			// 	log::info!("Renaming Blender instance: {}", instance.name);
			//
			//  let new_name = ???
			//
			// 	self.instances.get_mut(instance_index).unwrap().name = new_name.to_string();
			// 	Application::save_configuration(self);
			// }

			// TODO(mathias): i don't know how to make a dialog with a text input,
			// so for now we just use a text edit field directly in the context menu
			let name_response = ui.text_edit_singleline(&mut self.instances.get_mut(instance_index).unwrap().name);
			if name_response.changed() {
				// ideally we wouldn't use .changed, since it saves the config file on every key press,
				// but .lost_focus doesn't trigger if the user clicks outside the context menu to dismiss it
				// (seemingly because using the mouse to close the context menu doesn't count as losing *keyboard* focus)
				Application::save_configuration(self);
			}

			ui.separator(); // danger zone from here on now

			if ui.button("Remove").clicked() {
				log::info!("Removing Blender instance: {}", instance.name);

				// remove instance from list
				modal.close();
				self.instances.remove(instance_index);
				Application::save_configuration(self);
			}
		});

		modal.show_dialog();
	}

	fn ensure_first_time_initialization() {
		// create environment variable if it doesn't exist
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

		// create default config file if it doesn't exist
		let config_filepath = env::var(CONFIG_FILE_ENVIRONMENT_KEY).unwrap();
		if Path::new(&config_filepath).exists() == false {
			log::info!("Creating default config file: {}", config_filepath);
			Application::save_configuration(&mut Application::default());
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
		let settings_doc = &doc["settings"];
		let instances_doc = &doc["instances"];

		// load settings
		app.launch_on_system_startup = settings_doc["launch_on_system_startup"].as_bool().unwrap();

		// load instances
		instances_doc.as_vec().unwrap().iter().for_each(|instance_doc| {
			let name = instance_doc["name"].as_str().unwrap();
			let path = instance_doc["path"].as_str().unwrap();

			// check if the blender executable still exists (as in if the user has deleted the folder)
			if Path::new(path).exists() == false {
				log::warn!("Blender instance '{}' no longer exists, skipping", name);
				return;
			}

			app.instances.push(BlenderInstance {
				name: name.to_string(),
				path: path.to_string(),
			});
		});
	}

	/// Save the configuration of Blender Launcher to a YAML file
	fn save_configuration(app: &mut Application) {
		let config_filepath = env::var(CONFIG_FILE_ENVIRONMENT_KEY).unwrap();

		log::info!("Saving configuration to: {}", config_filepath);

		let mut yaml_text = String::new(); {
			let mut emitter = yaml_rust::YamlEmitter::new(&mut yaml_text);
			let mut doc = yaml_rust::yaml::Hash::new();

			let mut settings = yaml_rust::yaml::Hash::new();
			settings.insert(Yaml::String("launch_on_system_startup".to_string()), Yaml::Boolean(app.launch_on_system_startup));

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
		NativeOptions {
			// persist_window: true // TODO(mathias): enable persistence
			..Default::default()
		},
		Box::new(|cc| {
			Box::new(Application::new(cc))
		})
	).expect("Unable to initialize application");
}
