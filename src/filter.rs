pub struct DltFilter {
    ecu_id: Option<String>,
    application_id: Option<String>,
    context_id: Option<String>,
}

impl DltFilter {
    pub fn new() -> Self {
        Self {
            ecu_id: None,
            application_id: None,
            context_id: None,
        }
    }

    pub fn with_ecu_id(mut self, ecu_id: String) -> Self {
        self.ecu_id = Some(ecu_id);
        self
    }

    pub fn with_application_id(mut self, application_id: String) -> Self {
        self.application_id = Some(application_id);
        self
    }

    pub fn with_context_id(mut self, context_id: String) -> Self {
        self.context_id = Some(context_id);
        self
    }
}

impl Default for DltFilter {
    fn default() -> Self {
        Self::new()
    }
}
