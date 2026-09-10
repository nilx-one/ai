// © 2026 aiaiaiai · aiaiaiai.org
// SPDX-License-Identifier: MPL-2.0

//! Which local models this product serves, and which one a measured device gets.
//!
//! Model selection is product work. The foundation's browser adapter loads the identifier
//! it is handed and takes no view on which one that should be, so the catalog — two
//! entries, both `Qwen3`, both `q4f16_1` — and the rule that reads a device against it
//! live here.
//!
//! What these models produce is a choice from a closed set the product already resolved,
//! not prose a person reads. That is why the catalog is sized the way it is: a model that
//! stays resident while an owner spectates, and that a device will lose to cache eviction
//! and take back, is paid for in held memory and re-fetched bytes rather than in fluency.
//!
//! Nothing in this module downloads, probes, or loads anything. A [`DeviceCapability`]
//! arrives already measured and a model identity leaves, or nothing does. `None` is a whole
//! answer rather than a prompt to try something smaller: this crate has no third entry to
//! degrade into, and a remote provider standing in for a local one is a transition the
//! product reports, never a value this function quietly returns.

use serde::Serialize;

/// The weight format an artifact set was compiled in.
///
/// It decides the one device fact this module gates on: whether `shader-f16` is required.
/// The pinned `WebLLM` registry leaves `required_features` off its newer entries, so a
/// device without f16 downloads an entire model before failing to initialize it. Reading
/// the requirement off the quantization is what lets the served `model_list` declare that
/// feature itself and refuse before the first byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Quantization {
    /// `q4f16_1`: 4-bit weights, half-precision scales. Requires `shader-f16`.
    #[serde(rename = "q4f16_1")]
    Q4f16,
    /// `q4f32_1`: 4-bit weights, single-precision scales. Runs without `shader-f16` at
    /// roughly 1.3× the VRAM. Present as vocabulary; nothing is served in this shape.
    #[serde(rename = "q4f32_1")]
    Q4f32,
}

impl Quantization {
    /// Returns whether a device must grant `shader-f16` for this format.
    #[must_use]
    pub const fn requires_shader_f16(self) -> bool {
        matches!(self, Self::Q4f16)
    }
}

/// One model this product is prepared to serve.
///
/// Only this module constructs one, so the derived `shader-f16` requirement cannot
/// disagree with the quantization it was read from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct LocalModel {
    /// The `model_id` of the served `model_list` entry, and the identifier the browser
    /// adapter is constructed with.
    id: &'static str,
    quantization: Quantization,
    /// The pinned registry's `vram_required_MB` for this entry, rounded up to whole MB.
    vram_required_mb: u32,
    /// The context window the entry is configured for, and the window the VRAM figure
    /// above was measured at. The two are one number: reading VRAM from one configuration
    /// and running another makes the budget check meaningless.
    context_window: u32,
    /// Derived from [`Self::quantization`] and carried rather than re-derived, because the
    /// host that writes `required_features` into its `model_list` would otherwise keep a
    /// second copy of this rule in another language.
    requires_shader_f16: bool,
}

impl LocalModel {
    const fn new(
        id: &'static str,
        quantization: Quantization,
        vram_required_mb: u32,
        context_window: u32,
    ) -> Self {
        Self {
            id,
            quantization,
            vram_required_mb,
            context_window,
            requires_shader_f16: quantization.requires_shader_f16(),
        }
    }

    /// Returns the `model_id` the host loads.
    #[must_use]
    pub const fn id(&self) -> &'static str {
        self.id
    }

    /// Returns the weight format the artifacts were compiled in.
    #[must_use]
    pub const fn quantization(&self) -> Quantization {
        self.quantization
    }

    /// Returns the VRAM the pinned registry states this entry needs.
    #[must_use]
    pub const fn vram_required_mb(&self) -> u32 {
        self.vram_required_mb
    }

    /// Returns the context window this entry is configured and measured for.
    #[must_use]
    pub const fn context_window(&self) -> u32 {
        self.context_window
    }

    /// Returns whether the device must grant `shader-f16` to run this entry.
    #[must_use]
    pub const fn requires_shader_f16(&self) -> bool {
        self.requires_shader_f16
    }

    /// Returns whether `device` grants everything this entry needs.
    #[must_use]
    pub const fn runs_on(&self, device: &DeviceCapability) -> bool {
        if self.requires_shader_f16 && !device.shader_f16 {
            return false;
        }
        self.vram_required_mb <= device.vram_budget_mb
    }
}

/// What a device actually granted, as the product measured it.
///
/// There is no default and no unmeasured constructor. A surface nobody probed is not a
/// permissive surface, and treating it as one is the single substitution the foundation's
/// probe results refuse by name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceCapability {
    shader_f16: bool,
    vram_budget_mb: u32,
}

impl DeviceCapability {
    /// Records what a probe reported on this device.
    #[must_use]
    pub const fn measured(shader_f16: bool, vram_budget_mb: u32) -> Self {
        Self {
            shader_f16,
            vram_budget_mb,
        }
    }

    /// Returns whether the device granted `shader-f16`.
    #[must_use]
    pub const fn shader_f16(&self) -> bool {
        self.shader_f16
    }

    /// Returns the VRAM the product is willing to spend on a model here.
    #[must_use]
    pub const fn vram_budget_mb(&self) -> u32 {
        self.vram_budget_mb
    }
}

/// The models this product serves, smallest first.
///
/// Both are `Qwen3` so that one prompt profile, one conversation template, one thinking
/// switch, and one redistribution licence cover the whole catalog — and because Apache-2.0
/// outright is the only licence in the registry that costs a mirror nothing. Both are
/// `q4f16_1` because the half-precision shape is the only one a phone has the memory for,
/// and the full-precision twin of the smaller entry already costs more than the larger one
/// saves.
const SERVED_MODELS: [LocalModel; 2] = [
    LocalModel::new("Qwen3-0.6B-q4f16_1-MLC", Quantization::Q4f16, 1404, 4096),
    LocalModel::new("Qwen3-1.7B-q4f16_1-MLC", Quantization::Q4f16, 2037, 4096),
];

/// Returns every model this product serves.
#[must_use]
pub fn served_models() -> &'static [LocalModel] {
    &SERVED_MODELS
}

/// Returns the most capable served model `device` can actually run, if any.
///
/// VRAM stands in for capability here, which holds only because the catalog is one family
/// at two sizes. `None` means this device runs no local model — not that it should be
/// offered a smaller one, and not that inference moves somewhere else without saying so.
#[must_use]
pub fn select_local_model(device: &DeviceCapability) -> Option<&'static LocalModel> {
    SERVED_MODELS
        .iter()
        .filter(|model| model.runs_on(device))
        .max_by_key(|model| model.vram_required_mb)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{DeviceCapability, LocalModel, Quantization, select_local_model, served_models};

    const SMALLER: &str = "Qwen3-0.6B-q4f16_1-MLC";
    const LARGER: &str = "Qwen3-1.7B-q4f16_1-MLC";

    fn generous() -> DeviceCapability {
        DeviceCapability::measured(true, 8192)
    }

    #[test]
    fn the_catalog_serves_two_models_of_one_family() {
        let ids: Vec<&str> = served_models().iter().map(LocalModel::id).collect();

        assert_eq!(ids, vec![SMALLER, LARGER]);
    }

    /// A device that cannot run a format must be refused before it downloads one, so every
    /// served entry has to carry the requirement its quantization implies.
    #[test]
    fn every_served_model_gates_on_the_feature_its_format_needs() {
        for model in served_models() {
            assert_eq!(
                model.requires_shader_f16(),
                model.quantization().requires_shader_f16()
            );
            assert!(model.requires_shader_f16());
        }
    }

    #[test]
    fn a_device_without_f16_runs_nothing_however_much_memory_it_has() {
        let device = DeviceCapability::measured(false, 16_384);

        assert_eq!(select_local_model(&device), None);
        for model in served_models() {
            assert!(!model.runs_on(&device));
        }
    }

    #[test]
    fn a_device_under_the_floor_is_offered_nothing_smaller() {
        let device = DeviceCapability::measured(true, 1200);

        assert_eq!(select_local_model(&device), None);
    }

    #[test]
    fn a_budget_between_the_two_gets_the_one_that_fits() {
        let device = DeviceCapability::measured(true, 1800);

        assert_eq!(
            select_local_model(&device).map(LocalModel::id),
            Some(SMALLER)
        );
    }

    #[test]
    fn a_generous_device_gets_the_larger_model() {
        assert_eq!(
            select_local_model(&generous()).map(LocalModel::id),
            Some(LARGER)
        );
    }

    /// The budget check is only meaningful against the exact figure a selection returns.
    #[test]
    fn a_selection_never_exceeds_what_the_device_granted() {
        for shader_f16 in [false, true] {
            for vram_budget_mb in [0, 1200, 1403, 1404, 2036, 2037, 4096] {
                let device = DeviceCapability::measured(shader_f16, vram_budget_mb);
                if let Some(model) = select_local_model(&device) {
                    assert!(model.runs_on(&device));
                    assert!(model.vram_required_mb() <= vram_budget_mb);
                    assert!(shader_f16);
                }
            }
        }
    }

    /// The host writes these fields straight into its `model_list`, so their names and
    /// values are a contract rather than an internal shape.
    #[test]
    fn a_served_model_encodes_what_the_host_model_list_needs() {
        let encoded = serde_json::to_value(served_models()).expect("catalog encodes");

        assert_eq!(
            encoded,
            json!([
                {
                    "id": SMALLER,
                    "quantization": "q4f16_1",
                    "vram_required_mb": 1404,
                    "context_window": 4096,
                    "requires_shader_f16": true,
                },
                {
                    "id": LARGER,
                    "quantization": "q4f16_1",
                    "vram_required_mb": 2037,
                    "context_window": 4096,
                    "requires_shader_f16": true,
                },
            ])
        );
    }

    #[test]
    fn the_full_precision_format_needs_no_half_precision_support() {
        assert!(!Quantization::Q4f32.requires_shader_f16());
    }
}
