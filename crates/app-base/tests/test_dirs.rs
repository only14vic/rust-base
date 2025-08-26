use {
    app_base::prelude::*,
    std::env::{current_exe, set_current_dir}
};

#[test]
fn tests_dirs() -> Void {
    let home = getenv("HOME").unwrap();
    set_current_dir(env!("PWD"))?;
    let pwd = Dirs::cwd()?;

    let cur_exe_path = current_exe()?;
    let cur_exe_path = cur_exe_path.to_string_lossy();

    let mut dirs = Dirs::default();
    dirs.config = "{prefix}/etc/{suffix}".into();
    dirs.data = "{prefix}/share/{suffix}".into();
    dirs.init();

    assert_eq!(dirs.exe(), cur_exe_path);
    assert_eq!(&dirs.prefix, &pwd);
    assert_eq!(&dirs.suffix, "");
    assert_eq!(&dirs.home, &home);
    assert_eq!(&dirs.user_config, &format!("{home}/.config"));
    assert_eq!(&dirs.bin, &format!("{pwd}/bin"));
    assert_eq!(&dirs.lib, &format!("{pwd}/lib"));
    assert_eq!(&dirs.include, &format!("{pwd}/include"));
    assert_eq!(&dirs.config, &format!("{pwd}/etc"));
    assert_eq!(&dirs.data, &format!("{pwd}/share"));
    assert_eq!(&dirs.var, &format!("{pwd}/var"));
    assert_eq!(&dirs.state, &format!("{pwd}/var/lib"));
    assert_eq!(&dirs.cache, &format!("{pwd}/var/cache"));
    assert_eq!(&dirs.run, &format!("{pwd}/var/run"));
    assert_eq!(&dirs.log, &format!("{pwd}/var/log"));
    assert_eq!(&dirs.man, &format!("{pwd}/share/man"));
    assert_eq!(&dirs.doc, &format!("{pwd}/share/doc"));
    assert_eq!(&dirs.tmp, "/tmp");

    dirs = Dirs::default();
    dirs.config = "{prefix}/etc/{suffix}".into();
    dirs.data = "{prefix}/share/{suffix}".into();
    dirs.home = "/home/foo".into();
    dirs.prefix = "/usr/local".into();
    dirs.suffix = "myapp".into();
    dirs.var = "/var".into();
    dirs.init();

    assert_eq!(dirs.exe(), cur_exe_path);
    assert_eq!(&dirs.prefix, "/usr/local");
    assert_eq!(&dirs.suffix, "myapp");
    assert_eq!(&dirs.home, "/home/foo");
    assert_eq!(&dirs.user_config, "/home/foo/.config/myapp");
    assert_eq!(&dirs.bin, "/usr/local/bin");
    assert_eq!(&dirs.lib, "/usr/local/lib");
    assert_eq!(&dirs.include, "/usr/local/include/myapp");
    assert_eq!(&dirs.config, "/usr/local/etc/myapp");
    assert_eq!(&dirs.data, "/usr/local/share/myapp");
    assert_eq!(&dirs.man, "/usr/local/share/man/myapp");
    assert_eq!(&dirs.doc, "/usr/local/share/doc/myapp");
    assert_eq!(&dirs.var, "/var");
    assert_eq!(&dirs.state, "/var/lib/myapp");
    assert_eq!(&dirs.cache, "/var/cache/myapp");
    assert_eq!(&dirs.run, "/var/run/myapp");
    assert_eq!(&dirs.log, "/var/log/myapp");
    assert_eq!(&dirs.tmp, "/tmp/myapp");

    dbg!(&dirs);

    ok()
}
