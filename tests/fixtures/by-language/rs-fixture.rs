pub fn query_db() -> bool {}
pub fn query_db_fake() -> bool {}

mod Wrapper {
    fn private_func() {}
    pub fn public_func() -> bool {}
}

struct ContainerWithoutBlock;

struct ContainerWithBlock {
    name: &str,
}

impl ContainerWithBlock {
    pub fn container_method() {}
}

enum FileType {
    JS,
    PHP,
    RS,
}

impl FileType {
    pub fn file_type_method() {}
}

fn search_file<F>(re: &Regex, file_path: &str, config: &Config, callback: F)
where
    F: FnOnce(Vec<SearchResult>) + Send + 'static,
{
}
