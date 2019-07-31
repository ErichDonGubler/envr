use {
    cargo_deny::{
        ban::{check_bans, Config as BansConfig, DupGraph},
        get_all_crates,
        licenses::{check_licenses, Config as LicensesConfig, Gatherer},
        Crates,
    },
    serde::Deserialize,
    slog::{o, Drain, Logger},
    std::{collections::hash_map::DefaultHasher, hash::BuildHasherDefault, sync::Mutex},
    toml::de::from_str as from_toml_str,
};

fn get_slogger() -> Logger {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    Logger::root(drain, o!())
}

#[derive(Deserialize)]
struct Config {
    licenses: Option<LicensesConfig>,
    bans: Option<BansConfig>,
}

fn get_crates() -> Crates {
    get_all_crates(env!("CARGO_MANIFEST_DIR")).unwrap()
}

fn get_config() -> Config {
    from_toml_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/deny.toml",
    )))
    .unwrap()
}

#[test]
fn no_dupes() {
    if let Config {
        bans: Some(config),
        licenses: _,
    } = get_config()
    {
        let dupe_crate_deps = Mutex::new(Vec::new());
        check_bans(
            get_slogger(),
            &get_crates(),
            &config,
            Some(|g: DupGraph| {
                dupe_crate_deps.lock().unwrap().push(g.duplicate);
                Ok(())
            }),
        )
        .unwrap();
        let dupe_crate_deps = dupe_crate_deps.lock().unwrap();
        if !dupe_crate_deps.is_empty() {
            panic!("duplicate crate dependencies found: {:?}", dupe_crate_deps);
        }
    }
}

#[test]
fn licenses_are_compatible() {
    if let Config {
        bans: _,
        licenses: Some(config),
    } = get_config()
    {
        let logger = get_slogger();
        check_licenses(
            logger.clone(),
            Gatherer::new(logger).gather::<BuildHasherDefault<DefaultHasher>>(
                &get_crates().crates,
                Default::default(),
            ),
            &config,
        )
        .unwrap();
    }
}
