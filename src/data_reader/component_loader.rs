use log::warn;
use lru::LruCache;

use crate::data_reader::json_reader;
use crate::data_reader::json_writer::component_to_json_file;
use crate::data_reader::xml_parser::parse_xml_from_file;
use crate::model_objects::{Component, Query, SystemDeclarations};
use crate::protobuf_server::services;
use crate::protobuf_server::services::query_request::Settings;
use crate::system::input_enabler;
use crate::system::query_failures::SyntaxResult;
use crate::xml_parser;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub type ComponentsMap = HashMap<String, Component>;

struct ComponentTuple {
    components_hash: u32,
    components_map: Arc<ComponentsMap>,
}

/// A struct used for caching the models.
#[derive(Debug, Clone)]
pub struct ModelCache {
    // TODO: A concurrent lru may be faster to use and cause less prone to lock contention.
    cache: Arc<Mutex<LruCache<i32, ComponentTuple>>>,
}

impl Default for ModelCache {
    fn default() -> Self {
        Self {
            cache: Arc::new(Mutex::new(LruCache::<i32, ComponentTuple>::new(
                NonZeroUsize::new(100).unwrap(),
            ))),
        }
    }
}

impl ModelCache {
    /// A Method that creates a new cache with a given size limit.
    ///
    /// # Arguments
    ///
    /// * `cache_size` - A number representing the number of users that can be cached simultaneusly.
    pub fn new(cache_size: usize) -> Self {
        Self {
            cache: Arc::new(Mutex::new(LruCache::<i32, ComponentTuple>::new(
                NonZeroUsize::new(cache_size).unwrap(),
            ))),
        }
    }

    /// A Method that returns the model from the cache.
    ///
    /// # Arguments
    ///
    /// * `components_hash` - A hash of the components
    pub fn get_model(&self, user_id: i32, components_hash: u32) -> Option<ComponentContainer> {
        if components_hash == 0 {
            warn!("The component has no hash (0), so we assume it should not be cached.");
            return None;
        }

        let mut cache = self.cache.lock().unwrap();

        let components = cache.get(&user_id);

        components.and_then(|component_pair| {
            if component_pair.components_hash == components_hash {
                Some(ComponentContainer::new(Arc::clone(
                    &component_pair.components_map,
                )))
            } else {
                None
            }
        })
    }

    /// A method that inserts a new model into the cache.
    ///
    /// # Arguments
    ///
    /// * `components_hash` - A hash of the components
    /// * `container_components` - The `ComponentContainer's` loaded components (aka Model) to be cached.
    pub fn insert_model(
        &mut self,
        user_id: i32,
        components_hash: u32,
        container_components: Arc<ComponentsMap>,
    ) -> ComponentContainer {
        if components_hash == 0 {
            warn!("The component has no hash (0), so we assume it should not be cached.");
            return ComponentContainer::new(container_components);
        }

        self.cache.lock().unwrap().put(
            user_id,
            ComponentTuple {
                components_hash,
                components_map: Arc::clone(&container_components),
            },
        );

        ComponentContainer::new(container_components)
    }
}

pub trait ComponentLoader {
    fn get_component(&mut self, component_name: &str) -> Result<&Component, SyntaxResult>;
    fn save_component(&mut self, component: Component);
    fn get_settings(&self) -> &Settings;
    fn get_settings_mut(&mut self) -> &mut Settings;
}

#[derive(Debug, Default, Clone)]
pub struct ComponentContainer {
    pub loaded_components: Arc<ComponentsMap>,
    settings: Option<Settings>,
}

impl ComponentLoader for ComponentContainer {
    fn get_component(&mut self, component_name: &str) -> Result<&Component, SyntaxResult> {
        if let Some(component) = self.loaded_components.get(component_name) {
            assert_eq!(component_name, component.name);
            Ok(component)
        } else {
            panic!("The component '{}' could not be retrieved", component_name);
        }
    }
    fn save_component(&mut self, _component: Component) {
        //Intentionally left blank (no-op func)
    }

    fn get_settings(&self) -> &Settings {
        self.settings.as_ref().unwrap()
    }

    fn get_settings_mut(&mut self) -> &mut Settings {
        self.settings.as_mut().unwrap()
    }
}

impl ComponentContainer {
    pub fn new(map: Arc<ComponentsMap>) -> Self {
        ComponentContainer {
            loaded_components: map,
            settings: None,
        }
    }

    /// Sets the settings
    pub(crate) fn set_settings(&mut self, settings: Settings) {
        self.settings = Some(settings);
    }
}

impl From<Vec<Component>> for ComponentContainer {
    fn from(components: Vec<Component>) -> Self {
        let mut comp_hashmap = HashMap::<String, Component>::new();
        for mut component in components {
            log::trace!("Adding comp {} to container", component.name);
            let inputs: Vec<_> = component.get_input_actions();
            input_enabler::make_input_enabled(&mut component, &inputs);
            comp_hashmap.insert(component.name.to_string(), component);
        }
        ComponentContainer::new(Arc::new(comp_hashmap))
    }
}

pub fn parse_components_if_some(
    proto_component: &services::Component,
) -> Result<Vec<Component>, tonic::Status> {
    if let Some(rep) = &proto_component.rep {
        match rep {
            services::component::Rep::Json(json) => parse_json_component(json),
            services::component::Rep::Xml(xml) => Ok(parse_xml_components(xml)),
        }
    } else {
        Ok(vec![])
    }
}

fn parse_json_component(json: &str) -> Result<Vec<Component>, tonic::Status> {
    match json_reader::json_to_component(json) {
        Ok(comp) => Ok(vec![comp]),
        Err(_) => Err(tonic::Status::invalid_argument(
            "Failed to parse json component",
        )),
    }
}

fn parse_xml_components(xml: &str) -> Vec<Component> {
    let (comps, _, _) = xml_parser::parse_xml_from_str(xml);
    comps
}

pub trait ProjectLoader: ComponentLoader {
    fn get_declarations(&self) -> &SystemDeclarations;
    fn get_queries(&self) -> &Vec<Query>;
    fn get_project_path(&self) -> &PathBuf;
    fn to_comp_loader(self: Box<Self>) -> Box<dyn ComponentLoader>;
}

pub struct JsonProjectLoader {
    project_path: PathBuf,
    loaded_components: ComponentsMap,
    system_declarations: SystemDeclarations,
    queries: Vec<Query>,
    settings: Settings,
}

impl ComponentLoader for JsonProjectLoader {
    fn get_component(&mut self, component_name: &str) -> Result<&Component, SyntaxResult> {
        if !self.is_component_loaded(component_name) {
            self.load_component(component_name)?;
        }

        if let Some(component) = self.loaded_components.get(component_name) {
            assert_eq!(component_name, component.name);
            Ok(component)
        } else {
            panic!("The component '{}' could not be retrieved", component_name);
        }
    }

    fn save_component(&mut self, component: Component) {
        component_to_json_file(&self.project_path, &component);
        self.loaded_components
            .insert(component.name.clone(), component);
    }

    fn get_settings(&self) -> &Settings {
        &self.settings
    }
    fn get_settings_mut(&mut self) -> &mut Settings {
        &mut self.settings
    }
}

impl ProjectLoader for JsonProjectLoader {
    fn get_declarations(&self) -> &SystemDeclarations {
        &self.system_declarations
    }

    fn get_queries(&self) -> &Vec<Query> {
        &self.queries
    }

    fn get_project_path(&self) -> &PathBuf {
        &self.project_path
    }

    fn to_comp_loader(self: Box<Self>) -> Box<dyn ComponentLoader> {
        self
    }
}

impl JsonProjectLoader {
    #[allow(clippy::new_ret_no_self)]
    pub fn new_loader<P: AsRef<Path>>(
        project_path: P,
        settings: Settings,
    ) -> Box<dyn ProjectLoader> {
        let system_declarations = json_reader::read_system_declarations(&project_path).unwrap();
        let queries = json_reader::read_queries(&project_path).unwrap();

        Box::new(JsonProjectLoader {
            project_path: project_path.as_ref().to_path_buf(),
            loaded_components: HashMap::new(),
            system_declarations,
            queries,
            settings,
        })
    }

    fn load_component(&mut self, component_name: &str) -> Result<(), SyntaxResult> {
        let mut component = json_reader::read_json_component(&self.project_path, component_name)?;

        let opt_inputs = self
            .get_declarations()
            .get_component_inputs(&component.name);
        if let Some(inputs) = opt_inputs {
            input_enabler::make_input_enabled(&mut component, inputs);
        }

        self.loaded_components
            .insert(String::from(component_name), component);

        Ok(())
    }

    fn is_component_loaded(&self, component_name: &str) -> bool {
        self.loaded_components.contains_key(component_name)
    }
}

pub struct XmlProjectLoader {
    project_path: PathBuf,
    loaded_components: ComponentsMap,
    system_declarations: SystemDeclarations,
    queries: Vec<Query>,
    settings: Settings,
}

impl ComponentLoader for XmlProjectLoader {
    fn get_component(&mut self, component_name: &str) -> Result<&Component, SyntaxResult> {
        if let Some(component) = self.loaded_components.get(component_name) {
            assert_eq!(component_name, component.name);
            Ok(component)
        } else {
            panic!("The component '{}' could not be retrieved", component_name);
        }
    }

    fn save_component(&mut self, _: Component) {
        panic!("Saving components is not supported for XML projects")
    }

    fn get_settings(&self) -> &Settings {
        &self.settings
    }
    fn get_settings_mut(&mut self) -> &mut Settings {
        &mut self.settings
    }
}

impl ProjectLoader for XmlProjectLoader {
    fn get_declarations(&self) -> &SystemDeclarations {
        &self.system_declarations
    }

    fn get_queries(&self) -> &Vec<Query> {
        &self.queries
    }

    fn get_project_path(&self) -> &PathBuf {
        &self.project_path
    }

    fn to_comp_loader(self: Box<Self>) -> Box<dyn ComponentLoader> {
        self
    }
}

impl XmlProjectLoader {
    #[allow(clippy::new_ret_no_self)]
    pub fn new_loader<P: AsRef<Path>>(
        project_path: P,
        settings: Settings,
    ) -> Box<dyn ProjectLoader> {
        let (comps, system_declarations, queries) = parse_xml_from_file(&project_path);

        let mut map = HashMap::<String, Component>::new();
        for mut component in comps {
            let opt_inputs = system_declarations.get_component_inputs(&component.name);
            if let Some(opt_inputs) = opt_inputs {
                input_enabler::make_input_enabled(&mut component, opt_inputs);
            }

            let name = String::from(&component.name);
            map.insert(name, component);
        }

        Box::new(XmlProjectLoader {
            project_path: project_path.as_ref().to_path_buf(),
            loaded_components: map,
            system_declarations,
            queries,
            settings,
        })
    }
}
