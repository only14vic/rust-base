use {app_base::prelude::*, std::env::current_exe};

#[test]
fn tests_dirs() -> Void {
    let home = getenv("HOME").unwrap();
    let pwd = Dirs::cwd()?;

    let cur_exe_path = current_exe()?;
    let cur_exe_path = cur_exe_path.to_string_lossy();

    let mut dirs = Dirs::default();

    assert_eq!(dirs.exe(), cur_exe_path);
    assert_eq!(dirs.prefix(), "");
    assert_eq!(dirs.suffix(), "");
    assert_eq!(dirs.home(), &home);
    assert_eq!(&dirs.user_config, &format!("{home}/.config"));
    assert_eq!(&dirs.bin, "bin");
    assert_eq!(&dirs.lib, "lib");
    assert_eq!(&dirs.config, "etc");
    assert_eq!(&dirs.data, "share");
    assert_eq!(&dirs.state, "var/lib");
    assert_eq!(&dirs.cache, "var/cache");
    assert_eq!(&dirs.runtime, "var/run");
    assert_eq!(&dirs.log, "var/log");
    assert_eq!(&dirs.tmp, "/tmp");

    dirs.with_home("/home/foo")
        .with_prefix("/usr/local")
        .with_suffix("myapp");

    assert_eq!(dirs.exe(), cur_exe_path);
    assert_eq!(dirs.prefix(), "/usr/local");
    assert_eq!(dirs.suffix(), "myapp");
    assert_eq!(dirs.home(), "/home/foo");
    assert_eq!(&dirs.user_config, "/home/foo/.config/myapp");
    assert_eq!(&dirs.bin, "/usr/local/bin");
    assert_eq!(&dirs.lib, "/usr/local/lib/myapp");
    assert_eq!(&dirs.config, "/usr/local/etc/myapp");
    assert_eq!(&dirs.data, "/usr/local/share/myapp");
    assert_eq!(&dirs.state, "/usr/local/var/lib/myapp");
    assert_eq!(&dirs.cache, "/usr/local/var/cache/myapp");
    assert_eq!(&dirs.runtime, "/usr/local/var/run/myapp");
    assert_eq!(&dirs.log, "/usr/local/var/log/myapp");
    assert_eq!(&dirs.tmp, "/tmp/myapp");

    dirs.with_home("~").with_prefix(".").with_suffix("app");

    assert_eq!(dirs.exe(), cur_exe_path);
    assert_eq!(dirs.prefix(), &pwd);
    assert_eq!(dirs.suffix(), "app");
    assert_eq!(dirs.home(), &home);
    assert_eq!(&dirs.user_config, &format!("{home}/.config/app"));
    assert_eq!(&dirs.bin, &format!("{pwd}/bin"));
    assert_eq!(&dirs.lib, &format!("{pwd}/lib/app"));
    assert_eq!(&dirs.config, &format!("{pwd}/etc/app"));
    assert_eq!(&dirs.data, &format!("{pwd}/share/app"));
    assert_eq!(&dirs.state, &format!("{pwd}/var/lib/app"));
    assert_eq!(&dirs.cache, &format!("{pwd}/var/cache/app"));
    assert_eq!(&dirs.runtime, &format!("{pwd}/var/run/app"));
    assert_eq!(&dirs.log, &format!("{pwd}/var/log/app"));
    assert_eq!(&dirs.tmp, "/tmp/app");

    dbg!(&dirs);

    ok()
}
