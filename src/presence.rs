pub(crate) mod motion;
pub(crate) mod observer;
pub(crate) mod rig;

use bevy::{asset::io::embedded::EmbeddedAssetRegistry, prelude::*};

pub(crate) struct Runtime {
    pub source: &'static str,
}

impl Plugin for Runtime {
    fn build(&self, app: &mut App) {
        app.world()
            .resource::<EmbeddedAssetRegistry>()
            .insert_asset(
                file!().into(),
                std::path::Path::new("ainda_vivo/presenca.glb"),
                include_bytes!("../assets/models/presenca.glb").as_slice(),
            );
        let physical_smoke = app.world().resource::<crate::smoke::SmokeTest>().campaign;
        let legacy = std::env::args().any(|argument| {
            matches!(
                argument.as_str(),
                "--presence-legacy" | "--smoke-reality" | "--smoke-lab" | "--lab-campaign-preview"
            ) || argument == "--smoke-test" && !physical_smoke
        });
        app.insert_resource(rig::Source(self.source))
            .init_resource::<rig::Catalog>()
            .insert_resource(observer::Campaign::new(!legacy))
            .add_systems(Startup, rig::begin_import)
            .add_systems(Update, rig::finish_import)
            .add_systems(
                Update,
                observer::update_campaign
                    .after(crate::atmosphere::director::update)
                    .before(crate::scene::presence::update)
                    .run_if(crate::app::campaign_running),
            )
            .add_systems(
                Update,
                rig::animate_observer.after(observer::update_campaign),
            );
    }
}
