use reveaal::tests::TEST_SETTINGS;
use reveaal::{ComponentLoader, JsonProjectLoader};

const UNI_PATH: &str = "samples/json/EcdarUniversity";

pub fn get_uni_loader(disable_clock_reduction: bool) -> Box<dyn ComponentLoader + 'static> {
    let mut loader = JsonProjectLoader::new_loader(UNI_PATH, TEST_SETTINGS).to_comp_loader();
    loader.get_settings_mut().disable_clock_reduction = disable_clock_reduction;
    let _ = loader.get_component("Adm2");
    let _ = loader.get_component("Administration");
    let _ = loader.get_component("HalfAdm1");
    let _ = loader.get_component("HalfAdm2");
    let _ = loader.get_component("Machine");
    let _ = loader.get_component("Machine2");
    let _ = loader.get_component("Machine3");
    let _ = loader.get_component("Machine4");
    let _ = loader.get_component("Researcher");
    let _ = loader.get_component("Spec");
    loader
}
