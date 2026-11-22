use nazar_core::kernel_path::{VolumeName, VolumeNames, translate_registry_path};

fn volume_names() -> VolumeNames {
    VolumeNames::new(vec![
        VolumeName {
            device_path: "\\Device\\HarddiskVolume1".to_owned(),
            drive: "D:".to_owned(),
        },
        VolumeName {
            device_path: "\\Device\\HarddiskVolume10".to_owned(),
            drive: "C:".to_owned(),
        },
    ])
}

#[test]
fn device_path_becomes_a_drive_path() {
    let path = volume_names().translate_file_path("\\Device\\HarddiskVolume10\\Users\\me\\a.txt");

    assert_eq!(path, "C:\\Users\\me\\a.txt");
}

#[test]
fn volume_with_a_longer_number_is_not_mistaken_for_a_shorter_one() {
    let path = volume_names().translate_file_path("\\Device\\HarddiskVolume1\\data.bin");

    assert_eq!(path, "D:\\data.bin");
}

#[test]
fn device_path_is_matched_without_regard_to_case() {
    let path = volume_names().translate_file_path("\\device\\harddiskvolume1\\data.bin");

    assert_eq!(path, "D:\\data.bin");
}

#[test]
fn path_on_an_unknown_volume_is_left_unchanged() {
    let path = volume_names().translate_file_path("\\Device\\Mup\\server\\share\\a.txt");

    assert_eq!(path, "\\Device\\Mup\\server\\share\\a.txt");
}

#[test]
fn machine_and_user_registry_roots_get_their_familiar_names() {
    let machine_path = translate_registry_path("\\REGISTRY\\MACHINE\\SOFTWARE\\Vendor");
    let user_path = translate_registry_path("\\REGISTRY\\USER\\S-1-5-21-1\\Software\\Vendor");

    assert_eq!(machine_path, "HKLM\\SOFTWARE\\Vendor");
    assert_eq!(user_path, "HKU\\S-1-5-21-1\\Software\\Vendor");
}

#[test]
fn registry_path_outside_the_known_roots_is_left_unchanged() {
    let path = translate_registry_path("\\REGISTRY\\A\\{application-hive}\\Settings");

    assert_eq!(path, "\\REGISTRY\\A\\{application-hive}\\Settings");
}
