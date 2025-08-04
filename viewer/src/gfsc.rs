use bevy::{app::Plugin, render::{renderer::RenderDevice, settings::WgpuFeatures, RenderApp}};

pub struct GpuFeatureSupportChecker;

impl Plugin for GpuFeatureSupportChecker {
    fn build(&self, _app: &mut bevy::app::App) {}

    fn finish(&self, app: &mut bevy::app::App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        let render_device = render_app.world().resource::<RenderDevice>();

        // Check if the device support the required feature. If not, exit the example.
        // In a real application, you should setup a fallback for the missing feature
        if !render_device
            .features()
            .contains(WgpuFeatures::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING)
        {
            bevy::log::error!(
                "Render device doesn't support feature \
SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING, \
which is required for texture binding arrays"
            );
            std::process::exit(1);
        }
    }
}

