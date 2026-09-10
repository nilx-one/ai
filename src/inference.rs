// © 2026 aiaiaiai · aiaiaiai.org
// SPDX-License-Identifier: MPL-2.0

//! Which local models this product serves, and which one a given device may be offered.
//!
//! Model selection is product work. The foundation's browser adapter loads the identifier it
//! is handed and takes no view on which one that should be, so the catalog — two entries,
//! both `Qwen3`, both `q4f16_1` — and the rule that reads a device against it live here.
//!
//! In this slice Avaia's output is a route choice over targets the world layer already
//! resolved, not prose a person reads, so the catalog is sized for judgement over a short
//! structured prompt rather than for fluency. That is what this slice needs; it is not a
//! statement about what Avaia may ever do.
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
//! a model identity leaves — or a stated reason why this device runs no local Avaia at all.
//! This product's inference is on-device; [`NoLocalModel`] means local computation is
//! unavailable here, and it is not a request to compute somewhere else.

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
    /// all, so it is checked before the catalog is consulted.
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

/// Why this device is offered no local model.
///
/// Returned instead of an absent value so that a device short of `shader-f16`, a device the
/// runtime will not initialize on, and a budget below the smallest entry stay three
/// distinguishable facts. None of them is a request for remote inference: this product's
/// Avaia computes on the device or does not compute.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoLocalModel {
    /// A limit the pinned runtime requires is below its floor, so nothing would load.
    DeviceBelowRuntimeFloor {
        /// The limit that was short.
        limit: DeviceLimit,
    },
    /// The adapter offered no `shader-f16`, which every served entry needs.
    ShaderF16Unavailable,
    /// Every served entry states a requirement above what the product declared.
    BudgetBelowSmallestModel {
        /// What the product declared it would spend, in MB.
        declared_mb: u32,
        /// What the cheapest served entry states it needs, in MB.
        smallest_required_mb: u32,
    },
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
    vram_required_mb: u32,
    context_window: u32,
    requires_shader_f16: bool,
}

impl LocalModel {
    const fn new(id: &'static str, vram_required_mb: u32, context_window: u32) -> Self {
        Self {
            id,
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

    /// Returns the memory the pinned registry states this entry requires, in MB.
    ///
    /// A stated requirement, measured by `MLC` elsewhere — not a measurement of this device.
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
    /// `model_list` as `required_features`, which the pinned registry omits on these
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

/// The models this product serves, smallest first.
///
/// Both are `Qwen3` so that one prompt profile, one conversation template, one thinking
/// switch, and one redistribution licence cover the whole catalog — and because Apache-2.0
/// outright is the only licence in the pinned registry that costs a mirror nothing. Both are
/// `q4f16_1` because the half-precision shape is the only one a phone has the memory for,
/// and the full-precision twin of the smaller entry already costs more than the larger one
/// saves.
const SERVED_MODELS: [LocalModel; 2] = [
    LocalModel::new("Qwen3-0.6B-q4f16_1-MLC", 1404, 4096),
    LocalModel::new("Qwen3-1.7B-q4f16_1-MLC", 2037, 4096),
];

/// Returns every model this product serves.
#[must_use]
pub fn served_models() -> &'static [LocalModel] {
    &SERVED_MODELS
}

/// Returns the largest served model this device may be offered, or why none may be.
///
/// The three refusals are checked in the order they become knowable: what the runtime will
/// not start on at all, then what this device cannot execute, then what this product is
/// willing to spend. The stated requirement stands in for capability only because the
/// catalog is one family at two sizes.
///
/// # Errors
///
/// Returns [`NoLocalModel`] naming which of the three refused.
pub fn select_local_model(
    device: &DeviceCapability,
    budget: &MemoryBudget,
) -> Result<&'static LocalModel, NoLocalModel> {
    if let Some(limit) = device.below_runtime_floor() {
        return Err(NoLocalModel::DeviceBelowRuntimeFloor { limit });
    }

    let supported: Option<&LocalModel> = SERVED_MODELS
        .iter()
        .filter(|model| model.supported_by(device))
        .max_by_key(|model| model.vram_required_mb);
    let Some(_) = supported else {
        return Err(NoLocalModel::ShaderF16Unavailable);
    };

    SERVED_MODELS
        .iter()
        .filter(|model| model.supported_by(device) && model.within(budget))
        .max_by_key(|model| model.vram_required_mb)
        .ok_or(NoLocalModel::BudgetBelowSmallestModel {
            declared_mb: budget.megabytes,
            smallest_required_mb: smallest_requirement(),
        })
}

fn smallest_requirement() -> u32 {
    SERVED_MODELS
        .iter()
        .map(LocalModel::vram_required_mb)
        .min()
        .unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::{
        DeviceCapability, DeviceLimit, LocalModel, MemoryBudget, NoLocalModel,
        declares_half_precision, runtime_floor, select_local_model, served_models,
    };

    const SMALLER: &str = "Qwen3-0.6B-q4f16_1-MLC";
    const LARGER: &str = "Qwen3-1.7B-q4f16_1-MLC";

    /// A device that clears every floor the pinned runtime asks for.
    fn capable_device() -> DeviceCapability {
        DeviceCapability::probed(true, 1 << 30, 1 << 30, 64 << 10, 10)
    }

    #[test]
    fn the_catalog_serves_two_models_of_one_family() {
        let ids: Vec<&str> = served_models().iter().map(LocalModel::id).collect();

        assert_eq!(ids, vec![SMALLER, LARGER]);
    }

    /// The requirement is read from the identifier the host loads, so the two cannot drift.
    #[test]
    fn the_feature_requirement_comes_from_the_identifier() {
        for model in served_models() {
            assert!(model.requires_shader_f16());
        }
        assert!(declares_half_precision("Qwen3-0.6B-q4f16_1-MLC"));
        assert!(!declares_half_precision("Qwen3-0.6B-q4f32_1-MLC"));
        assert!(!declares_half_precision("q4f1"));
    }

    #[test]
    fn a_device_without_f16_is_offered_nothing_however_large_the_budget() {
        let device = DeviceCapability::probed(false, 1 << 30, 1 << 30, 64 << 10, 10);

        assert_eq!(
            select_local_model(&device, &MemoryBudget::declared(16_384)),
            Err(NoLocalModel::ShaderF16Unavailable)
        );
    }

    /// A device the runtime refuses to start on is refused before any model is considered,
    /// and the refusal names the limit rather than blaming the catalog.
    #[test]
    fn each_runtime_floor_refuses_on_its_own() {
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
            assert_eq!(
                select_local_model(&device, &MemoryBudget::declared(16_384)),
                Err(NoLocalModel::DeviceBelowRuntimeFloor { limit: expected })
            );
        }
    }

    /// A capable device with eight storage buffers per stage is the case worth naming: it is
    /// exactly the `WebGPU` default, and the runtime asks for ten.
    #[test]
    fn the_webgpu_default_storage_buffer_count_is_not_enough() {
        let device = DeviceCapability::probed(true, 1 << 30, 1 << 30, 64 << 10, 8);

        assert_eq!(
            select_local_model(&device, &MemoryBudget::declared(4096)),
            Err(NoLocalModel::DeviceBelowRuntimeFloor {
                limit: DeviceLimit::MaxStorageBuffersPerShaderStage
            })
        );
    }

    #[test]
    fn a_budget_under_the_floor_is_offered_nothing_smaller() {
        assert_eq!(
            select_local_model(&capable_device(), &MemoryBudget::declared(1200)),
            Err(NoLocalModel::BudgetBelowSmallestModel {
                declared_mb: 1200,
                smallest_required_mb: 1404,
            })
        );
    }

    #[test]
    fn a_budget_between_the_two_gets_the_one_that_fits() {
        let selected = select_local_model(&capable_device(), &MemoryBudget::declared(1800));

        assert_eq!(selected.map(LocalModel::id), Ok(SMALLER));
    }

    #[test]
    fn a_generous_budget_gets_the_larger_model() {
        let selected = select_local_model(&capable_device(), &MemoryBudget::declared(4096));

        assert_eq!(selected.map(LocalModel::id), Ok(LARGER));
    }

    /// A selection is only meaningful if it stays inside both halves it was checked against.
    #[test]
    fn a_selection_never_exceeds_the_declaration_or_the_device() {
        for shader_f16 in [false, true] {
            for declared_mb in [0, 1200, 1403, 1404, 2036, 2037, 4096] {
                let device = DeviceCapability::probed(shader_f16, 1 << 30, 1 << 30, 64 << 10, 10);
                let budget = MemoryBudget::declared(declared_mb);
                if let Ok(model) = select_local_model(&device, &budget) {
                    assert!(model.within(&budget));
                    assert!(model.supported_by(&device));
                    assert!(shader_f16);
                }
            }
        }
    }
}
