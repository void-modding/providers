use std::sync::Arc;

use lib_vmm::{api::ProviderApi, registry::ProviderSource, runtime::ContextBuilder, traits::mod_provider::ModProvider};

mod game_providers;
mod mod_providers;
mod helper;

/// This is not how we should do this, but it works for now to just get *something*
/// In the future, we should use the same method as whatever normal plugins will use
pub fn register_all_providers(ctx_builder: &mut ContextBuilder, api: Arc<dyn ProviderApi>) {
    let payday2_game_provider = Arc::new(game_providers::Payday2Provider::new());
    let modworkshop_provider = Arc::new(mod_providers::ModWorkShopProvider::new(api));

    // All of this is bad, we shouldn't do this, especially passing context directly to plugins
    // but since this is a "trusted" plugin, and we don't have a better way of loading plugins yet, it'll have to do
    ctx_builder
        .register_mod_provider(&modworkshop_provider.register(), modworkshop_provider, ProviderSource::Core)
        .expect("Failed to register Modworkshop mod provider");

    ctx_builder
        .register_game_provider(payday2_game_provider, ProviderSource::Core)
        .expect("Failed to register PAYDAY 2 game provider");
}
