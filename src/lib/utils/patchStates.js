/**
 * Utility functions for patch state detection and display
 * Used by PatchCard component to show visual state indicators
 */

/**
 * Check if patch was imported within the last 7 days
 * @param {string} createdAt - ISO 8601 timestamp
 * @returns {boolean} True if patch is less than 7 days old
 */
export function isNewPatch(createdAt) {
  if (!createdAt) return false;

  const created = new Date(createdAt);
  const now = new Date();
  const daysSince = (now - created) / (1000 * 60 * 60 * 24);

  return daysSince <= 7;
}

/**
 * Check if patch has been modified since import
 * @param {string} createdAt - ISO 8601 timestamp
 * @param {string} updatedAt - ISO 8601 timestamp
 * @returns {boolean} True if timestamps differ
 */
export function isModifiedPatch(createdAt, updatedAt) {
  if (!createdAt || !updatedAt) return false;
  return createdAt !== updatedAt;
}

/**
 * Format usage count for display
 * @param {number} count - Number of times patch is used
 * @returns {string} Formatted count (e.g., "×3", "×99+")
 */
export function formatUsageCount(count) {
  if (!count || count <= 0) return '';
  if (count > 99) return '×99+';
  return `×${count}`;
}

/**
 * Generate comprehensive ARIA label for patch card
 * @param {Object} patch - Patch object with all properties
 * @returns {string} Accessible label describing patch and all states
 */
export function generatePatchStateAriaLabel(patch) {
  const parts = [patch.name];

  if (patch.is_favorite) {
    parts.push('Favorite');
  }

  if (isNewPatch(patch.created_at)) {
    parts.push('Recently imported');
  }

  if (!patch.tags || patch.tags.length === 0) {
    parts.push('Untagged');
  } else {
    parts.push(`Tagged: ${patch.tags.join(', ')}`);
  }

  if (patch.usage_count && patch.usage_count > 1) {
    parts.push(`Used in ${patch.usage_count} banks`);
  }

  if (isModifiedPatch(patch.created_at, patch.updated_at)) {
    parts.push('Modified');
  }

  return parts.join('. ');
}
