use crate::core::constants::{
    ENV_RAFCTL_PROFILE, ENV_RAFCTL_PROFILE_TOOL, ENV_RAFCTL_VERSION, VERSION,
};
use crate::core::profile::{load_profile, resolve_profile_alias};
use crate::error::RafctlError;

pub fn handle_env(profile_name: &str) -> Result<(), RafctlError> {
    let resolved_name = resolve_profile_alias(profile_name)?;
    let name_lower = resolved_name.to_lowercase();

    let profile = load_profile(&name_lower)?;
    let config_dir = profile.tool.config_dir_for_profile(&name_lower)?;

    println!(
        "export {}=\"{}\"",
        profile.tool.env_var_name(),
        config_dir.display()
    );
    println!("export {}=\"{}\"", ENV_RAFCTL_PROFILE, profile.name);
    println!("export {}=\"{}\"", ENV_RAFCTL_PROFILE_TOOL, profile.tool);
    println!("export {}=\"{}\"", ENV_RAFCTL_VERSION, VERSION);

    Ok(())
}
