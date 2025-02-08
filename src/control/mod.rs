use backend::Backend;

pub struct Controller {
    backend: Backend,
}

impl Controller {
    pub fn new() -> Self {
        let b = Backend::new();
        //let r = b.receiver();
        Controller {
            backend: b,
        }
    }

    pub fn backend_version(&self) -> &str {
        backend::VERSION
    }

    pub fn set_access_point_mode(&self, auto: bool) {
        self.backend.set_access_point_mode(auto);
    }
}
