use backend::Backend;

pub(crate) struct Controller {
    backend: Backend,
}

impl Controller {
    pub(crate) fn new() -> Self {
        let backend = Backend::new();
        //let r = backend.receiver();
        Controller { backend }
    }

    pub(crate) fn backend_name() -> &'static str {
        backend::NAME
    }

    pub(crate) fn backend_version() -> &'static str {
        backend::VERSION
    }

    pub(crate) fn get_access_point_mode(&self) -> bool {
        self.backend.get_access_point_mode()
    }

    pub(crate) fn set_access_point_mode(&self, auto: bool) {
        self.backend.set_access_point_mode(auto);
    }
}
