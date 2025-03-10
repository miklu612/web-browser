pub fn get_site(url: &str) -> String {
    ureq::get(url.to_owned())
        .call()
        .unwrap()
        .body_mut()
        .read_to_string()
        .unwrap()
}
