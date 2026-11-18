import { html } from "/static/vendor/js/lit-all.v3.3.1.min.js";
import { MultiSelect } from "/static/js/common/multi-select.js";

const DEFAULT_PLACEHOLDER = "Search labels";
const NO_LABEL_OPTION_NAME = "No label";
export const NO_LABEL_OPTION_VALUE = "__no_label__";

/**
 * CfsLabelSelector renders a searchable multi-select for CFS labels.
 *
 * It maps `labels` to the generic multi-select options and keeps the CFS
 * texts, default field name and `data-cfs-label-*` markers.
 *
 * @property {boolean} includeNoLabel Whether to prepend a synthetic "No label" filter option
 * @property {Array<Object>} labels Available labels for selection
 * @property {number|null} noLabelCount Active submission count for the "No label" option
 * @property {boolean} noLabelSelected Whether the "No label" filter option is selected
 * @property {Array<string>} selected Selected event_cfs_label_id values
 * @fires change Bubbling native event dispatched after a selection change renders
 */
export class CfsLabelSelector extends MultiSelect {
  static properties = {
    includeNoLabel: { type: Boolean, attribute: "include-no-label", reflect: true },
    labels: { type: Array, attribute: "labels" },
    noLabelCount: { type: Number, attribute: "no-label-count" },
    noLabelSelected: { type: Boolean, attribute: "no-label-selected" },
    showCounts: { type: Boolean, attribute: "show-counts", reflect: true },
  };

  constructor() {
    super();
    this.emptyMessage = "No labels found";
    this.includeNoLabel = false;
    this.labels = [];
    this.name = "label_ids";
    this.noLabelCount = null;
    this.noLabelSelected = false;
    this.placeholder = DEFAULT_PLACEHOLDER;
    this.showCounts = false;
    this._addMorePlaceholder = "Add labels";
    this._clearSelectionLabel = "Clear selected labels";
  }

  /**
   * Clears every selection, including the synthetic "No label" option.
   */
  cleanSelected() {
    this.noLabelSelected = false;
    super.cleanSelected();
  }

  /**
   * Keeps the synthetic "No label" option pinned at the top when included.
   * @returns {Array<{value: string, name: string, color?: string}>}
   */
  get _sortedOptions() {
    const sorted = super._sortedOptions;
    if (!this.includeNoLabel || this.keepOrder) {
      return sorted;
    }

    const noLabelIndex = sorted.findIndex((option) => option.value === NO_LABEL_OPTION_VALUE);
    if (noLabelIndex <= 0) {
      return sorted;
    }

    return [sorted[noLabelIndex], ...sorted.slice(0, noLabelIndex), ...sorted.slice(noLabelIndex + 1)];
  }

  /**
   * Normalizes labels and maps them to options before rendering.
   * @param {Map<string, unknown>} changedProperties Changed reactive properties
   */
  willUpdate(changedProperties) {
    if (changedProperties.has("labels")) {
      this.labels = normalizeLabels(this.labels);
    }

    if (
      changedProperties.has("labels") ||
      changedProperties.has("includeNoLabel") ||
      changedProperties.has("noLabelCount") ||
      changedProperties.has("showCounts")
    ) {
      const labelOptions = this.labels.map((label) => ({
        color: label.color,
        ...(this.showCounts && Number.isInteger(label.submissions_count)
          ? { count: label.submissions_count }
          : {}),
        name: label.name,
        value: label.event_cfs_label_id,
      }));

      if (this.includeNoLabel) {
        const noLabelOption = {
          ...(this.showCounts && Number.isInteger(this.noLabelCount) && this.noLabelCount >= 0
            ? { count: this.noLabelCount }
            : {}),
          name: NO_LABEL_OPTION_NAME,
          value: NO_LABEL_OPTION_VALUE,
        };
        this.options = [noLabelOption, ...labelOptions];
      } else {
        this.options = labelOptions;
      }
    }

    if (this.includeNoLabel && changedProperties.has("noLabelSelected")) {
      if (this.noLabelSelected) {
        this.selected = [NO_LABEL_OPTION_VALUE];
      } else if (Array.isArray(this.selected) && this.selected.includes(NO_LABEL_OPTION_VALUE)) {
        this.selected = this.selected.filter((value) => value !== NO_LABEL_OPTION_VALUE);
      }
    }

    super.willUpdate(changedProperties);
  }

  /**
   * Clears every selection and resets the "No label" flag.
   * @param {Event} [event] Click event
   */
  async _clearSelections(event) {
    this.noLabelSelected = false;
    return super._clearSelections(event);
  }

  /**
   * Removes a selection and keeps the "No label" flag in sync.
   * @param {string} value Option value to remove
   * @param {Event} [event] Click event
   */
  async _removeSelection(value, event) {
    if (value === NO_LABEL_OPTION_VALUE) {
      this.noLabelSelected = false;
    }
    return super._removeSelection(value, event);
  }

  /**
   * Renders `no_labels=true` when filtering for unlabeled submissions, or `label_ids[]` otherwise.
   * @returns {import("lit").TemplateResult|symbol}
   */
  _renderHiddenInputs() {
    if (this.includeNoLabel && this.selected.includes(NO_LABEL_OPTION_VALUE)) {
      return html`<input type="hidden" name="no_labels" value="true" />`;
    }
    return super._renderHiddenInputs();
  }

  /**
   * Toggles an option while keeping "No label" mutually exclusive with label selections.
   * @param {string} value Option value to toggle
   */
  async _toggleSelection(value) {
    if (this.disabled) {
      return;
    }

    if (this.includeNoLabel && value === NO_LABEL_OPTION_VALUE) {
      const isSelected = this.selected.includes(NO_LABEL_OPTION_VALUE);
      this.selected = isSelected ? [] : [NO_LABEL_OPTION_VALUE];
      this.noLabelSelected = !isSelected;
      if (this.closeOnSelect) {
        this._combobox.close();
      }
      await this.updateComplete;
      this._dispatchSelectionChange();
      return;
    }

    if (this.includeNoLabel && this.selected.includes(NO_LABEL_OPTION_VALUE)) {
      this.selected = [];
      this.noLabelSelected = false;
    }

    return super._toggleSelection(value);
  }

  updated(changedProperties) {
    super.updated(changedProperties);

    // Keep the legacy markers next to the generic ones.
    this._searchAnchorElement()?.toggleAttribute("data-cfs-label-search", true);
    this._dropdownElement()?.toggleAttribute("data-cfs-label-dropdown", true);
  }
}

/**
 * Normalizes labels into unique entries with string ids.
 * @param {unknown} labels Raw labels
 * @returns {Array<{color: string, event_cfs_label_id: string, name: string, submissions_count?: number}>}
 */
const normalizeLabels = (labels) => {
  if (!Array.isArray(labels)) {
    return [];
  }

  const normalized = [];
  const seen = new Set();
  for (const label of labels) {
    const eventCfsLabelId = String(label?.event_cfs_label_id || "");
    const name = String(label?.name || "").trim();
    const color = String(label?.color || "").trim();
    if (!eventCfsLabelId || !name || seen.has(eventCfsLabelId)) {
      continue;
    }

    seen.add(eventCfsLabelId);
    normalized.push({
      color,
      event_cfs_label_id: eventCfsLabelId,
      name,
      ...(Number.isInteger(label?.submissions_count)
        ? { submissions_count: Math.max(0, label.submissions_count) }
        : {}),
    });
  }
  return normalized;
};

if (!customElements.get("cfs-label-selector")) {
  customElements.define("cfs-label-selector", CfsLabelSelector);
}
