use gtk4::prelude::*;
use libadwaita::prelude::*;
use libadwaita::Application;

const APP_ID: &str = "org.folio.Folio";

pub struct FolioApp {
    inner: Application,
}

impl FolioApp {
    pub fn new() -> Self {
        let inner = Application::builder()
            .application_id(APP_ID)
            .build();

        inner.connect_activate(|app| {
            Self::on_activate(app);
        });

        FolioApp { inner }
    }

    pub fn run(&self) {
        self.inner.run();
    }

    fn on_activate(app: &Application) {
        // On first launch: show the document launch screen.
        // The launch screen will open a document window when the user picks a file.
        let launch = crate::launch::LaunchWindow::new(app);
        launch.present();
    }
}

impl Default for FolioApp {
    fn default() -> Self { Self::new() }
}
