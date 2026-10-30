// This file was made using https://github.com/Dadoum/Sideloader as a reference.
// I'm planning on redoing this later to better handle entitlements, extensions, etc, but it will do for now

use isideload_vfs::fs::File;
use plist::{Dictionary, Value, to_writer_binary};
use rootcause::prelude::*;
use std::{
    io::BufWriter,
    path::{Path, PathBuf},
};

use crate::SideloadError;

#[derive(Debug, Clone)]
pub struct Bundle {
    pub app_info: Dictionary,
    pub bundle_dir: PathBuf,

    app_extensions: Vec<Bundle>,
    frameworks: Vec<Bundle>,
    watch_apps: Vec<Bundle>,
    _libraries: Vec<String>,
}

impl Bundle {
    pub fn new(bundle_dir: PathBuf) -> Result<Self, Report> {
        let mut bundle_path = bundle_dir;
        // Remove trailing slash/backslash
        if let Some(path_str) = bundle_path.to_str()
            && (path_str.ends_with('/') || path_str.ends_with('\\'))
        {
            bundle_path = PathBuf::from(&path_str[..path_str.len() - 1]);
        }

        let info_plist_path = bundle_path.join("Info.plist");
        assert_bundle(
            isideload_vfs::fs::metadata(&info_plist_path).is_ok(),
            &format!("No Info.plist here: {}", info_plist_path.display()),
        )?;

        let plist_data = isideload_vfs::fs::read(&info_plist_path).context(
            SideloadError::InvalidBundle("Failed to read Info.plist".to_string()),
        )?;

        let app_info = plist::from_bytes(&plist_data).context(SideloadError::InvalidBundle(
            "Failed to parse Info.plist".to_string(),
        ))?;

        // Xcode 26 may embed a single-target watchOS application in PlugIns/.
        // Keep ordinary app extensions separate from WatchKit companion apps.
        let plug_ins_dir = bundle_path.join("PlugIns");
        let plug_in_bundles: Vec<Bundle> = if isideload_vfs::fs::metadata(&plug_ins_dir).is_ok() {
            isideload_vfs::fs::read_dir(&plug_ins_dir)
                .context(SideloadError::InvalidBundle(
                    "Failed to read PlugIns directory".to_string(),
                ))?
                .filter_map(|entry| entry.ok())
                .filter(|entry| {
                    entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false)
                        && isideload_vfs::fs::metadata(&entry.path().join("Info.plist")).is_ok()
                })
                .filter_map(|entry| Bundle::new(entry.path()).ok())
                .collect()
        } else {
            Vec::new()
        };
        let (mut watch_apps, app_extensions): (Vec<Bundle>, Vec<Bundle>) =
            plug_in_bundles.into_iter().partition(|bundle| bundle.is_watch_application());

        // Load frameworks from Frameworks directory
        let frameworks_dir = bundle_path.join("Frameworks");
        let frameworks = if isideload_vfs::fs::metadata(&frameworks_dir).is_ok() {
            isideload_vfs::fs::read_dir(&frameworks_dir)
                .context(SideloadError::InvalidBundle(
                    "Failed to read Frameworks directory".to_string(),
                ))?
                .filter_map(|entry| entry.ok())
                .filter(|entry| {
                    entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false)
                        && isideload_vfs::fs::metadata(&entry.path().join("Info.plist")).is_ok()
                })
                .filter_map(|entry| Bundle::new(entry.path()).ok())
                .collect()
        } else {
            Vec::new()
        };

        // Preserve compatibility with older WatchKit packages that use Watch/.
        let watch_dir = bundle_path.join("Watch");
        if isideload_vfs::fs::metadata(&watch_dir).is_ok() {
            for entry in isideload_vfs::fs::read_dir(&watch_dir)
                .context(SideloadError::InvalidBundle(
                    "Failed to read Watch directory".to_string(),
                ))?
            {
                let entry = entry.context(SideloadError::InvalidBundle(
                    "Failed to read Watch directory entry".to_string(),
                ))?;
                let path = entry.path();
                if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false)
                    && path.extension().is_some_and(|ext| ext == "app")
                    && isideload_vfs::fs::metadata(&path.join("Info.plist")).is_ok()
                {
                    if let Ok(bundle) = Bundle::new(path) {
                        watch_apps.push(bundle);
                    }
                }
            }
        }

        // Find all .dylib files in the bundle directory (recursive)
        let libraries = find_dylibs(&bundle_path, &bundle_path)?;

        Ok(Bundle {
            app_info,
            bundle_dir: bundle_path,
            app_extensions,
            frameworks,
            watch_apps,
            _libraries: libraries,
        })
    }

    pub fn set_bundle_identifier(&mut self, id: &str) {
        let old_id = self.bundle_identifier().and_then(|s| Some(s.to_string()));
        if let Some(Value::Array(bg_tasks)) =
            self.app_info.get_mut("BGTaskSchedulerPermittedIdentifiers")
        {
            for task in bg_tasks.iter_mut() {
                if let Value::String(task_id) = task {
                    if let Some(old_bundle_id) = &old_id {
                        if task_id.contains(old_bundle_id) {
                            *task_id = task_id.replace(old_bundle_id, id);
                        }
                    }
                }
            }
        }

        self.app_info.insert(
            "CFBundleIdentifier".to_string(),
            Value::String(id.to_string()),
        );
    }

    pub fn bundle_identifier(&self) -> Option<&str> {
        self.app_info
            .get("CFBundleIdentifier")
            .and_then(|v| v.as_string())
    }

    pub fn bundle_name(&self) -> Option<&str> {
        self.app_info
            .get("CFBundleName")
            .and_then(|v| v.as_string())
    }

    pub fn app_extensions(&self) -> &[Bundle] {
        &self.app_extensions
    }

    pub fn app_extensions_mut(&mut self) -> &mut [Bundle] {
        &mut self.app_extensions
    }

    pub fn frameworks(&self) -> &[Bundle] {
        &self.frameworks
    }

    pub fn frameworks_mut(&mut self) -> &mut [Bundle] {
        &mut self.frameworks
    }

    pub fn watch_apps(&self) -> &[Bundle] {
        &self.watch_apps
    }

    pub fn watch_apps_mut(&mut self) -> &mut [Bundle] {
        &mut self.watch_apps
    }

    fn is_watch_application(&self) -> bool {
        let has_watchkit_marker = self
            .app_info
            .get("WKApplication")
            .and_then(Value::as_boolean)
            .unwrap_or(false);
        let targets_watch_device = self
            .app_info
            .get("UIDeviceFamily")
            .and_then(Value::as_array)
            .is_some_and(|families| {
                families.iter().any(|family| family.as_unsigned_integer() == Some(4))
            });

        has_watchkit_marker && targets_watch_device
    }

    fn rewrite_child_bundle_id(
        child: &mut Bundle,
        main_app_bundle_id: &str,
        main_app_id_str: &str,
        kind: &str,
    ) -> Result<(), Report> {
        let id = child
            .bundle_identifier()
            .ok_or_else(|| report!("{} bundle is missing CFBundleIdentifier", kind))?
            .to_string();

        // Accept both an original IPA identifier and one already rewritten by a
        // previous packaging step. Never append the team suffix twice.
        let rewritten_id = if id.starts_with(main_app_id_str)
            && id.len() > main_app_id_str.len()
        {
            id
        } else if id.starts_with(main_app_bundle_id)
            && id.len() > main_app_bundle_id.len()
        {
            format!(
                "{}{}",
                main_app_id_str,
                &id[main_app_bundle_id.len()..]
            )
        } else {
            bail!(SideloadError::InvalidBundle(format!(
                "{} {} is not part of the main app bundle identifier: {}",
                kind,
                child.bundle_name().unwrap_or("Unknown"),
                id
            )));
        };

        child.set_bundle_identifier(&rewritten_id);
        child.rewrite_embedded_bundle_ids(main_app_bundle_id, main_app_id_str)?;
        Ok(())
    }

    pub fn rewrite_embedded_bundle_ids(
        &mut self,
        main_app_bundle_id: &str,
        main_app_id_str: &str,
    ) -> Result<(), Report> {
        for ext in self.app_extensions.iter_mut() {
            Self::rewrite_child_bundle_id(
                ext,
                main_app_bundle_id,
                main_app_id_str,
                "Extension",
            )?;
        }

        for watch_app in self.watch_apps.iter_mut() {
            if let Some(companion_id) = watch_app
                .app_info
                .get("WKCompanionAppBundleIdentifier")
                .and_then(Value::as_string)
                && companion_id != main_app_bundle_id
                && companion_id != main_app_id_str
            {
                bail!(SideloadError::InvalidBundle(format!(
                    "Watch app {} references companion bundle identifier {}, expected {}",
                    watch_app.bundle_name().unwrap_or("Unknown"),
                    companion_id,
                    main_app_bundle_id
                )));
            }

            watch_app.app_info.insert(
                "WKCompanionAppBundleIdentifier".to_string(),
                Value::String(main_app_id_str.to_string()),
            );

            Self::rewrite_child_bundle_id(
                watch_app,
                main_app_bundle_id,
                main_app_id_str,
                "Watch app",
            )?;
        }

        Ok(())
    }

    pub fn collect_app_id_bundles_into<'a>(&'a self, bundles: &mut Vec<&'a Bundle>) {
        for bundle in &self.app_extensions {
            bundles.push(bundle);
            bundle.collect_app_id_bundles_into(bundles);
        }
        for bundle in &self.watch_apps {
            bundles.push(bundle);
            bundle.collect_app_id_bundles_into(bundles);
        }
    }

    pub fn collect_app_id_bundles(&self) -> Vec<&Bundle> {
        let mut bundles = vec![self];
        self.collect_app_id_bundles_into(&mut bundles);
        bundles
    }

    pub fn write_info_recursive(&self) -> Result<(), Report> {
        self.write_info()?;
        for bundle in &self.app_extensions {
            bundle.write_info_recursive()?;
        }
        for bundle in &self.frameworks {
            bundle.write_info_recursive()?;
        }
        for bundle in &self.watch_apps {
            bundle.write_info_recursive()?;
        }
        Ok(())
    }

    pub fn write_info(&self) -> Result<(), Report> {
        let info_plist_path = self.bundle_dir.join("Info.plist");
        // plist::to_file_binary(&info_plist_path, &self.app_info).context(
        //     SideloadError::InvalidBundle("Failed to write Info.plist".to_string()),
        // )?;
        let mut file = File::create(&info_plist_path).context(SideloadError::InvalidBundle(
            "Failed to write Info.plist".to_string(),
        ))?;
        to_writer_binary(BufWriter::new(&mut file), &self.app_info).context(
            SideloadError::InvalidBundle("Failed to create Info.plist writer".to_string()),
        )?;

        file.sync_all().context(SideloadError::InvalidBundle(
            "Failed to sync Info.plist".to_string(),
        ))?;
        Ok(())
    }

    fn from_dylib_path(dylib_path: PathBuf) -> Self {
        Self {
            app_info: Dictionary::new(),
            bundle_dir: dylib_path,
            app_extensions: Vec::new(),
            frameworks: Vec::new(),
            watch_apps: Vec::new(),
            _libraries: Vec::new(),
        }
    }

    fn collect_dylib_bundles(&self) -> Vec<Bundle> {
        self._libraries
            .iter()
            .map(|relative| Self::from_dylib_path(self.bundle_dir.join(relative)))
            .collect()
    }

    fn collect_nested_bundles_into(&self, bundles: &mut Vec<Bundle>) {
        for bundle in &self.app_extensions {
            bundles.push(bundle.clone());
            bundle.collect_nested_bundles_into(bundles);
        }

        for bundle in &self.frameworks {
            bundles.push(bundle.clone());
            bundle.collect_nested_bundles_into(bundles);
        }
        for bundle in &self.watch_apps {
            bundles.push(bundle.clone());
            bundle.collect_nested_bundles_into(bundles);
        }
    }

    pub fn collect_nested_bundles(&self) -> Vec<Bundle> {
        let mut bundles = Vec::new();
        self.collect_nested_bundles_into(&mut bundles);
        bundles.extend(self.collect_dylib_bundles());
        bundles
    }

    pub fn collect_bundles_sorted(&self) -> Vec<Bundle> {
        let mut bundles = self.collect_nested_bundles();
        bundles.push(self.clone());
        bundles.sort_by_key(|b| b.bundle_dir.components().count());
        bundles.reverse();
        bundles
    }
}

fn assert_bundle(condition: bool, msg: &str) -> Result<(), Report> {
    if !condition {
        bail!(SideloadError::InvalidBundle(msg.to_string()))
    } else {
        Ok(())
    }
}

fn find_dylibs(dir: &Path, bundle_root: &Path) -> Result<Vec<String>, Report> {
    let mut libraries = Vec::new();

    fn collect_dylibs(
        dir: &Path,
        bundle_root: &Path,
        libraries: &mut Vec<String>,
    ) -> Result<(), Report> {
        let entries = isideload_vfs::fs::read_dir(dir).context(SideloadError::InvalidBundle(
            format!("Failed to read directory {}", dir.display()),
        ))?;

        for entry in entries {
            let entry = entry.context(SideloadError::InvalidBundle(
                "Failed to read directory entry".to_string(),
            ))?;

            let path = entry.path();
            let file_type = isideload_vfs::fs::metadata(&path)
                .map(|m| m.file_type())
                .context(SideloadError::InvalidBundle(
                    "Failed to get file type".to_string(),
                ))?;

            if file_type.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str())
                    && name.ends_with(".dylib")
                {
                    // Get relative path from bundle root
                    if let Ok(relative_path) = path.strip_prefix(bundle_root)
                        && let Some(relative_str) = relative_path.to_str()
                    {
                        libraries.push(relative_str.to_string());
                    }
                }
            } else if file_type.is_dir() {
                collect_dylibs(&path, bundle_root, libraries)?;
            }
        }
        Ok(())
    }

    collect_dylibs(dir, bundle_root, &mut libraries)?;
    Ok(libraries)
}


#[cfg(test)]
mod watch_tests {
    use super::*;

    fn test_bundle(id: &str, name: &str) -> Bundle {
        let mut app_info = Dictionary::new();
        app_info.insert(
            "CFBundleIdentifier".to_string(),
            Value::String(id.to_string()),
        );
        app_info.insert(
            "CFBundleName".to_string(),
            Value::String(name.to_string()),
        );

        Bundle {
            app_info,
            bundle_dir: PathBuf::from(name),
            app_extensions: Vec::new(),
            frameworks: Vec::new(),
            watch_apps: Vec::new(),
            _libraries: Vec::new(),
        }
    }

    fn write_info_plist(path: &Path, info: &Dictionary) {
        let file = std::fs::File::create(path).unwrap();
        plist::to_writer_binary(file, info).unwrap();
    }

    #[test]
    fn detects_modern_watch_application_marker_and_device_family() {
        let mut watch = test_bundle("com.example.app.watchkitapp", "Watch");
        watch.app_info.insert("WKApplication".to_string(), Value::Boolean(true));
        watch.app_info.insert(
            "UIDeviceFamily".to_string(),
            Value::Array(vec![Value::Integer(4.into())]),
        );
        assert!(watch.is_watch_application());

        let extension = test_bundle("com.example.app.extension", "Extension");
        assert!(!extension.is_watch_application());
    }

    #[test]
    fn discovers_xcode26_watch_companion_inside_plugins() {
        let root = std::env::temp_dir().join(format!(
            "isideload-watch-package-{}",
            uuid::Uuid::new_v4()
        ));
        let host = root.join("NOVACompanion.app");
        let watch = host.join("PlugIns").join("NOVAWatch.app");
        let extension = host.join("PlugIns").join("ShareExtension.appex");

        std::fs::create_dir_all(&watch).unwrap();
        std::fs::create_dir_all(&extension).unwrap();

        let mut host_info = Dictionary::new();
        host_info.insert(
            "CFBundleIdentifier".to_string(),
            Value::String("com.example.app".to_string()),
        );
        host_info.insert(
            "CFBundleName".to_string(),
            Value::String("NOVA Companion".to_string()),
        );
        write_info_plist(&host.join("Info.plist"), &host_info);

        let mut watch_info = Dictionary::new();
        watch_info.insert(
            "CFBundleIdentifier".to_string(),
            Value::String("com.example.app.watchkitapp".to_string()),
        );
        watch_info.insert(
            "CFBundleName".to_string(),
            Value::String("NOVA Watch".to_string()),
        );
        watch_info.insert("WKApplication".to_string(), Value::Boolean(true));
        watch_info.insert(
            "WKCompanionAppBundleIdentifier".to_string(),
            Value::String("com.example.app".to_string()),
        );
        watch_info.insert(
            "UIDeviceFamily".to_string(),
            Value::Array(vec![Value::Integer(4.into())]),
        );
        write_info_plist(&watch.join("Info.plist"), &watch_info);

        let mut extension_info = Dictionary::new();
        extension_info.insert(
            "CFBundleIdentifier".to_string(),
            Value::String("com.example.app.share".to_string()),
        );
        extension_info.insert(
            "CFBundleName".to_string(),
            Value::String("Share Extension".to_string()),
        );
        write_info_plist(&extension.join("Info.plist"), &extension_info);

        let bundle = Bundle::new(host).unwrap();

        assert_eq!(bundle.watch_apps().len(), 1);
        assert_eq!(
            bundle.watch_apps()[0].bundle_identifier(),
            Some("com.example.app.watchkitapp")
        );
        assert_eq!(bundle.app_extensions().len(), 1);
        assert_eq!(
            bundle.app_extensions()[0].bundle_identifier(),
            Some("com.example.app.share")
        );

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rewrites_watch_bundle_and_companion_identifier_once() {
        let mut root = test_bundle("com.example.app", "Main");
        let mut watch = test_bundle("com.example.app.TEAM.watchkitapp", "Watch");
        watch.app_info.insert(
            "WKCompanionAppBundleIdentifier".to_string(),
            Value::String("com.example.app.TEAM".to_string()),
        );
        root.watch_apps.push(watch);

        root.rewrite_embedded_bundle_ids("com.example.app", "com.example.app.TEAM")
            .unwrap();
        root.set_bundle_identifier("com.example.app.TEAM");

        assert_eq!(
            root.watch_apps[0].bundle_identifier(),
            Some("com.example.app.TEAM.watchkitapp")
        );
        assert_eq!(
            root.watch_apps[0]
                .app_info
                .get("WKCompanionAppBundleIdentifier")
                .and_then(Value::as_string),
            Some("com.example.app.TEAM")
        );
    }
}
