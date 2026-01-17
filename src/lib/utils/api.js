import { invoke } from "@tauri-apps/api/core";

// Library operations
export async function getAllLibraries() {
  return invoke("get_all_libraries");
}

export async function getLibraryById(id) {
  return invoke("get_library_by_id", { id });
}

export async function updateLibrary(id, name = null, description = null, color = null) {
  return invoke("update_library", { id, name, description, color });
}

export async function deleteLibrary(id) {
  return invoke("delete_library", { id });
}

// Patch operations
export async function getAllPatches(filter = null) {
  return invoke("get_all_patches", { filter });
}

export async function getPatchById(id) {
  return invoke("get_patch_by_id", { id });
}

export async function toggleFavorite(patchId) {
  return invoke("toggle_favorite", { patchId });
}

export async function updatePatchNotes(patchId, notes) {
  return invoke("update_patch_notes", { patchId, notes });
}

export async function deletePatch(patchId) {
  return invoke("delete_patch", { patchId });
}

export async function searchPatches(query) {
  return invoke("search_patches", { query });
}

// Sequence operations
export async function getAllSequences(filter = null) {
  return invoke("get_all_sequences", { filter });
}

export async function getSequenceById(id) {
  return invoke("get_sequence_by_id", { id });
}

export async function updateSequenceNotes(sequenceId, notes) {
  return invoke("update_sequence_notes", { sequenceId, notes });
}

export async function deleteSequence(sequenceId) {
  return invoke("delete_sequence", { sequenceId });
}

export async function searchSequences(query) {
  return invoke("search_sequences", { query });
}

// Category operations
export async function getAllCategories() {
  return invoke("get_all_categories");
}

export async function createCategory(name, description = null, color = null) {
  return invoke("create_category", { name, description, color });
}

export async function updateCategory(id, name = null, description = null, color = null) {
  return invoke("update_category", { id, name, description, color });
}

export async function deleteCategory(id) {
  return invoke("delete_category", { id });
}

export async function assignPatchToCategory(patchId, categoryId) {
  return invoke("assign_patch_to_category", { patchId, categoryId });
}

export async function removePatchFromCategory(patchId, categoryId) {
  return invoke("remove_patch_from_category", { patchId, categoryId });
}

export async function assignSequenceToCategory(sequenceId, categoryId) {
  return invoke("assign_sequence_to_category", { sequenceId, categoryId });
}

export async function removeSequenceFromCategory(sequenceId, categoryId) {
  return invoke("remove_sequence_from_category", { sequenceId, categoryId });
}

// Bank operations
export async function getBanksForLibrary(libraryId) {
  return invoke("get_banks_for_library", { libraryId });
}

export async function getBankByNumber(libraryId, bankNumber) {
  return invoke("get_bank_by_number", { libraryId, bankNumber });
}

export async function updateBankName(libraryId, bankNumber, name) {
  return invoke("update_bank_name", { libraryId, bankNumber, name });
}

export async function assignPatchToBank(libraryId, bankNumber, patchNumber, patchId = null) {
  return invoke("assign_patch_to_bank", { libraryId, bankNumber, patchNumber, patchId });
}

export async function assignSequenceToBank(libraryId, bankNumber, sequenceNumber, sequenceId = null) {
  return invoke("assign_sequence_to_bank", { libraryId, bankNumber, sequenceNumber, sequenceId });
}

export async function clearBankSlot(libraryId, bankNumber, patchNumber) {
  return invoke("clear_bank_slot", { libraryId, bankNumber, patchNumber });
}

// Import operations
export async function importLibraryZip(filePath) {
  return invoke("import_library_zip", { filePath });
}

export async function importBankDirectory(directoryPath) {
  return invoke("import_bank_directory", { directoryPath });
}

export async function validateLibraryStructure(path) {
  return invoke("validate_library_structure", { path });
}

// Export operations
export async function exportLibrary(libraryId, outputPath) {
  return invoke("export_library", { libraryId, outputPath });
}

export async function previewExport(libraryId) {
  return invoke("preview_export", { libraryId });
}
