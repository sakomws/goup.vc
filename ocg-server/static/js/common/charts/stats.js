import "/static/js/common/svg-spinner.js";
import {
  clearChartElement,
  initChart,
  loadEChartsScript,
  showChartEmptyState,
} from "/static/js/common/charts/charts.js";
import { debounce } from "/static/js/common/common.js";
import { getElementById, isDatasetReady, markDatasetReady } from "/static/js/common/dom.js";
import { parseJsonText } from "/static/js/common/utils.js";

const CHART_RESIZE_DELAY = 200;
const chartRegistry = new Map();
const markersInFlight = new WeakSet();
let chartResizeObserver = null;

/**
 * Render a chart or show an empty state based on data availability.
 * @param {string} elementId - Target chart element id.
 * @param {Object} option - ECharts option object.
 * @param {boolean} hasData - Whether the chart has data to render.
 * @param {Object} options - Render options.
 * @param {number} options.minTimeSeriesPoints - Minimum points for time-series charts.
 * @returns {echarts.ECharts|null} Chart instance or null.
 */
export const renderChart = (elementId, option, hasData, options = {}) => {
  const { minTimeSeriesPoints = 2 } = options;
  const seriesData = option?.series?.[0]?.data || [];
  const needsTrendLine = option?.xAxis?.type === "time" && option?.series?.[0]?.type === "line";
  const canRender = hasData && (!needsTrendLine || seriesData.length >= minTimeSeriesPoints);

  if (!canRender) {
    unregisterChart(getElementById(document, elementId));
    showChartEmptyState(elementId);
    return null;
  }

  const element = clearChartElement(elementId);
  if (!element) {
    return null;
  }

  const chart = initChart(elementId, option);
  if (chart) {
    registerChart(element, chart);
  }
  return chart;
};

/**
 * Adds a rendered chart to the collection when the container has chart data.
 * @param {Array<echarts.ECharts>} charts - Chart collection.
 * @param {string} elementId - Chart container ID.
 * @param {Object} chartOptions - ECharts options.
 * @param {boolean} hasData - Whether chart data is available.
 * @returns {void}
 */
export const addRenderedChart = (charts, elementId, chartOptions, hasData) => {
  const chart = renderChart(elementId, chartOptions, hasData);
  if (chart) {
    charts.push(chart);
  }
};

/**
 * Disposes registered charts rendered inside an element.
 * @param {Element} root Element being removed or cleaned up.
 * @returns {void}
 */
export const disposeChartsWithin = (root) => {
  if (!(root instanceof Element) || chartRegistry.size === 0) {
    return;
  }
  Array.from(chartRegistry.keys())
    .filter((element) => root === element || root.contains(element))
    .forEach((element) => unregisterChart(element));
};

/**
 * Register a debounced resize handler for chart instances.
 * @param {Iterable<echarts.ECharts>|Function} chartsSource - Chart list or getter.
 * @param {number} delay - Debounce delay in milliseconds.
 * @returns {Function} Debounced resize handler.
 */
export const registerChartResizeHandler = (chartsSource, delay = 200) => {
  const getCharts = typeof chartsSource === "function" ? chartsSource : () => Array.from(chartsSource || []);
  const resizeCharts = debounce(() => {
    const charts = (getCharts() || []).filter(Boolean);
    charts.forEach((chart) => chart.resize());
  }, delay);

  window.addEventListener("resize", resizeCharts);
  return resizeCharts;
};

/**
 * Finds a stats JSON marker in a root or matching root element.
 * @param {Document|Element} root - Root element to search from.
 * @param {string} selector - JSON marker selector.
 * @returns {Element|null} Matching marker.
 */
export const getStatsMarker = (root, selector) => {
  if (root instanceof Element && root.matches(selector)) {
    return root;
  }

  return root.querySelector?.(selector) || null;
};

/**
 * Initializes charts from an inert JSON marker once.
 * @param {Object} config - Initialization configuration.
 * @param {Document|Element} config.root - Root element to search from.
 * @param {string} config.selector - JSON marker selector.
 * @param {string} config.readyKey - Dataset key used to guard initialization.
 * @param {Function} config.initialize - Chart initializer.
 * @param {string} config.parseErrorMessage - Payload parse error message.
 * @param {string} config.initErrorMessage - Chart initialization error message.
 * @param {boolean} config.force - Whether to ignore an existing ready marker.
 * @returns {Promise<void>} Promise resolved when initialization finishes.
 */
export const initializeChartsFromJsonMarker = async ({
  root = document,
  selector,
  readyKey,
  initialize,
  parseErrorMessage,
  initErrorMessage,
  force = false,
}) => {
  const marker = getStatsMarker(root, selector);
  if (!marker || markersInFlight.has(marker) || (!force && isDatasetReady(marker, readyKey))) {
    return;
  }

  const stats = parseJsonText(marker.textContent || "{}", null, (error) => {
    console.error(parseErrorMessage, error);
  });
  if (!stats) {
    return;
  }

  markersInFlight.add(marker);
  try {
    await loadEChartsScript();
    if (!marker.isConnected) {
      return;
    }

    await initialize(stats, { marker });
    markDatasetReady(marker, readyKey);
  } catch (error) {
    console.error(initErrorMessage, error);
  } finally {
    markersInFlight.delete(marker);
  }
};

/**
 * Compute a reasonable label interval for category axes.
 * @param {number} count - Total number of labels.
 * @param {number} target - Target labels to show.
 * @returns {number} ECharts interval value.
 */
export const getCategoryLabelInterval = (count, target = 12) => {
  if (!count) {
    return 0;
  }

  const step = Math.ceil(count / target);
  return Math.max(step - 1, 0);
};

/**
 * Compute a reasonable split count for time axes.
 * @param {number} count - Total number of points.
 * @returns {number} ECharts splitNumber value.
 */
export const getTimeSplitNumber = (count) => {
  if (!count) {
    return 4;
  }

  return Math.min(6, Math.max(3, Math.round(count / 6)));
};

const getChartResizeObserver = () => {
  if (!chartResizeObserver && typeof ResizeObserver !== "undefined") {
    chartResizeObserver = new ResizeObserver((entries) => {
      entries.forEach((entry) => chartRegistry.get(entry.target)?.scheduleResize());
    });
  }
  return chartResizeObserver;
};

const unregisterChart = (element, { keepChart = null } = {}) => {
  const entry = element ? chartRegistry.get(element) : null;
  if (!entry) {
    return;
  }
  entry.cancelResize();
  chartRegistry.delete(element);
  chartResizeObserver?.unobserve(element);
  if (entry.chart !== keepChart && !entry.chart.isDisposed?.()) {
    entry.chart.dispose?.();
  }
};

const pruneDetachedCharts = () => {
  Array.from(chartRegistry.keys())
    .filter((element) => !element.isConnected)
    .forEach((element) => unregisterChart(element));
};

const registerChart = (element, chart) => {
  pruneDetachedCharts();
  unregisterChart(element, { keepChart: chart });

  let timeoutId = null;
  chartRegistry.set(element, {
    cancelResize: () => clearTimeout(timeoutId),
    chart,
    scheduleResize: () => {
      clearTimeout(timeoutId);
      timeoutId = setTimeout(() => chart.resize(), CHART_RESIZE_DELAY);
    },
  });
  getChartResizeObserver()?.observe(element);
};

document.addEventListener("htmx:beforeCleanupElement", (event) => {
  disposeChartsWithin(event.target);
});
