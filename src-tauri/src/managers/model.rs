use crate::settings::{get_settings, write_settings};
use anyhow::Result;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager};

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub enum EngineType {
    /// Any GGML/GGUF model loaded through transcribe-cpp (Whisper, Parakeet,
    /// Voxtral, Qwen3-ASR, Nemotron, …). The architecture is auto-detected from
    /// the file, so this one variant covers the whole transcribe-cpp family.
    TranscribeCpp,
    Parakeet,
    Moonshine,
    MoonshineStreaming,
    SenseVoice,
    GigaAM,
    Canary,
    Cohere,
}

/// A model that ships inside the application bundle and is read in place from
/// the install directory. Bundled models are never copied into app data and can
/// never be deleted — the app is unusable without them.
pub struct BundledModel {
    pub id: &'static str,
    /// Directory name under `resources/models/`, also used as
    /// `ModelInfo::filename`.
    pub dir_name: &'static str,
    pub is_directory: bool,
}

/// Every model shipped in the bundle. A table rather than a single constant so
/// adding a second bundled model later needs no refactor.
pub const BUNDLED_MODELS: &[BundledModel] = &[BundledModel {
    id: "gigaam-v3-e2e-ctc",
    dir_name: "giga-am-v3-int8",
    is_directory: true,
}];

/// Whether `model_id` names a model that ships in the bundle. Used to refuse
/// deletion and to normalize a stale persisted selection.
pub fn is_bundled_model_id(model_id: &str) -> bool {
    BUNDLED_MODELS.iter().any(|entry| entry.id == model_id)
}

/// Scan a models resource directory for bundled models that are actually
/// present. A missing entry is not an error — the app just reports the model as
/// not installed.
///
/// Pure so it is unit-testable without an `AppHandle`; `ModelManager` supplies
/// the real resource root.
pub fn discover_bundled_models(models_root: &Path) -> HashMap<String, PathBuf> {
    let mut found = HashMap::new();
    for entry in BUNDLED_MODELS {
        let path = models_root.join(entry.dir_name);
        let present = if entry.is_directory {
            path.is_dir()
        } else {
            path.is_file()
        };
        if present {
            info!("Bundled model '{}' found at {:?}", entry.id, path);
            found.insert(entry.id.to_string(), path);
        } else {
            warn!(
                "Bundled model '{}' not found at {:?}; it will be reported as not installed",
                entry.id, path
            );
        }
    }
    found
}

/// Resolve the on-disk directory a bundled model should be loaded from.
///
/// PRIORITY — do not reorder: a user-provided copy in the app-data models
/// directory wins over the bundled copy in the install directory. Some users
/// run custom GigaAM builds by dropping `giga-am-v3-int8` into their app-data
/// folder; inverting this order would silently ignore their model. The bundled
/// path is only the fallback for a clean install.
pub fn resolve_model_path(
    model_id: &str,
    models_dir: &Path,
    bundled_paths: &HashMap<String, PathBuf>,
) -> Option<PathBuf> {
    let entry = BUNDLED_MODELS.iter().find(|entry| entry.id == model_id)?;

    let user_copy = models_dir.join(entry.dir_name);
    if user_copy.exists() {
        return Some(user_copy);
    }

    bundled_paths.get(model_id).cloned()
}

/// Where a model's files come from.
///
/// This build has no download path, so a model is either shipped in the bundle
/// (read in place from the install directory) or a user-supplied custom file in
/// the app-data models directory.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub enum ModelSource {
    /// Shipped inside the bundle as `resources/models/<dir_name>`.
    Bundled { dir_name: String },
    /// Already present on disk — a user-provided custom model. Nothing to
    /// download.
    Local,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub filename: String,
    pub source: ModelSource,
    pub size_mb: u64,
    pub is_downloaded: bool,
    pub is_downloading: bool,
    pub partial_size: u64,
    pub is_directory: bool,
    pub engine_type: EngineType,
    pub accuracy_score: f32,        // 0.0 to 1.0, higher is more accurate
    pub speed_score: f32,           // 0.0 to 1.0, higher is faster
    pub supports_translation: bool, // Whether the model supports translating to English
    pub is_recommended: bool,       // Whether this is the recommended model for new users
    pub supported_languages: Vec<String>, // Languages this model can transcribe
    pub supports_language_selection: bool, // Whether the user can explicitly pick a language
    pub is_custom: bool,            // Whether this is a user-provided custom model
    pub supports_streaming: bool, // Whether this model supports live streaming preview (transcribe-cpp)
    pub supports_language_detection: bool, // Whether the model can auto-detect language (gates the "Auto" option)
}

const CHINESE_LANGUAGE_CODE: &str = "zh";

fn recognition_language(language: &str) -> &str {
    match language {
        "zh-Hans" | "zh-Hant" => CHINESE_LANGUAGE_CODE,
        other => other,
    }
}

/// A tag's primary language subtag, with any BCP-47 region/script suffix
/// dropped (`en-US` → `en`, `zh-Hant` → `zh`).
fn base_language(language: &str) -> &str {
    language.split(&['-', '_'][..]).next().unwrap_or(language)
}

/// The stable user intent used to compare language codes across model
/// families. Norwegian Bokmål (`nb`) maps to Norwegian (`no`), while Nynorsk
/// (`nn`) remains distinct; Filipino (`fil`) maps to Tagalog (`tl`). This is
/// only for matching: callers must return/pass the model's original code.
pub(crate) fn canonical_language_code(language: &str) -> &str {
    match base_language(language) {
        "nb" => "no",
        "fil" => "tl",
        base => base,
    }
}

fn canonicalize_supported_languages(languages: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut canonical = Vec::with_capacity(languages.len());

    for language in languages {
        let language = recognition_language(&language).to_string();
        if seen.insert(language.clone()) {
            canonical.push(language);
        }
    }

    canonical
}

/// Resolve the user's persisted language *intent* (`"auto"` or a language code)
/// into the language a given model will actually use.
///
/// The canonical coercion used on every transcription path: computed at the
/// point of use and **never written back** to settings, so the user's last
/// explicit intent survives switching to an incompatible model and back.
///
/// Matching is base-aware ([`base_language`]) and returns the model's own
/// *concrete* code, so a bare intent (`en`) resolves to the exact string the
/// engine's prompt table expects (`en-US`) for models that advertise full
/// BCP-47 locales. Chinese *script* intents (`zh-Hans`/`zh-Hant`) are the sole
/// exception: they pass through unchanged so the downstream Simplified /
/// Traditional output conversion still fires (the engine path collapses them to
/// a plain Chinese code separately).
pub fn effective_language(
    intent: &str,
    supported_languages: &[String],
    supports_language_detection: bool,
) -> String {
    if supported_languages.is_empty() {
        return intent.to_string();
    }

    if intent != "auto" {
        // Prefer the same base code before considering an equivalence alias. If
        // a future model advertises both `no` and `nb`, an explicit `nb` intent
        // must select `nb`, regardless of capability-list order.
        let exact_base_match = supported_languages
            .iter()
            .find(|language| base_language(language) == base_language(intent));
        let equivalent_match = || {
            supported_languages.iter().find(|language| {
                canonical_language_code(language) == canonical_language_code(intent)
            })
        };

        if let Some(code) = exact_base_match.or_else(equivalent_match) {
            if intent == "zh-Hans" || intent == "zh-Hant" {
                return intent.to_string();
            }
            return code.clone();
        }
    }

    if supports_language_detection {
        return "auto".to_string();
    }

    // Model can't auto-detect and the intent isn't usable: fall back to a
    // concrete language (prefer English) so we never hand the engine "auto".
    if let Some(en) = supported_languages
        .iter()
        .find(|language| base_language(language) == "en")
    {
        return en.clone();
    }
    recognition_language(&supported_languages[0]).to_string()
}

/// RAII guard that clears the `is_rescanning` single-flight flag on drop, so the
/// slot is released on every exit path (including early returns and `?`).
struct RescanGuard {
    flag: Arc<AtomicBool>,
}

impl Drop for RescanGuard {
    fn drop(&mut self) {
        self.flag.store(false, Ordering::SeqCst);
    }
}

pub struct ModelManager {
    app_handle: AppHandle,
    models_dir: PathBuf,
    available_models: Mutex<HashMap<String, ModelInfo>>,
    /// Resolved install-directory paths of bundled models, keyed by model id.
    /// Only models actually present in the bundle appear here.
    bundled_paths: Mutex<HashMap<String, PathBuf>>,
    /// Single-flight guard for [`Self::rescan_local_models`] so concurrent
    /// refresh requests coalesce instead of scanning the disk in parallel.
    is_rescanning: Arc<AtomicBool>,
}

impl ModelManager {
    pub fn new(app_handle: &AppHandle) -> Result<Self> {
        // Create models directory in app data
        let models_dir = crate::portable::app_data_dir(app_handle)
            .map_err(|e| anyhow::anyhow!("Failed to get app data dir: {}", e))?
            .join("models");

        if !models_dir.exists() {
            fs::create_dir_all(&models_dir)?;
        }

        let mut available_models = HashMap::new();

        // The bundled GigaAM v3 entry. `size_mb` stays informative: it is the
        // on-disk footprint of the shipped files, not a download size.
        available_models.insert(
            "gigaam-v3-e2e-ctc".to_string(),
            ModelInfo {
                id: "gigaam-v3-e2e-ctc".to_string(),
                name: "GigaAM v3".to_string(),
                description: "Russian speech recognition. Fast and accurate.".to_string(),
                filename: "giga-am-v3-int8".to_string(),
                source: ModelSource::Bundled {
                    dir_name: "giga-am-v3-int8".to_string(),
                },
                size_mb: 151,
                is_downloaded: false,
                is_downloading: false,
                partial_size: 0,
                is_directory: true,
                engine_type: EngineType::GigaAM,
                accuracy_score: 0.85,
                speed_score: 0.75,
                supports_translation: false,
                is_recommended: true,
                supported_languages: vec!["ru".to_string()],
                supports_language_selection: false,
                is_custom: false,
                supports_streaming: false,
                supports_language_detection: true,
            },
        );

        let manager = Self {
            app_handle: app_handle.clone(),
            models_dir,
            available_models: Mutex::new(available_models),
            bundled_paths: Mutex::new(HashMap::new()),
            is_rescanning: Arc::new(AtomicBool::new(false)),
        };

        // Locate models shipped inside the bundle. Runs before any status check
        // so a bundled model is already known as installed when first queried.
        manager.resolve_bundled_models();

        // Check which models are already installed
        manager.update_download_status()?;

        // Auto-select a model if none is currently selected
        manager.auto_select_model_if_needed()?;

        Ok(manager)
    }

    pub fn get_available_models(&self) -> Vec<ModelInfo> {
        let mut list: Vec<ModelInfo> = {
            let models = self.available_models.lock().unwrap();
            models.values().cloned().collect()
        };
        // Stable, reasonable order: recommended model first, then by accuracy,
        // speed, and name.
        list.sort_by(|a, b| {
            (!a.is_recommended)
                .cmp(&(!b.is_recommended))
                .then(b.accuracy_score.total_cmp(&a.accuracy_score))
                .then(b.speed_score.total_cmp(&a.speed_score))
                .then_with(|| a.name.cmp(&b.name))
        });
        list
    }

    /// Claim the single rescan slot. Returns a guard that releases it on drop,
    /// or `None` if a rescan is already running (callers should just skip).
    fn try_start_rescan(&self) -> Option<RescanGuard> {
        if self.is_rescanning.swap(true, Ordering::SeqCst) {
            None
        } else {
            Some(RescanGuard {
                flag: self.is_rescanning.clone(),
            })
        }
    }

    /// Re-run the local discovery scans (custom models dir + shared HF cache) so
    /// models dropped in or downloaded outside Handy show up without a restart.
    /// The merge is additive: only new ids are inserted, so existing entries keep
    /// their values — including runtime-probed capabilities from
    /// [`Self::set_runtime_capabilities`]. It then runs [`Self::update_download_status`],
    /// which recomputes disk-derived flags for *every* entry; a rescan racing an
    /// in-flight download can briefly clear its `is_downloading`, but the download
    /// continues and the event-driven UI self-corrects.
    ///
    /// The disk walk and 64 KiB header probes run against a cloned snapshot
    /// *off-lock* so readers never block on I/O; only the brief merge takes the
    /// registry lock. Concurrent calls coalesce via [`Self::try_start_rescan`].
    pub fn rescan_local_models(&self) -> Result<()> {
        let _guard = match self.try_start_rescan() {
            Some(g) => g,
            None => {
                debug!("Model rescan already in progress; skipping");
                return Ok(());
            }
        };

        // Bundled models live in the install directory, which changes when the
        // app is updated in place, so re-resolve them on every rescan.
        self.resolve_bundled_models();

        self.update_download_status()?;
        self.auto_select_model_if_needed()?;
        let _ = self.app_handle.emit("models-updated", ());
        Ok(())
    }

    pub fn get_model_info(&self, model_id: &str) -> Option<ModelInfo> {
        let models = self.available_models.lock().unwrap();
        models.get(model_id).cloned()
    }

    /// Reconcile a model's advertised capabilities with the ground truth from the
    /// loaded model (transcribe-cpp's GGUF-derived capabilities), overwriting the
    /// pre-download view (catalog metadata or a header probe — see
    /// [`super::model_capabilities`]).
    ///
    /// This corrects the header probe's gaps. It matters most for **streaming**
    /// (transcribe-cpp infers it at load for parakeet/streaming families, where
    /// the flat GGUF key can be absent, and it gates whether streaming is even
    /// attempted — see `actions.rs`) and for **language detection** / the
    /// **supported-language set**, which feed [`effective_language`]; a mislabeled
    /// header would otherwise coerce an "auto" intent to a forced language for good.
    /// Translate is reconciled too for badge accuracy, though run paths re-read it
    /// live regardless.
    pub fn set_runtime_capabilities(
        &self,
        model_id: &str,
        supports_streaming: bool,
        supports_translation: bool,
        supports_language_detection: bool,
        supported_languages: Vec<String>,
    ) {
        let supported_languages = canonicalize_supported_languages(supported_languages);
        let mut models = self.available_models.lock().unwrap();
        if let Some(model) = models.get_mut(model_id) {
            model.supports_streaming = supports_streaming;
            model.supports_translation = supports_translation;
            model.supports_language_detection = supports_language_detection;
            // An empty set means the model is language-agnostic — but it is also
            // what a failed capability read leaves behind, so keep the probed /
            // catalog list rather than blanking a known one to nothing.
            if !supported_languages.is_empty() {
                model.supports_language_selection = supported_languages.len() > 1;
                model.supported_languages = supported_languages;
            }
        }
    }

    /// Refresh [`Self::bundled_paths`] from the bundle's models resource
    /// directory. Idempotent, and safe to call on every rescan.
    fn resolve_bundled_models(&self) {
        let resolved = match self.app_handle.path().resolve(
            "resources/models",
            tauri::path::BaseDirectory::Resource,
        ) {
            Ok(root) => discover_bundled_models(&root),
            Err(e) => {
                warn!("Could not resolve the models resource directory: {}", e);
                HashMap::new()
            }
        };
        *self.bundled_paths.lock().unwrap() = resolved;
    }

    fn update_download_status(&self) -> Result<()> {
        // Snapshot in-flight download ids before taking the registry lock (the
        // two locks are never nested) so a mid-download entry is never dropped.
        let bundled: HashSet<String> = self.bundled_paths.lock().unwrap().keys().cloned().collect();
        let mut models = self.available_models.lock().unwrap();

        for model in models.values_mut() {
            if bundled.contains(&model.id) {
                // A bundled model is always complete: it shipped inside the
                // bundle and there is nothing to download or extract.
                model.is_downloaded = true;
                model.is_downloading = false;
                model.partial_size = 0;
                continue;
            }

            if model.is_directory {
                // For directory-based models, check if the directory exists
                let model_path = self.models_dir.join(&model.filename);
                let partial_path = self.models_dir.join(format!("{}.partial", &model.filename));

                model.is_downloaded = model_path.exists() && model_path.is_dir();
                model.is_downloading = false;

                // Get partial file size if it exists (for the .tar.gz being downloaded)
                if partial_path.exists() {
                    model.partial_size = partial_path.metadata().map(|m| m.len()).unwrap_or(0);
                } else {
                    model.partial_size = 0;
                }
            } else {
                // For file-based models (existing logic)
                let model_path = self.models_dir.join(&model.filename);
                let partial_path = self.models_dir.join(format!("{}.partial", &model.filename));

                model.is_downloaded = model_path.exists();
                model.is_downloading = false;

                // Get partial file size if it exists
                if partial_path.exists() {
                    model.partial_size = partial_path.metadata().map(|m| m.len()).unwrap_or(0);
                } else {
                    model.partial_size = 0;
                }
            }
        }

        Ok(())
    }

    fn selected_model_is_available(models: &HashMap<String, ModelInfo>, model_id: &str) -> bool {
        models
            .get(model_id)
            .is_some_and(|model| model.is_downloaded)
    }

    fn auto_select_model_if_needed(&self) -> Result<()> {
        let mut settings = get_settings(&self.app_handle);

        // A model cannot remain selected after its files disappear. Catalog
        // entries stay in the registry so they can be downloaded again, so
        // checking only whether the id exists is not sufficient.
        if !settings.selected_model.is_empty() {
            let is_available = {
                let models = self.available_models.lock().unwrap();
                Self::selected_model_is_available(&models, &settings.selected_model)
            };

            if !is_available {
                info!(
                    "Selected model '{}' is not available on disk; clearing selection",
                    settings.selected_model
                );
                settings.selected_model = String::new();
                write_settings(&self.app_handle, settings.clone());
            }
        }

        // If onboarding is still pending, do not auto-select just because a
        // compatible model exists on disk or in the shared HF cache. The
        // onboarding model step should present that choice explicitly.
        if !settings.onboarding_completed {
            debug!("Skipping model auto-selection until onboarding is complete");
            return Ok(());
        }

        // If no model is selected, pick the first downloaded one using the same
        // ranked order the UI receives.
        if settings.selected_model.is_empty() {
            if let Some(available_model) = self
                .get_available_models()
                .into_iter()
                .find(|model| model.is_downloaded)
            {
                info!(
                    "Auto-selecting model: {} ({})",
                    available_model.id, available_model.name
                );

                // Update settings with the selected model
                let mut updated_settings = settings;
                updated_settings.selected_model = available_model.id.clone();
                write_settings(&self.app_handle, updated_settings);

                info!("Successfully auto-selected model: {}", available_model.id);
            }
        }

        Ok(())
    }

    pub fn delete_model(&self, model_id: &str) -> Result<()> {
        debug!("ModelManager: delete_model called for: {}", model_id);

        let model_info = {
            let models = self.available_models.lock().unwrap();
            models.get(model_id).cloned()
        };

        let model_info =
            model_info.ok_or_else(|| anyhow::anyhow!("Model not found: {}", model_id))?;

        if is_bundled_model_id(model_id) {
            return Err(anyhow::anyhow!(
                "Built-in model '{}' cannot be deleted",
                model_id
            ));
        }

        debug!("ModelManager: Found model info: {:?}", model_info);

        let model_path = self.models_dir.join(&model_info.filename);
        let partial_path = self
            .models_dir
            .join(format!("{}.partial", &model_info.filename));
        debug!("ModelManager: Model path: {:?}", model_path);
        debug!("ModelManager: Partial path: {:?}", partial_path);

        let mut deleted_something = false;

        if model_info.is_directory {
            // Delete complete model directory if it exists
            if model_path.exists() && model_path.is_dir() {
                info!("Deleting model directory at: {:?}", model_path);
                fs::remove_dir_all(&model_path)?;
                info!("Model directory deleted successfully");
                deleted_something = true;
            }
        } else {
            // Delete complete model file if it exists
            if model_path.exists() {
                info!("Deleting model file at: {:?}", model_path);
                fs::remove_file(&model_path)?;
                info!("Model file deleted successfully");
                deleted_something = true;
            }
        }

        // Delete partial file if it exists (same for both types)
        if partial_path.exists() {
            info!("Deleting partial file at: {:?}", partial_path);
            fs::remove_file(&partial_path)?;
            info!("Partial file deleted successfully");
            deleted_something = true;
        }

        // Files already missing (e.g. removed outside Handy) is not a failure —
        // deleting is idempotent, so this still needs to fall through and clear
        // the stale "Downloaded" entry rather than erroring out and leaving it stuck.
        if !deleted_something {
            debug!(
                "ModelManager: no files found on disk for {}; clearing stale entry",
                model_id
            );
        }

        // Custom models should be removed from the list entirely since they
        // have no download URL and can't be re-downloaded
        if model_info.is_custom {
            let mut models = self.available_models.lock().unwrap();
            models.remove(model_id);
            debug!("ModelManager: removed custom model from available models");
        } else {
            // Update download status (marks predefined models as not downloaded)
            self.update_download_status()?;
            debug!("ModelManager: download status updated");
        }

        // Emit event to notify UI
        let _ = self.app_handle.emit("model-deleted", model_id);

        Ok(())
    }

    /// Reconcile a model that was advertised as installed but whose path has
    /// since disappeared. Only bundled models survive in the registry, and their
    /// path is resolved from the install directory on every status update, so
    /// there is no stale registry state to reconcile here.
    fn mark_model_unavailable(&self, model_id: &str) {
        {
            let mut models = self.available_models.lock().unwrap();
            if let Some(model) = models.get_mut(model_id) {
                model.is_downloaded = false;
            } else {
                return;
            }
        }

        info!("Marking model '{}' as unavailable on disk", model_id);
        // Keep the persisted preference here: an already-loaded engine may
        // still be usable.
        let _ = self.app_handle.emit("models-updated", ());
    }

    pub fn get_model_path(&self, model_id: &str) -> Result<PathBuf> {
        let model_info = self
            .get_model_info(model_id)
            .ok_or_else(|| anyhow::anyhow!("Model not found: {}", model_id))?;

        if !model_info.is_downloaded {
            return Err(anyhow::anyhow!("Model not available: {}", model_id));
        }

        // Ensure we don't return partial files/directories
        if model_info.is_downloading {
            return Err(anyhow::anyhow!(
                "Model is currently downloading: {}",
                model_id
            ));
        }

        // A bundled model is read straight from the install directory, but a
        // user-supplied copy in app data takes priority — see
        // `resolve_model_path`.
        if let Some(path) = resolve_model_path(
            model_id,
            &self.models_dir,
            &self.bundled_paths.lock().unwrap(),
        ) {
            return Ok(path);
        }

        self.mark_model_unavailable(model_id);
        Err(anyhow::anyhow!(
            "Model files not found for '{}' in the install directory or the models folder",
            model_id
        ))
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn bundled_table_ships_gigaam_directory_layout() {
        let entry = BUNDLED_MODELS
            .iter()
            .find(|entry| entry.id == "gigaam-v3-e2e-ctc")
            .expect("GigaAM v3 must ship bundled");

        assert_eq!(entry.dir_name, "giga-am-v3-int8");
        assert!(entry.is_directory);
    }

    #[test]
    fn bundled_table_has_unique_ids_and_dir_names() {
        let mut ids: Vec<&str> = BUNDLED_MODELS.iter().map(|entry| entry.id).collect();
        let mut dirs: Vec<&str> = BUNDLED_MODELS
            .iter()
            .map(|entry| entry.dir_name)
            .collect();
        ids.sort_unstable();
        dirs.sort_unstable();
        let unique_ids = ids.len();
        let unique_dirs = dirs.len();
        ids.dedup();
        dirs.dedup();

        assert_eq!(ids.len(), unique_ids, "duplicate bundled model id");
        assert_eq!(dirs.len(), unique_dirs, "duplicate bundled dir_name");
    }

    #[test]
    fn is_bundled_model_id_recognizes_only_bundled_ids() {
        assert!(is_bundled_model_id("gigaam-v3-e2e-ctc"));
        assert!(!is_bundled_model_id("whisper-small"));
        assert!(!is_bundled_model_id(""));
    }

    #[test]
    fn discover_bundled_models_finds_existing_gigaam_directory() {
        let temp = TempDir::new().unwrap();
        let models_root = temp.path();
        fs::create_dir_all(models_root.join("giga-am-v3-int8")).unwrap();

        let found = discover_bundled_models(models_root);

        assert_eq!(
            found.get("gigaam-v3-e2e-ctc"),
            Some(&models_root.join("giga-am-v3-int8"))
        );
    }

    #[test]
    fn discover_bundled_models_skips_missing_entries() {
        let temp = TempDir::new().unwrap();

        assert!(discover_bundled_models(temp.path()).is_empty());
    }

    #[test]
    fn discover_bundled_models_ignores_wrong_kind_of_entry() {
        let temp = TempDir::new().unwrap();
        let models_root = temp.path();
        // A regular file where a directory is expected must not count as
        // installed — GigaAM would fail to load with a confusing engine error.
        fs::write(models_root.join("giga-am-v3-int8"), b"").unwrap();

        assert!(discover_bundled_models(models_root).is_empty());
    }

    #[test]
    fn resolve_model_path_prefers_user_copy_over_bundled() {
        let temp = TempDir::new().unwrap();
        let models_dir = temp.path().join("appdata-models");
        let bundled_dir = temp.path().join("install-resources");
        fs::create_dir_all(models_dir.join("giga-am-v3-int8")).unwrap();
        fs::create_dir_all(bundled_dir.join("giga-am-v3-int8")).unwrap();

        let mut bundled = std::collections::HashMap::new();
        bundled.insert(
            "gigaam-v3-e2e-ctc".to_string(),
            bundled_dir.join("giga-am-v3-int8"),
        );

        assert_eq!(
            resolve_model_path("gigaam-v3-e2e-ctc", &models_dir, &bundled),
            Some(models_dir.join("giga-am-v3-int8"))
        );
    }

    #[test]
    fn resolve_model_path_falls_back_to_bundled_when_no_user_copy() {
        let temp = TempDir::new().unwrap();
        let models_dir = temp.path().join("appdata-models");
        let bundled_dir = temp.path().join("install-resources");
        fs::create_dir_all(&models_dir).unwrap();
        fs::create_dir_all(bundled_dir.join("giga-am-v3-int8")).unwrap();

        let mut bundled = std::collections::HashMap::new();
        bundled.insert(
            "gigaam-v3-e2e-ctc".to_string(),
            bundled_dir.join("giga-am-v3-int8"),
        );

        assert_eq!(
            resolve_model_path("gigaam-v3-e2e-ctc", &models_dir, &bundled),
            Some(bundled_dir.join("giga-am-v3-int8"))
        );
    }

    #[test]
    fn resolve_model_path_returns_none_when_nothing_is_installed() {
        let temp = TempDir::new().unwrap();

        assert_eq!(
            resolve_model_path(
                "gigaam-v3-e2e-ctc",
                &temp.path().join("appdata-models"),
                &std::collections::HashMap::new()
            ),
            None
        );
    }

    #[test]
    fn test_effective_language_accepts_chinese_script_intent_for_zh_capability() {
        let languages = vec!["zh".to_string()];

        assert_eq!(effective_language("zh-Hans", &languages, false), "zh-Hans");
        assert_eq!(effective_language("zh-Hant", &languages, false), "zh-Hant");
    }

    #[test]
    fn test_effective_language_falls_back_to_canonical_chinese() {
        let languages = vec!["zh-Hant".to_string()];

        assert_eq!(effective_language("auto", &languages, false), "zh");
    }

    #[test]
    fn test_effective_language_resolves_bare_intent_to_concrete_locale() {
        // A model advertising full BCP-47 locales (e.g. Nemotron Streaming):
        // a bare intent must resolve to the exact code the engine expects, not
        // be handed back as the bare form the prompt table may not contain.
        let languages = vec![
            "en-US".to_string(),
            "en-GB".to_string(),
            "es-ES".to_string(),
            "zh-CN".to_string(),
            "ja-JP".to_string(),
        ];

        assert_eq!(effective_language("en", &languages, true), "en-US");
        assert_eq!(effective_language("es", &languages, true), "es-ES");
        // `zh`/`ja` have no bare entry in this model's table; resolve to locale.
        assert_eq!(effective_language("zh", &languages, true), "zh-CN");
        assert_eq!(effective_language("ja", &languages, true), "ja-JP");
        // An unsupported intent still auto-detects when the model can.
        assert_eq!(effective_language("fr", &languages, true), "auto");
    }

    #[test]
    fn test_effective_language_preserves_chinese_script_intent_for_locale_model() {
        // Script intents survive so Simplified/Traditional output conversion
        // still fires, even when the model advertises a regioned Chinese code.
        let languages = vec!["en-US".to_string(), "zh-CN".to_string()];

        assert_eq!(effective_language("zh-Hans", &languages, true), "zh-Hans");
        assert_eq!(effective_language("zh-Hant", &languages, true), "zh-Hant");
    }

    #[test]
    fn test_effective_language_preserves_intent_across_model_code_variants() {
        assert_eq!(effective_language("no", &["nb".to_string()], true), "nb");
        assert_eq!(effective_language("nb", &["no".to_string()], true), "no");
        assert_eq!(effective_language("tl", &["fil".to_string()], true), "fil");
        assert_eq!(effective_language("fil", &["tl".to_string()], true), "tl");
    }

    #[test]
    fn test_effective_language_prefers_exact_norwegian_code() {
        let languages = vec!["no".to_string(), "nb".to_string()];

        assert_eq!(effective_language("nb", &languages, true), "nb");
        assert_eq!(effective_language("no", &languages, true), "no");
    }

    #[test]
    fn test_canonicalize_supported_languages_collapses_chinese_scripts() {
        let languages = canonicalize_supported_languages(
            vec!["en", "zh", "zh-Hans", "zh-Hant", "yue"]
                .into_iter()
                .map(String::from)
                .collect(),
        );

        assert_eq!(languages, vec!["en", "zh", "yue"]);
    }

}
