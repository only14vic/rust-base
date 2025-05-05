use {
    super::{FileMeta, Scanner},
    app_base::prelude::*,
    std::{collections::HashSet, path::Path, rc::Rc}
};

pub struct Sorter;

impl Default for Sorter {
    fn default() -> Self {
        Sorter
    }
}

impl Sorter {
    pub fn sort(&self, path: &dyn AsRef<Path>, simple: bool) -> Ok<Vec<Rc<FileMeta>>> {
        let files = Scanner::default().find(&path)?;
        let mut list = Vec::with_capacity(files.len());

        for file in files.iter() {
            let file_path = path.as_ref().to_string_lossy() + "/" + file.as_str();
            let file_meta = FileMeta::try_new(&file_path.to_string())?;
            list.push(Rc::new(file_meta));
        }

        if simple {
            list.sort();
        } else {
            self.sort_list_file_meta(&mut list)?;
        }

        Ok(list)
    }

    #[cold]
    fn sort_list_file_meta(&self, list: &mut Vec<Rc<FileMeta>>) -> Ok<()> {
        let mut all_targets: Vec<String> = Vec::new();

        for file_meta in list.iter() {
            for node_info in file_meta.nodes.iter() {
                if let Some(ref target_name) = node_info.create_target() {
                    all_targets.push(target_name.to_string());
                }
            }
        }

        for file_meta in list.iter_mut() {
            for node_info in Rc::get_mut(file_meta).unwrap().nodes.iter_mut() {
                node_info.find_need_targets(&all_targets);
            }
        }

        // first sorts by priority
        list.sort();

        let mut reorders_count = 0;

        loop {
            let mut is_reordered = false;
            let mut sort_list: Vec<Rc<FileMeta>> = Vec::new();
            let mut reordered: Vec<(Rc<FileMeta>, Rc<FileMeta>)> = Vec::new();

            for (pos, file_meta) in list.iter().enumerate() {
                if sort_list.contains(file_meta) {
                    continue;
                }

                let mut create_targets = file_meta
                    .nodes
                    .iter()
                    .filter_map(|node| node.create_target())
                    .collect::<HashSet<_>>();

                let file_name = file_meta.file_name().to_string();
                create_targets.insert(&file_name);

                let mut drop_targets = HashSet::<String>::from_iter(
                    file_meta
                        .nodes
                        .iter()
                        .map(|node| node.drop_targets.as_slice())
                        .collect::<Vec<_>>()
                        .concat()
                );

                drop_targets.retain(|target| create_targets.contains(&target) == false);

                let mut need_targets = HashSet::<String>::from_iter(
                    file_meta
                        .nodes
                        .iter()
                        .map(|node| node.need_targets.as_slice())
                        .collect::<Vec<_>>()
                        .concat()
                );

                need_targets.retain(|target| create_targets.contains(&target) == false);

                if need_targets.is_empty() && drop_targets.is_empty() {
                    sort_list.push(file_meta.clone());
                    continue;
                }

                for (pos2, file_meta_2) in list.iter().enumerate() {
                    if file_meta == file_meta_2 {
                        continue;
                    }

                    let mut create_targets_2 = file_meta_2
                        .nodes
                        .iter()
                        .filter_map(|node| node.create_target())
                        .collect::<HashSet<_>>();

                    let file_name = file_meta_2.file_name().to_string();
                    create_targets_2.insert(&file_name);

                    let mut drop_targets_2 = HashSet::<String>::from_iter(
                        file_meta_2
                            .nodes
                            .iter()
                            .map(|node| node.drop_targets.as_slice())
                            .collect::<Vec<_>>()
                            .concat()
                    );

                    drop_targets_2
                        .retain(|target| create_targets_2.contains(&target) == false);

                    let need_targets_2 = HashSet::<String>::from_iter(
                        file_meta_2
                            .nodes
                            .iter()
                            .map(|node| node.need_targets.as_slice())
                            .collect::<Vec<_>>()
                            .concat()
                    );

                    let mut found = create_targets_2.iter().any(|create_target| {
                        need_targets.contains(create_target.as_str())
                    });

                    if found == false {
                        found = need_targets_2.iter().any(|need_target| {
                            drop_targets.contains(need_target)
                                && drop_targets_2.contains(need_target) == false
                        });
                    }

                    if found && pos2 > pos {
                        sort_list.push(file_meta_2.clone());
                        reordered.push((file_meta_2.clone(), file_meta.clone()));
                        is_reordered = true;
                    }
                }

                sort_list.push(file_meta.clone());
            }

            *list = sort_list;
            reorders_count += 1;

            if is_reordered == false {
                break;
            }

            if reorders_count > 20 {
                Err(format!(
                    "Detected infinite loop during sorting. \n\
                    Last time was reordered: \n\
                    {reordered:#?}"
                ))?
            }
        }

        ok()
    }
}
