// © 2026 aiaiaiai · aiaiaiai.org
// SPDX-License-Identifier: MPL-2.0

//! Which local models this product serves, and which of them a given device may be offered.
//!
//! Model selection is product work. The foundation's browser adapter loads the identifier it
//! is handed and takes no view on which one that should be, so the catalog — five entries
//! across four families, all `q4f16_1` — and the rule that reads a device against it live
//! here.
//!
//! # Eligible, chosen, default
//!
//! Three different facts, and this module owns only two of them.
//!
//! **Eligibility** is the device's and the product's: [`eligible_local_models`] answers for
//! every served entry, each admitted or refused with its reason. An entry a device cannot run
//! is still in the answer, because a person is shown it with its reason rather than having it
//! quietly vanish.
//!
//! **Choice** is the owner's, made within eligibility, and it lives on the device that made
//! it. Nothing here stores or infers it.
//!
//! **The default** is what a surface with nobody to ask runs. [`select_local_model`] returns
//! it when it is eligible and the default's own refusal when it is not. It never picks another
//! entry on someone's behalf: across four families, "the largest one that fits" or "the next
//! one down" would be a judgement nobody made.
//!
//! # Three kinds of fact, kept apart
//!
//! [`DeviceCapability`] carries only what the `WebGPU` probe can actually obtain. Adapter
//! limits and features are measurable; **device memory is not** — `WebGPU` exposes no
//! memory budget, so nothing here pretends to have measured one.
//!
//! [`MemoryBudget`] is the other half, and it is policy: how much memory this product has
//! decided to spend on a model on this surface. Its constructor is named for that
//! provenance so that a reader cannot mistake it for something a device reported.
//!
//! The floors in [`runtime_floor`] are neither — they are quoted from the pinned runtime,
//! which refuses to initialize below them whatever model is asked for.
//!
//! Nothing in this module downloads, probes, or loads. A capability and a budget arrive, and
//! verdicts leave. This product's inference is on-device; a refusal means local computation
//! is unavailable for that entry here, and it is not a request to compute somewhere else.

/// Device limits the pinned runtime requires before any model is loaded.
///
/// Quoted from `@mlc-ai/web-llm@0.2.84`, which requests these as `requiredLimits` when it
/// acquires a `WebGPU` device: it asks for 1 GiB of buffer and storage-binding headroom,
/// falls back once to the values below, and throws if even those are refused. The last two
/// have no fallback at all, and `maxStorageBuffersPerShaderStage` sits above the `WebGPU`
/// default of 8 — a device can be perfectly modern and still fail here.
pub mod runtime_floor {
    /// Smallest `maxBufferSize` the runtime will accept, in bytes (256 MiB).
    pub const MAX_BUFFER_SIZE: u64 = 1 << 28;
    /// Smallest `maxStorageBufferBindingSize` the runtime will accept, in bytes (128 MiB).
    pub const MAX_STORAGE_BUFFER_BINDING_SIZE: u64 = 1 << 27;
    /// Required `maxComputeWorkgroupStorageSize`, in bytes (32 KiB). No fallback.
    pub const MAX_COMPUTE_WORKGROUP_STORAGE_SIZE: u32 = 32 << 10;
    /// Required `maxStorageBuffersPerShaderStage`. No fallback; the `WebGPU` default is 8.
    pub const MAX_STORAGE_BUFFERS_PER_SHADER_STAGE: u32 = 10;
}

/// One limit a device reported, named so a refusal can say which one was short.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceLimit {
    /// `maxBufferSize`.
    MaxBufferSize,
    /// `maxStorageBufferBindingSize`.
    MaxStorageBufferBindingSize,
    /// `maxComputeWorkgroupStorageSize`.
    MaxComputeWorkgroupStorageSize,
    /// `maxStorageBuffersPerShaderStage`.
    MaxStorageBuffersPerShaderStage,
}

impl DeviceLimit {
    /// Returns the limit's `WebGPU` name, as an adapter reports it.
    #[must_use]
    pub const fn webgpu_name(self) -> &'static str {
        match self {
            Self::MaxBufferSize => "maxBufferSize",
            Self::MaxStorageBufferBindingSize => "maxStorageBufferBindingSize",
            Self::MaxComputeWorkgroupStorageSize => "maxComputeWorkgroupStorageSize",
            Self::MaxStorageBuffersPerShaderStage => "maxStorageBuffersPerShaderStage",
        }
    }
}

/// What the `WebGPU` probe reported about this device.
///
/// Every field is something an adapter answers: the `shader-f16` feature and four limits.
/// There is no memory field, because there is no way to ask a browser for one, and a value
/// nobody can obtain must not travel in a type whose constructor claims it was probed.
///
/// There is also no default and no unmeasured constructor. A surface nobody probed is not a
/// permissive surface, and treating it as one is the substitution the foundation's own probe
/// results refuse by name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceCapability {
    shader_f16: bool,
    max_buffer_size: u64,
    max_storage_buffer_binding_size: u64,
    max_compute_workgroup_storage_size: u32,
    max_storage_buffers_per_shader_stage: u32,
}

impl DeviceCapability {
    /// Records one probe result. Every argument is a value the adapter reported.
    #[must_use]
    pub const fn probed(
        shader_f16: bool,
        max_buffer_size: u64,
        max_storage_buffer_binding_size: u64,
        max_compute_workgroup_storage_size: u32,
        max_storage_buffers_per_shader_stage: u32,
    ) -> Self {
        Self {
            shader_f16,
            max_buffer_size,
            max_storage_buffer_binding_size,
            max_compute_workgroup_storage_size,
            max_storage_buffers_per_shader_stage,
        }
    }

    /// Returns whether the adapter offered the `shader-f16` feature.
    #[must_use]
    pub const fn shader_f16(&self) -> bool {
        self.shader_f16
    }

    /// Returns the first limit that falls below what the pinned runtime demands.
    ///
    /// This is about the runtime, not about any model: a device short here loads nothing at
    /// all, so it refuses every entry before the entry's own requirements are read.
    #[must_use]
    pub const fn below_runtime_floor(&self) -> Option<DeviceLimit> {
        if self.max_buffer_size < runtime_floor::MAX_BUFFER_SIZE {
            return Some(DeviceLimit::MaxBufferSize);
        }
        if self.max_storage_buffer_binding_size < runtime_floor::MAX_STORAGE_BUFFER_BINDING_SIZE {
            return Some(DeviceLimit::MaxStorageBufferBindingSize);
        }
        if self.max_compute_workgroup_storage_size
            < runtime_floor::MAX_COMPUTE_WORKGROUP_STORAGE_SIZE
        {
            return Some(DeviceLimit::MaxComputeWorkgroupStorageSize);
        }
        if self.max_storage_buffers_per_shader_stage
            < runtime_floor::MAX_STORAGE_BUFFERS_PER_SHADER_STAGE
        {
            return Some(DeviceLimit::MaxStorageBuffersPerShaderStage);
        }
        None
    }
}

/// How much device memory this product has decided to spend on a model here.
///
/// This is policy, not measurement. `WebGPU` reports no memory budget, and the registry's
/// `vram_required_MB` is a figure `MLC` measured on its own hardware — so the comparison
/// this type takes part in is *our declared willingness against the model's stated
/// requirement*, and both halves are claims rather than readings. The constructor is named
/// `declared` so that no caller can read it as anything else.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryBudget {
    megabytes: u32,
}

impl MemoryBudget {
    /// Declares, on the product's authority, how much memory a model may take here.
    #[must_use]
    pub const fn declared(megabytes: u32) -> Self {
        Self { megabytes }
    }

    /// Returns the declared budget in whole MB.
    #[must_use]
    pub const fn megabytes(&self) -> u32 {
        self.megabytes
    }
}

/// Why one served entry is not offered on this device.
///
/// Three distinguishable facts, checked in the order they become knowable: what the runtime
/// will not start on at all, what this device cannot execute, and what this product is
/// willing to spend. None of them is a request for remote inference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ineligible {
    /// A limit the pinned runtime requires is below its floor, so nothing would load.
    DeviceBelowRuntimeFloor {
        /// The limit that was short.
        limit: DeviceLimit,
    },
    /// The adapter offered no `shader-f16`, which this entry's quantization needs.
    ShaderF16Unavailable,
    /// The entry states a requirement above what the product declared for this surface.
    OverBudget {
        /// What the entry states it needs, in whole MB rounded up.
        required_mb: u32,
        /// What the product declared it would spend, in MB.
        declared_mb: u32,
    },
}

/// The model family an entry belongs to.
///
/// A family is what shares a tokenizer, a conversation template and a way of being prompted.
/// Four of them means four prompt profiles to validate, which is now an obligation rather than
/// something one family's validation covers — see `docs/model-selection.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModelFamily {
    /// Qwen3 — the only family here that reads the `enable_thinking` switch.
    Qwen3,
    /// `SmolLM2`.
    SmolLm2,
    /// `OLMo` 2.
    Olmo2,
    /// Llama 3.2.
    Llama32,
}

impl ModelFamily {
    /// Returns the family's identifier, as `nilx-one/web`'s catalog spells it.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Qwen3 => "qwen3",
            Self::SmolLm2 => "smollm2",
            Self::Olmo2 => "olmo2",
            Self::Llama32 => "llama3.2",
        }
    }
}

/// The licence an entry's weights are distributed under, as the upstream card names it.
///
/// Serving weights from our host is redistribution, so the licence is part of the entry and
/// not a footnote to it. What each one obliges is recorded in `docs/model-licences.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Licence {
    /// Apache License 2.0.
    Apache2,
    /// Llama 3.2 Community License Agreement.
    Llama32Community,
}

impl Licence {
    /// Returns the licence identifier the upstream model card's `license:` field carries.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Apache2 => "apache-2.0",
            Self::Llama32Community => "llama3.2",
        }
    }

    /// Returns whether a model trained on this licence's outputs must carry its name.
    ///
    /// Llama 3.2 §1.b.i: a model created, trained, fine-tuned or otherwise improved with Llama
    /// outputs and then distributed must have a name that begins with "Llama". Output an entry
    /// under this licence produced is therefore marked, so a training-egress policy can keep
    /// it out rather than rename whatever it would have trained.
    #[must_use]
    pub const fn names_derived_models(self) -> bool {
        matches!(self, Self::Llama32Community)
    }
}

/// One model this product is prepared to serve.
///
/// This is `ai`'s own descriptor and deliberately not `WebLLM`'s `ModelRecord`: that record
/// needs artifact URLs — `model`, `model_lib` — which depend on the mirror a deployment
/// serves from, and which this crate neither owns nor should invent. Mapping a served entry
/// onto a `model_list` entry, with `required_features`, `overrides` and integrity hashes, is
/// the host adapter's work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalModel {
    id: &'static str,
    family: ModelFamily,
    licence: Licence,
    vram_required_mb: u32,
    context_window: u32,
    requires_shader_f16: bool,
}

impl LocalModel {
    const fn new(
        id: &'static str,
        family: ModelFamily,
        licence: Licence,
        vram_required_mb: u32,
        context_window: u32,
    ) -> Self {
        Self {
            id,
            family,
            licence,
            vram_required_mb,
            context_window,
            requires_shader_f16: declares_half_precision(id),
        }
    }

    /// Returns the `model_id` the host loads.
    #[must_use]
    pub const fn id(&self) -> &'static str {
        self.id
    }

    /// Returns the family this entry belongs to.
    #[must_use]
    pub const fn family(&self) -> ModelFamily {
        self.family
    }

    /// Returns the licence this entry's weights are redistributed under.
    #[must_use]
    pub const fn licence(&self) -> Licence {
        self.licence
    }

    /// Returns the memory the pinned registry states this entry requires, in whole MB.
    ///
    /// The registry's figure rounded up, so a budget in whole MB compares the same way it
    /// would against the fractional figure. A stated requirement, measured by `MLC`
    /// elsewhere — not a measurement of this device.
    #[must_use]
    pub const fn vram_required_mb(&self) -> u32 {
        self.vram_required_mb
    }

    /// Returns the context window this entry is configured and measured for.
    #[must_use]
    pub const fn context_window(&self) -> u32 {
        self.context_window
    }

    /// Returns whether the device must offer `shader-f16` to run this entry.
    ///
    /// Read out of the `model_id`, which is where `MLC` states the quantization, so it
    /// cannot drift from the artifacts the host actually loads. The host writes it into its
    /// `model_list` as `required_features`, which the pinned registry omits on most of these
    /// entries — leaving a device without f16 to download a whole model before failing.
    #[must_use]
    pub const fn requires_shader_f16(&self) -> bool {
        self.requires_shader_f16
    }

    /// Returns whether `device` offers the features this entry needs.
    #[must_use]
    pub const fn supported_by(&self, device: &DeviceCapability) -> bool {
        !self.requires_shader_f16 || device.shader_f16
    }

    /// Returns whether the product's declared budget covers this entry's stated requirement.
    #[must_use]
    pub const fn within(&self, budget: &MemoryBudget) -> bool {
        self.vram_required_mb <= budget.megabytes
    }

    /// Returns whether this device and this budget admit the entry, or why not.
    ///
    /// # Errors
    ///
    /// Returns [`Ineligible`] naming the first of the three checks that refused.
    pub const fn eligibility(
        &self,
        device: &DeviceCapability,
        budget: &MemoryBudget,
    ) -> Result<(), Ineligible> {
        if let Some(limit) = device.below_runtime_floor() {
            return Err(Ineligible::DeviceBelowRuntimeFloor { limit });
        }
        if !self.supported_by(device) {
            return Err(Ineligible::ShaderF16Unavailable);
        }
        if !self.within(budget) {
            return Err(Ineligible::OverBudget {
                required_mb: self.vram_required_mb,
                declared_mb: budget.megabytes,
            });
        }
        Ok(())
    }
}

/// Reads the quantization out of a `model_id`, which is where `MLC` writes it.
const fn declares_half_precision(id: &str) -> bool {
    contains(id.as_bytes(), b"q4f16")
}

const fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.len() > haystack.len() {
        return false;
    }
    let mut start = 0;
    while start + needle.len() <= haystack.len() {
        let mut index = 0;
        while index < needle.len() && haystack[start + index] == needle[index] {
            index += 1;
        }
        if index == needle.len() {
            return true;
        }
        start += 1;
    }
    false
}

/// The models this product serves, in the order they are presented.
///
/// The order is presentation and nothing more. It is not a preference the device reads:
/// eligibility answers for every entry, the owner chooses among the eligible ones, and a
/// surface with nobody to ask runs [`DEFAULT_MODEL_INDEX`] or nothing.
///
/// Every entry is `q4f16_1` because the half-precision shape is the only one a phone has the
/// memory for. VRAM is the pinned registry's `vram_required_MB`, rounded up.
const SERVED_MODELS: [LocalModel; 5] = [
    LocalModel::new(
        "Qwen3-0.6B-q4f16_1-MLC",
        ModelFamily::Qwen3,
        Licence::Apache2,
        1404,
        4096,
    ),
    LocalModel::new(
        "Qwen3-1.7B-q4f16_1-MLC",
        ModelFamily::Qwen3,
        Licence::Apache2,
        2037,
        4096,
    ),
    LocalModel::new(
        "SmolLM2-360M-Instruct-q4f16_1-MLC",
        ModelFamily::SmolLm2,
        Licence::Apache2,
        377,
        4096,
    ),
    LocalModel::new(
        "OLMo-2-0425-1B-Instruct-q4f16_1-MLC",
        ModelFamily::Olmo2,
        Licence::Apache2,
        1777,
        4096,
    ),
    LocalModel::new(
        "Llama-3.2-1B-Instruct-q4f16_1-MLC",
        ModelFamily::Llama32,
        Licence::Llama32Community,
        880,
        4096,
    ),
];

/// Which entry of [`SERVED_MODELS`] is the default: `Qwen3-0.6B`, unchanged by the catalog.
const DEFAULT_MODEL_INDEX: usize = 0;

/// Returns every model this product serves, in presentation order.
#[must_use]
pub fn served_models() -> &'static [LocalModel] {
    &SERVED_MODELS
}

/// Returns the entry a surface with nobody to ask runs.
#[must_use]
pub fn default_local_model() -> &'static LocalModel {
    &SERVED_MODELS[DEFAULT_MODEL_INDEX]
}

/// Returns the served entry with this `model_id`, if this product serves one.
#[must_use]
pub fn find_local_model(id: &str) -> Option<&'static LocalModel> {
    SERVED_MODELS.iter().find(|model| model.id == id)
}

/// One served entry, and whether this device and this budget admit it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Admission {
    model: &'static LocalModel,
    verdict: Result<(), Ineligible>,
}

impl Admission {
    /// Returns the entry this verdict is about.
    #[must_use]
    pub const fn model(&self) -> &'static LocalModel {
        self.model
    }

    /// Returns the verdict: admitted, or refused with its reason.
    ///
    /// # Errors
    ///
    /// Returns [`Ineligible`] when this entry is not offered on this device.
    pub const fn verdict(&self) -> Result<(), Ineligible> {
        self.verdict
    }

    /// Returns whether this entry may be offered — and therefore chosen — here.
    #[must_use]
    pub const fn is_eligible(&self) -> bool {
        self.verdict.is_ok()
    }
}

/// Returns every served entry with its verdict for this device and this budget.
///
/// Every entry is answered for, the refused ones included: an entry a device cannot run is
/// shown with its reason, never quietly left out, and never offered.
#[must_use]
pub fn eligible_local_models(device: &DeviceCapability, budget: &MemoryBudget) -> Vec<Admission> {
    SERVED_MODELS
        .iter()
        .map(|model| Admission {
            model,
            verdict: model.eligibility(device, budget),
        })
        .collect()
}

/// Returns the default entry when this device and this budget admit it.
///
/// This is the answer for a surface with nobody to ask. It is not a selection among the
/// eligible: an owner who can be asked chooses within [`eligible_local_models`], and a surface
/// that cannot ask runs the default or nothing — never another family picked on someone's
/// behalf because it happened to fit.
///
/// # Errors
///
/// Returns the default entry's own [`Ineligible`] reason.
pub fn select_local_model(
    device: &DeviceCapability,
    budget: &MemoryBudget,
) -> Result<&'static LocalModel, Ineligible> {
    let default = default_local_model();
    default.eligibility(device, budget).map(|()| default)
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::{
        DeviceCapability, DeviceLimit, Ineligible, Licence, LocalModel, MemoryBudget, ModelFamily,
        declares_half_precision, default_local_model, eligible_local_models, find_local_model,
        runtime_floor, select_local_model, served_models,
    };

    const QWEN_SMALL: &str = "Qwen3-0.6B-q4f16_1-MLC";
    const QWEN_LARGE: &str = "Qwen3-1.7B-q4f16_1-MLC";
    const SMOLLM2: &str = "SmolLM2-360M-Instruct-q4f16_1-MLC";
    const OLMO2: &str = "OLMo-2-0425-1B-Instruct-q4f16_1-MLC";
    const LLAMA: &str = "Llama-3.2-1B-Instruct-q4f16_1-MLC";

    /// The fixture `nilx-one/web` tests its own reading of this rule against.
    const SHARED_FIXTURE: &str = include_str!("../fixtures/local-model-eligibility.json");

    /// A device that clears every floor the pinned runtime asks for.
    fn capable_device() -> DeviceCapability {
        DeviceCapability::probed(true, 1 << 30, 1 << 30, 64 << 10, 10)
    }

    fn eligible_ids(budget_mb: u32) -> Vec<&'static str> {
        eligible_local_models(&capable_device(), &MemoryBudget::declared(budget_mb))
            .iter()
            .filter(|admission| admission.is_eligible())
            .map(|admission| admission.model().id())
            .collect()
    }

    #[test]
    fn the_catalog_serves_five_entries_across_four_families() {
        let ids: Vec<&str> = served_models().iter().map(LocalModel::id).collect();
        assert_eq!(ids, vec![QWEN_SMALL, QWEN_LARGE, SMOLLM2, OLMO2, LLAMA]);

        let families: Vec<ModelFamily> = served_models().iter().map(LocalModel::family).collect();
        assert_eq!(
            families,
            vec![
                ModelFamily::Qwen3,
                ModelFamily::Qwen3,
                ModelFamily::SmolLm2,
                ModelFamily::Olmo2,
                ModelFamily::Llama32,
            ]
        );
    }

    #[test]
    fn the_default_is_unchanged() {
        assert_eq!(default_local_model().id(), QWEN_SMALL);
    }

    #[test]
    fn every_entry_carries_the_licence_its_upstream_card_names() {
        for model in served_models() {
            let expected = if model.family() == ModelFamily::Llama32 {
                Licence::Llama32Community
            } else {
                Licence::Apache2
            };
            assert_eq!(model.licence(), expected, "{}", model.id());
        }
        assert_eq!(Licence::Apache2.id(), "apache-2.0");
        assert_eq!(Licence::Llama32Community.id(), "llama3.2");
    }

    /// Only Llama 3.2's licence reaches the name of a model trained on its outputs.
    #[test]
    fn only_llama_names_what_is_trained_on_its_output() {
        assert!(Licence::Llama32Community.names_derived_models());
        assert!(!Licence::Apache2.names_derived_models());
    }

    /// The requirement is read from the identifier the host loads, so the two cannot drift.
    #[test]
    fn the_feature_requirement_comes_from_the_identifier() {
        for model in served_models() {
            assert!(model.requires_shader_f16(), "{}", model.id());
        }
        assert!(declares_half_precision("Qwen3-0.6B-q4f16_1-MLC"));
        assert!(!declares_half_precision("Qwen3-0.6B-q4f32_1-MLC"));
        assert!(!declares_half_precision("q4f1"));
    }

    #[test]
    fn finds_a_served_entry_and_nothing_else() {
        assert_eq!(find_local_model(LLAMA).map(LocalModel::id), Some(LLAMA));
        assert_eq!(find_local_model("gemma3-1b-it-q4f16_1-MLC"), None);
    }

    /// Every entry is answered for, and the refused ones say why.
    #[test]
    fn a_budget_admits_some_families_and_refuses_others() {
        let admissions = eligible_local_models(&capable_device(), &MemoryBudget::declared(1000));

        assert_eq!(admissions.len(), served_models().len());
        assert_eq!(eligible_ids(1000), vec![SMOLLM2, LLAMA]);
        for admission in admissions
            .iter()
            .filter(|admission| !admission.is_eligible())
        {
            assert_eq!(
                admission.verdict(),
                Err(Ineligible::OverBudget {
                    required_mb: admission.model().vram_required_mb(),
                    declared_mb: 1000,
                })
            );
        }
    }

    /// Nobody chose Llama for this surface just because it fits where the default does not.
    #[test]
    fn a_surface_with_nobody_to_ask_runs_the_default_or_nothing() {
        assert_eq!(
            select_local_model(&capable_device(), &MemoryBudget::declared(1000)),
            Err(Ineligible::OverBudget {
                required_mb: 1404,
                declared_mb: 1000,
            })
        );
        assert_eq!(
            select_local_model(&capable_device(), &MemoryBudget::declared(1500))
                .map(LocalModel::id),
            Ok(QWEN_SMALL)
        );
    }

    /// Size is not a judgement: a generous budget admits everything and still runs the default.
    #[test]
    fn the_largest_entry_does_not_win() {
        assert_eq!(
            eligible_ids(4096),
            vec![QWEN_SMALL, QWEN_LARGE, SMOLLM2, OLMO2, LLAMA]
        );
        assert_eq!(
            select_local_model(&capable_device(), &MemoryBudget::declared(4096))
                .map(LocalModel::id),
            Ok(QWEN_SMALL)
        );
    }

    #[test]
    fn a_device_without_f16_is_offered_nothing_however_large_the_budget() {
        let device = DeviceCapability::probed(false, 1 << 30, 1 << 30, 64 << 10, 10);

        for admission in eligible_local_models(&device, &MemoryBudget::declared(16_384)) {
            assert_eq!(admission.verdict(), Err(Ineligible::ShaderF16Unavailable));
        }
        assert_eq!(
            select_local_model(&device, &MemoryBudget::declared(16_384)),
            Err(Ineligible::ShaderF16Unavailable)
        );
    }

    /// A device the runtime refuses to start on refuses every entry, and the refusal names the
    /// limit rather than blaming any entry.
    #[test]
    fn each_runtime_floor_refuses_every_entry_on_its_own() {
        let cases = [
            (
                DeviceCapability::probed(
                    true,
                    runtime_floor::MAX_BUFFER_SIZE - 1,
                    1 << 30,
                    64 << 10,
                    10,
                ),
                DeviceLimit::MaxBufferSize,
            ),
            (
                DeviceCapability::probed(
                    true,
                    1 << 30,
                    runtime_floor::MAX_STORAGE_BUFFER_BINDING_SIZE - 1,
                    64 << 10,
                    10,
                ),
                DeviceLimit::MaxStorageBufferBindingSize,
            ),
            (
                DeviceCapability::probed(
                    true,
                    1 << 30,
                    1 << 30,
                    runtime_floor::MAX_COMPUTE_WORKGROUP_STORAGE_SIZE - 1,
                    10,
                ),
                DeviceLimit::MaxComputeWorkgroupStorageSize,
            ),
            (
                DeviceCapability::probed(
                    true,
                    1 << 30,
                    1 << 30,
                    64 << 10,
                    runtime_floor::MAX_STORAGE_BUFFERS_PER_SHADER_STAGE - 1,
                ),
                DeviceLimit::MaxStorageBuffersPerShaderStage,
            ),
        ];

        for (device, expected) in cases {
            for admission in eligible_local_models(&device, &MemoryBudget::declared(16_384)) {
                assert_eq!(
                    admission.verdict(),
                    Err(Ineligible::DeviceBelowRuntimeFloor { limit: expected })
                );
            }
        }
    }

    /// A capable device with eight storage buffers per stage is the case worth naming: it is
    /// exactly the `WebGPU` default, and the runtime asks for ten.
    #[test]
    fn the_webgpu_default_storage_buffer_count_is_not_enough() {
        let device = DeviceCapability::probed(true, 1 << 30, 1 << 30, 64 << 10, 8);

        assert_eq!(
            select_local_model(&device, &MemoryBudget::declared(4096)),
            Err(Ineligible::DeviceBelowRuntimeFloor {
                limit: DeviceLimit::MaxStorageBuffersPerShaderStage
            })
        );
    }

    /// An admission is only meaningful if it stays inside both halves it was checked against.
    #[test]
    fn an_admission_never_exceeds_the_declaration_or_the_device() {
        for shader_f16 in [false, true] {
            for declared_mb in [
                0, 376, 377, 879, 880, 1403, 1404, 1776, 1777, 2036, 2037, 4096,
            ] {
                let device = DeviceCapability::probed(shader_f16, 1 << 30, 1 << 30, 64 << 10, 10);
                let budget = MemoryBudget::declared(declared_mb);
                for admission in eligible_local_models(&device, &budget) {
                    if admission.is_eligible() {
                        assert!(admission.model().within(&budget));
                        assert!(admission.model().supported_by(&device));
                        assert!(shader_f16);
                    }
                }
            }
        }
    }

    /// The same cases `nilx-one/web` reads its own copy of, so the two rules cannot drift.
    #[test]
    fn the_shared_fixture_holds() {
        let fixture: Value = serde_json::from_str(SHARED_FIXTURE).expect("fixture parses");
        let cases = fixture["cases"].as_array().expect("cases");
        assert!(!cases.is_empty());

        for case in cases {
            let name = case["name"].as_str().expect("name");
            let device = &case["device"];
            let capability = DeviceCapability::probed(
                device["shader_f16"].as_bool().expect("shader_f16"),
                device["max_buffer_size"].as_u64().expect("max_buffer_size"),
                device["max_storage_buffer_binding_size"]
                    .as_u64()
                    .expect("max_storage_buffer_binding_size"),
                u32::try_from(
                    device["max_compute_workgroup_storage_size"]
                        .as_u64()
                        .expect("u64"),
                )
                .expect("u32"),
                u32::try_from(
                    device["max_storage_buffers_per_shader_stage"]
                        .as_u64()
                        .expect("u64"),
                )
                .expect("u32"),
            );
            let budget = MemoryBudget::declared(
                u32::try_from(case["budget_mb"].as_u64().expect("budget_mb")).expect("u32"),
            );

            for admission in eligible_local_models(&capability, &budget) {
                let expected = case["expected"][admission.model().id()]
                    .as_str()
                    .unwrap_or_else(|| {
                        panic!("{name}: no expectation for {}", admission.model().id())
                    });
                assert_eq!(
                    fixture_word(admission.verdict()),
                    expected,
                    "{name}: {}",
                    admission.model().id()
                );
            }
            assert_eq!(
                fixture_word(select_local_model(&capability, &budget).map(|_| ())),
                case["default"].as_str().expect("default"),
                "{name}: default"
            );
        }
    }

    fn fixture_word(verdict: Result<(), Ineligible>) -> String {
        match verdict {
            Ok(()) => "eligible".to_owned(),
            Err(Ineligible::ShaderF16Unavailable) => "missing_shader_f16".to_owned(),
            Err(Ineligible::OverBudget { .. }) => "over_budget".to_owned(),
            Err(Ineligible::DeviceBelowRuntimeFloor { limit }) => {
                format!("below_runtime_floor:{}", limit.webgpu_name())
            }
        }
    }
}
