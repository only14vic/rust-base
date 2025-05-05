use {
    app_base::prelude::*,
    std::{fs::read_dir, ops::BitOr, path::Path}
};

pub struct Scanner {
    exclude: Vec<&'static str>
}

impl Default for Scanner {
    fn default() -> Self {
        Self { exclude: vec![".down.sql"] }
    }
}

impl Scanner {
    pub fn find(&self, path: &dyn AsRef<Path>) -> Ok<Vec<String>> {
        let mut files = self.find_inner(&path)?;
        files.sort_by_key(|a| a[a.rfind('/').map(|p| p + 1).unwrap_or(0)..].to_string());

        Ok(files)
    }

    #[allow(clippy::only_used_in_recursion)]
    fn find_inner(&self, path: &dyn AsRef<Path>) -> Ok<Vec<String>> {
        let mut files = Vec::new();
        let mut files_exclude = Vec::new();
        let app_env = Env::env();

        for entry in read_dir(path).map_err(|err| {
            std::io::Error::new(
                err.kind(),
                format!(
                    "{err}: {path}",
                    err = err,
                    path = path.as_ref().to_string_lossy()
                )
            )
        })? {
            let entry = entry?;
            let file_name = entry.file_name().to_string_lossy().to_string();

            if entry.file_type()?.is_file()
                && file_name.ends_with(".sql")
                && self
                    .exclude
                    .iter()
                    .fold(false, |acc, &item| acc.bitor(file_name.ends_with(item)))
                    == false
            {
                let env_in_file = file_name
                    .split_terminator('.')
                    .skip(1)
                    .find(|s| ["dev", "prod", "test"].contains(s));

                if env_in_file.is_none() {
                    files.push(file_name.clone());
                } else if env_in_file == Some(app_env) {
                    files.push(file_name.clone());
                    files_exclude.push(file_name.replace(&format!(".{app_env}."), "."));
                }
            }

            if entry.file_type()?.is_dir() {
                let mut dir_files = self.find_inner(&entry.path())?;
                dir_files
                    .iter_mut()
                    .for_each(|item| *item = format!("{}/{}", &file_name, item));
                files.extend(dir_files)
            }
        }

        files_exclude.iter().for_each(|file| {
            if let Some(pos) = files.iter().position(|f| f == file) {
                files.remove(pos);
            }
        });

        Ok(files)
    }
}
