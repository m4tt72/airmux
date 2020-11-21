use super::*;
use clap::crate_name;
use std::ffi::OsString;
use std::os;
use std::path::PathBuf;
use tempfile::tempdir;

const APP_NAME: &'static str = crate_name!();
const APP_AUTHOR: &'static str = "dermoumi";

fn make_config(
    app_name: Option<&'static str>,
    app_author: Option<&'static str>,
    tmux_command: Option<OsString>,
    config_dir: Option<PathBuf>,
) -> Config {
    Config {
        app_name: app_name.unwrap_or(APP_NAME),
        app_author: app_author.unwrap_or(APP_AUTHOR),
        tmux_command: tmux_command.unwrap_or(OsString::from("tmux")),
        config_dir,
    }
}

#[test]
fn edit_project_fails_when_editor_is_empty() {
    let temp_dir = tempdir().unwrap();
    let temp_dir = temp_dir.path().to_path_buf();
    let test_config = make_config(None, None, None, Some(temp_dir));

    assert!(matches!(
        edit_project(&test_config, "project", "", false)
            .err()
            .unwrap()
            .downcast_ref::<Error>()
            .unwrap(),
        Error::EditorEmpty {}
    ))
}

#[test]
fn edit_project_succeeds_when_project_file_does_not_exist() {
    let temp_dir = tempdir().unwrap();
    let temp_dir = temp_dir.path().to_path_buf();
    let test_config = make_config(None, None, None, Some(temp_dir));
    let project_name = "project";
    let project_path = test_config
        .get_projects_dir(project_name)
        .unwrap()
        .with_extension("yml");

    let result = edit_project(&test_config, project_name, "test", false);

    assert!(project_path.is_file());
    assert!(result.is_ok());
}

#[test]
fn edit_project_succeeds_when_project_file_exists() {
    let temp_dir = tempdir().unwrap();
    let temp_dir = temp_dir.path().to_path_buf();
    let test_config = make_config(None, None, None, Some(temp_dir));
    let project_name = "project";

    // Make sure the file exists
    let projects_dir = test_config.get_projects_dir("").unwrap();
    let project_path = projects_dir.join(project_name).with_extension("yml");
    mkdirp(projects_dir).unwrap();
    edit::create_project(project_name, &project_path).unwrap();
    assert!(project_path.is_file());

    // Run edit_project
    let result = edit_project(&test_config, project_name, "test", false);

    assert!(project_path.is_file());
    assert!(result.is_ok());
}

#[test]
fn edit_project_creates_sub_directories_as_needed() {
    let temp_dir = tempdir().unwrap();
    let temp_dir = temp_dir.path().to_path_buf();
    let test_config = make_config(None, None, None, Some(temp_dir));
    let project_name = "subdir1/subdir2/project";
    let project_path = test_config
        .get_projects_dir(project_name)
        .unwrap()
        .with_extension("yml");
    let subdir_path = test_config.get_projects_dir("subdir1/subdir2").unwrap();

    let result = edit_project(&test_config, project_name, "test", false);

    assert!(subdir_path.is_dir());
    assert!(project_path.is_file());
    assert!(result.is_ok());
}

#[test]
fn edit_project_fails_when_project_path_is_directory() {
    let temp_dir = tempdir().unwrap();
    let temp_dir = temp_dir.path().to_path_buf();
    let test_config = make_config(None, None, None, Some(temp_dir));
    let project_name = "project";
    let project_path = test_config
        .get_projects_dir(project_name)
        .unwrap()
        .with_extension("yml");

    mkdirp(&project_path).unwrap();
    assert!(&project_path.is_dir());

    let result = edit_project(&test_config, project_name, "test", false);
    assert!(result.is_err());
    assert!(matches!(
        result.err().unwrap().downcast_ref::<Error>().unwrap(),
        Error::ProjectFileIsADirectory { path } if path == &project_path
    ));
}

#[test]
fn edit_project_project_name_cannot_be_empty() {
    let temp_dir = tempdir().unwrap();
    let temp_dir = temp_dir.path().to_path_buf();
    let test_config = make_config(None, None, None, Some(temp_dir));
    let project_name = "";

    let result = edit_project(&test_config, project_name, "test", false);
    assert!(result.is_err());
    assert!(matches!(
        result.err().unwrap().downcast_ref::<Error>().unwrap(),
        Error::ProjectNameEmpty {}
    ));
}

#[test]
fn remove_project_removes_existing_project() {
    let temp_dir = tempdir().unwrap();
    let temp_dir = temp_dir.path().to_path_buf();
    let test_config = make_config(None, None, None, Some(temp_dir));
    let project_name = "project";

    // Make sure the file exists
    let projects_dir = test_config.get_projects_dir("").unwrap();
    let project_path = projects_dir.join(project_name).with_extension("yml");
    mkdirp(projects_dir).unwrap();
    edit::create_project(project_name, &project_path).unwrap();
    assert!(project_path.is_file());

    let result = remove_project(&test_config, project_name, true);
    assert!(result.is_ok());
    assert!(!project_path.exists());
}

#[test]
fn remove_project_removes_parent_subdirectories_if_empty() {
    let temp_dir = tempdir().unwrap();
    let temp_dir = temp_dir.path().to_path_buf();
    let test_config = make_config(None, None, None, Some(temp_dir));
    let project_name = "subdir1/subdir2/project";

    // Make sure the project's parent directory exists
    let namespace = utils::get_project_namespace(project_name).unwrap();
    let data_dir = test_config.get_projects_dir("").unwrap();
    mkdirp(data_dir.join(&namespace)).unwrap();

    // Make sure the file exists
    let projects_dir = test_config.get_projects_dir("").unwrap();
    let project_path = projects_dir.join(project_name).with_extension("yml");
    edit::create_project(project_name, &project_path).unwrap();
    assert!(project_path.is_file());

    let result = remove_project(&test_config, project_name, true);
    assert!(result.is_ok());
    assert!(!project_path.exists());
    assert!(!project_path.parent().unwrap().exists());
    assert!(!project_path.parent().unwrap().parent().unwrap().exists());
}

#[test]
fn remove_project_does_not_remove_parent_subdirs_if_not_empty() {
    let temp_dir = tempdir().unwrap();
    let temp_dir = temp_dir.path().to_path_buf();
    let test_config = make_config(None, None, None, Some(temp_dir));
    let project1_name = "subdir1/subdir2/project1";
    let project2_name = "subdir1/project2";

    // Make sure the project's parent directory exists
    let namespace = utils::get_project_namespace(project1_name).unwrap();
    let data_dir = test_config.get_projects_dir("").unwrap();
    mkdirp(data_dir.join(&namespace)).unwrap();

    // Make sure the file exists
    let projects_dir = test_config.get_projects_dir("").unwrap();
    let project1_path = projects_dir.join(project1_name).with_extension("yml");
    edit::create_project(project1_name, &project1_path).unwrap();
    assert!(project1_path.is_file());
    let project2_path = projects_dir.join(project2_name).with_extension("yml");
    edit::create_project(project2_name, &project2_path).unwrap();
    assert!(project2_path.is_file());

    let result = remove_project(&test_config, project1_name, true);
    assert!(result.is_ok());
    assert!(!project1_path.exists());
    assert!(!project1_path.parent().unwrap().exists());
    assert!(project1_path.parent().unwrap().parent().unwrap().exists());
}

#[test]
fn remove_project_fails_if_project_does_not_exist() {
    let temp_dir = tempdir().unwrap();
    let temp_dir = temp_dir.path().to_path_buf();
    let test_config = make_config(None, None, None, Some(temp_dir));
    let project1_name = "project";

    let result = remove_project(&test_config, project1_name, true);
    assert!(result.is_err());
    assert!(matches!(
        result.err().unwrap().downcast_ref::<Error>().unwrap(),
        Error::ProjectDoesNotExist { project_name } if project_name == project1_name
    ));
}

#[test]
fn remove_project_project_name_cannot_be_empty() {
    let temp_dir = tempdir().unwrap();
    let temp_dir = temp_dir.path().to_path_buf();
    let test_config = make_config(None, None, None, Some(temp_dir));
    let project_name = "";

    let result = remove_project(&test_config, project_name, true);
    assert!(result.is_err());
    assert!(matches!(
        result.err().unwrap().downcast_ref::<Error>().unwrap(),
        Error::ProjectNameEmpty {}
    ));
}

#[test]
fn list_project_does_not_fail() {
    let temp_dir = tempdir().unwrap();
    let temp_dir = temp_dir.path().to_path_buf();
    let test_config = make_config(None, None, None, Some(temp_dir));
    let projects_dir = test_config.get_projects_dir("").unwrap();

    for n in 0..5 {
        let project_name = OsString::from(format!("project{}", n));

        edit::create_project(&project_name, projects_dir.join(&project_name)).unwrap();
    }

    list_projects(&test_config).unwrap();
}

#[test]
fn get_project_list_returns_projects_without_extensions() {
    let temp_dir = tempdir().unwrap();
    let temp_dir = temp_dir.path().to_path_buf();

    let mut expected_project_list = Vec::with_capacity(5);
    for n in 0..5 {
        let project_name = OsString::from(format!("project{}", n));

        edit::create_project(&project_name, temp_dir.join(&project_name)).unwrap();
        expected_project_list.push(project_name);
    }
    expected_project_list.sort();

    let mut project_list = list::get_projects(&temp_dir).unwrap();
    project_list.sort();

    assert_eq!(project_list, expected_project_list);
}

#[test]
fn list_shows_projects_in_subdirectories() {
    let temp_dir = tempdir().unwrap();
    let temp_dir = temp_dir.path().to_path_buf();

    let mut expected_project_list = Vec::with_capacity(4);
    for n in 0..2 {
        let project_name = OsString::from(format!("project{}", n));

        edit::create_project(&project_name, temp_dir.join(&project_name)).unwrap();
        expected_project_list.push(project_name);
    }
    for n in 2..4 {
        let project_name = OsString::from(format!("subdir1/project{}", n));
        mkdirp(temp_dir.join("subdir1")).unwrap();

        edit::create_project(&project_name, temp_dir.join(&project_name)).unwrap();
        expected_project_list.push(project_name);
    }
    for n in 4..6 {
        let project_name = OsString::from(format!("subdir2/project{}", n));
        mkdirp(temp_dir.join("subdir2")).unwrap();

        edit::create_project(&project_name, temp_dir.join(&project_name)).unwrap();
        expected_project_list.push(project_name);
    }
    expected_project_list.sort();

    let mut project_list = list::get_projects(&temp_dir).unwrap();
    project_list.sort();

    assert_eq!(project_list, expected_project_list);
}

#[test]
fn list_follows_symlinks() {
    let temp_dir = tempdir().unwrap();
    let temp_dir = temp_dir.path().to_path_buf();

    let mut expected_project_list = Vec::with_capacity(4);
    for n in 0..2 {
        let project_name = OsString::from(format!("project{}", n));

        edit::create_project(&project_name, temp_dir.join(&project_name)).unwrap();
        expected_project_list.push(project_name);
    }
    for n in 2..4 {
        let project_name = OsString::from(format!("subdir1/project{}", n));
        mkdirp(temp_dir.join("subdir1")).unwrap();

        edit::create_project(&project_name, temp_dir.join(&project_name)).unwrap();
        expected_project_list.push(project_name);
    }
    for n in 2..4 {
        let project_name = OsString::from(format!("subdir2/project{}", n));

        expected_project_list.push(project_name);
    }
    expected_project_list.sort();

    #[cfg(windows)]
    os::windows::fs::symlink_dir(temp_dir.join("subdir1"), temp_dir.join("subdir2")).unwrap();
    #[cfg(unix)]
    os::unix::fs::symlink(temp_dir.join("subdir1"), temp_dir.join("subdir2")).unwrap();
    assert!(temp_dir.join("subdir2").is_dir());

    let mut project_list = list::get_projects(&temp_dir).unwrap();
    project_list.sort();

    assert_eq!(project_list, expected_project_list);
}

#[test]
fn list_detects_symlink_loops() {
    let temp_dir = tempdir().unwrap();
    let temp_dir = temp_dir.path().to_path_buf();

    let mut expected_project_list = Vec::with_capacity(4);
    for n in 0..2 {
        let project_name = OsString::from(format!("project{}", n));

        edit::create_project(&project_name, temp_dir.join(&project_name)).unwrap();
        expected_project_list.push(project_name);
    }
    for n in 2..4 {
        let project_name = OsString::from(format!("subdir1/project{}", n));
        mkdirp(temp_dir.join("subdir1")).unwrap();

        edit::create_project(&project_name, temp_dir.join(&project_name)).unwrap();
        expected_project_list.push(project_name);
    }
    expected_project_list.sort();

    #[cfg(windows)]
    os::windows::fs::symlink_dir(&temp_dir, temp_dir.join("subdir2")).unwrap();
    #[cfg(unix)]
    os::unix::fs::symlink(&temp_dir, temp_dir.join("subdir2")).unwrap();
    assert!(temp_dir.join("subdir2").is_dir());

    let mut project_list = list::get_projects(&temp_dir).unwrap();
    project_list.sort();

    assert_eq!(project_list, expected_project_list);
}