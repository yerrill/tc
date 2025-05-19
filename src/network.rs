struct Download {
    choked: bool,
    interested: bool,
}

impl Download {
    fn new() -> Self {
        Self {
            choked: true,
            interested: false,
        }
    }
}
