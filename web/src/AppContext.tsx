import React, {
  createContext,
  useContext,
  useEffect,
  useState,
  useRef,
} from "react";
import PACKAGE from "../package.json";

type WasmModule = {
  zhconv: (
    text: string,
    target: string,
    mediawiki: boolean,
    cgroup: string,
  ) => Promise<string>;
  get_build_timestamp: () => string;
  get_commit: () => string;
  get_mediawiki_commit: () => string;
  get_opencc_commit: () => string;
  infer_variant_confidence: (text: string) => string;
};

export type RulesetMode = "mediawiki" | "opencc" | "both";

type AppContextType = {
  wasm: WasmModule | null;
  rulesetMode: RulesetMode;
  setRulesetMode: (mode: RulesetMode) => void;
  cgroups: {
    data: { [name: string]: string };
    timestamp: number | null;
  } | null;
};

const AppContext = createContext<AppContextType>({
  wasm: null,
  rulesetMode: "mediawiki",
  setRulesetMode: () => {},
  cgroups: null,
});

export const useApp = () => useContext(AppContext);

// Alias for backward compatibility during refactor, or we can just remove it.
// keeping it temporarily might help if I miss one, but the goal is to replace.
// export const useWasm = useApp;

// Initialize rulesetMode from localStorage immediately
const getInitialRulesetMode = (): RulesetMode => {
  const stored = localStorage.getItem(`${PACKAGE.name}-ruleset-mode`);
  if (
    stored &&
    (stored === "mediawiki" || stored === "opencc" || stored === "both")
  ) {
    return stored as RulesetMode;
  }
  return "mediawiki";
};

const initialRulesetMode = getInitialRulesetMode();

// Start loading WASM immediately
const getWasmImportPath = (mode: RulesetMode) => {
  switch (mode) {
    case "mediawiki":
      return import("@pkg-mediawiki/zhconv.js");
    case "opencc":
      return import("@pkg-opencc/zhconv.js");
    case "both":
      return import("@pkg-both/zhconv.js");
  }
};

const initialWasmPromise = getWasmImportPath(initialRulesetMode);

export const AppProvider: React.FC<React.PropsWithChildren> = ({
  children,
}) => {
  // wasmCache stores the loaded WebAssembly modules to prevent re-fetching and re-instantiating
  // them every time the OpenCC option is toggled. This improves performance and avoids
  // unnecessary loading indicator flashes.
  const wasmCache = useRef<{ [key: string]: WasmModule }>({});
  const [wasm, setWasm] = useState<WasmModule | null>(null);
  const [rulesetMode, setRulesetMode] =
    useState<RulesetMode>(initialRulesetMode);

  const [cgroups, setCGroups] = useState<{
    data: { [name: string]: string };
    timestamp: number | null;
  } | null>(null);

  // Load WASM
  useEffect(() => {
    let cancelled = false;
    localStorage.setItem(`${PACKAGE.name}-ruleset-mode`, rulesetMode);
    const loadWasm = async () => {
      const cacheKey = rulesetMode;
      const loadingLabel = `zhconv loading (${cacheKey})`;
      console.time(loadingLabel);

      // Check if this is the initial load and we have a matching preloaded promise
      if (rulesetMode === initialRulesetMode && !wasmCache.current[cacheKey]) {
        try {
          const wasmModule = await initialWasmPromise;
          wasmCache.current[cacheKey] = wasmModule;

          if (cancelled) return;

          setWasm(wasmModule);
          console.log(`Using preloaded wasm`);
          console.timeEnd(loadingLabel);
          return;
        } catch (e) {
          console.error("Failed to load preloaded wasm", e);
        }
      }

      // If the module is already in cache, use it directly
      if (wasmCache.current[cacheKey]) {
        setWasm(wasmCache.current[cacheKey]);
        console.log(`Using cached wasm`);
        console.timeEnd(loadingLabel);
        return;
      }

      // Set wasm to null to trigger the loading indicator
      setWasm(null);
      const wasmModule = await getWasmImportPath(rulesetMode);

      // Always cache, even if cancelled
      wasmCache.current[cacheKey] = wasmModule;

      if (cancelled) return;

      setWasm(wasmModule);
      console.timeEnd(loadingLabel);
    };
    loadWasm();
    return () => {
      cancelled = true;
    };
  }, [rulesetMode]);

  // Load cgroups.json
  useEffect(() => {
    async function loadCGroups() {
      try {
        const res = await fetch("/cgroups.json");
        const json = await res.json();
        setCGroups({
          data: json.data as { [name: string]: string },
          timestamp: json.timestamp as number,
        });
      } catch (e) {
        console.error("Failed to load cgroups.json", e);
      }
    }
    loadCGroups();
  }, []);

  return (
    <AppContext.Provider value={{ wasm, rulesetMode, setRulesetMode, cgroups }}>
      {children}
    </AppContext.Provider>
  );
};
