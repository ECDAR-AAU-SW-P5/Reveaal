#[cfg(test)]
mod samples {
    use crate::data_reader::component_loader::JsonProjectLoader;

    const CONJUNCTION_SAMPLE: &str = "samples/json/Conjunction";

    #[test]
    fn test_locations_t1() {
        let mut project_loader =
            JsonProjectLoader::new_loader(CONJUNCTION_SAMPLE, crate::tests::TEST_SETTINGS);
        let t1 = project_loader.get_component("Test1").unwrap();

        assert_eq!(t1.name, "Test1");
        assert_eq!(t1.locations.len(), 2);
    }

    #[test]
    fn test_locations_t2() {
        let mut project_loader =
            JsonProjectLoader::new_loader(CONJUNCTION_SAMPLE, crate::tests::TEST_SETTINGS);
        let t2 = project_loader.get_component("Test2").unwrap();

        assert_eq!(t2.name, "Test2");
        assert_eq!(t2.locations.len(), 2);
    }

    #[test]
    fn test_locations_t3() {
        let mut project_loader =
            JsonProjectLoader::new_loader(CONJUNCTION_SAMPLE, crate::tests::TEST_SETTINGS);
        let t3 = project_loader.get_component("Test3").unwrap();

        assert_eq!(t3.name, "Test3");
        assert_eq!(t3.locations.len(), 3);
    }

    #[test]
    fn test_names_t1_through_t12() {
        let mut project_loader =
            JsonProjectLoader::new_loader(CONJUNCTION_SAMPLE, crate::tests::TEST_SETTINGS);

        for i in 1..12 {
            let t = project_loader
                .get_component(&format!("Test{}", i).to_string())
                .unwrap();

            assert_eq!(t.name, format!("Test{}", i));
        }
    }
}
