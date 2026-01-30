import type { MetricType } from "web-vitals";

const reportWebVitals = (onPerfEntry?: (metric: MetricType) => void) => {
  if (!onPerfEntry) return;

  import("web-vitals").then(
    ({ onCLS, onINP, onFCP, onLCP, onTTFB }) => {
      onCLS(onPerfEntry);
      onINP(onPerfEntry); // FID → INP
      onFCP(onPerfEntry);
      onLCP(onPerfEntry);
      onTTFB(onPerfEntry);
    }
  );
};

export default reportWebVitals;
